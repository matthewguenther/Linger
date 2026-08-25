//! The bytes. Neither of these paths is under `/api/v1`, and that is the point
//! (ARCHITECTURE §8): uploads and downloads never travel through the JSON API.
//!
//! - `PUT /upload/:upload_id/:part` — the local backend's listener. No session,
//!   no header, no database write: the URL was signed by this server when the
//!   slot was created and that signature is the whole authorisation. It is the
//!   same shape as an S3 presigned URL on purpose, so the client does one thing
//!   whichever backend a server runs (T-502).
//! - `GET /objects/*key` — serving a stored object back.
//!
//! **Why serving is not authenticated.** An object key contains a UUIDv7 with
//! 74 random bits, so the URL is the secret — the same arrangement every chat
//! app uses, and the only one that lets an `<img>` tag work at all. What keeps
//! a hostile upload harmless is not a login check but the headers below:
//! anything that is not an ordinary image, video or audio file is served as a
//! download, with sniffing turned off, so it cannot become a page. T-503
//! finishes the job by moving these responses to their own origin, where they
//! are not same-site with anybody's session.

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::Router;
use linger_core::media;
use linger_core::{AttachmentId, UploadId};
use serde::Deserialize;
use sqlx::Row;

use crate::error::ApiError;
use crate::state::AppState;
use crate::storage::local::PartError;
use crate::storage::{part_plan, ObjectBody};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload/{upload_id}/{part}", put(put_part))
        .route("/objects/{*key}", get(get_object))
}

#[derive(Deserialize)]
struct Signature {
    exp: i64,
    sig: String,
}

/// Accept one part of an upload.
async fn put_part(
    State(state): State<AppState>,
    Path((upload_id, part)): Path<(UploadId, u32)>,
    Query(signature): Query<Signature>,
    body: Body,
) -> Result<Response, ApiError> {
    let Some(local) = state.local.as_ref() else {
        return Err(ApiError::not_found("No such thing on this server."));
    };
    if !local.verify_part_url(upload_id, part, signature.exp, &signature.sig) {
        // Wrong signature, wrong part, or the link has aged out. The client's
        // move is the same in all three cases: ask for a fresh slot.
        return Err(ApiError::forbidden(
            "That upload link isn't valid any more.",
        ));
    }

    let record = crate::repo::attachments::record(&state.db.read, AttachmentId(upload_id.0))
        .await?
        .ok_or_else(|| ApiError::not_found("No such upload."))?;
    if record.state != "pending" {
        return Err(ApiError::conflict("That upload is already finished."));
    }

    let (count, part_size) = part_plan(record.size_bytes);
    if part == 0 || part > count {
        return Err(ApiError::validation("That upload has no such part."));
    }
    // The last part is whatever is left over; every other one is a full part.
    let max_bytes = if part == count {
        record.size_bytes - part_size * u64::from(count - 1)
    } else {
        part_size
    };

    match local.write_part(upload_id, part, max_bytes, body).await {
        Ok(etag) => {
            let mut headers = HeaderMap::new();
            if let Ok(value) = HeaderValue::from_str(&format!("\"{etag}\"")) {
                headers.insert(header::ETAG, value);
            }
            Ok((StatusCode::OK, headers).into_response())
        }
        Err(PartError::TooLarge) => Err(ApiError::file_too_large(
            "That part is bigger than the slot it was for.",
        )),
        Err(PartError::Aborted(why)) => {
            // Normal, and the reason multipart exists: resend this one part.
            tracing::debug!(%why, "upload part did not finish");
            Err(ApiError::validation("That part didn't finish arriving."))
        }
        Err(PartError::Io(err)) => {
            tracing::error!(error = %err, "writing upload part");
            Err(ApiError::internal())
        }
    }
}

/// Serve a stored object.
async fn get_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Response, ApiError> {
    let row = sqlx::query(
        "SELECT filename, mime, poster_key FROM attachments
         WHERE state = 'complete' AND (object_key = ? OR poster_key = ?)",
    )
    .bind(&key)
    .bind(&key)
    .fetch_optional(&state.db.read)
    .await?
    .ok_or_else(|| ApiError::not_found("No such file."))?;

    let is_poster = row.get::<Option<String>, _>("poster_key").as_deref() == Some(key.as_str());
    let filename: String = row.get("filename");
    let (filename, mime) = if is_poster {
        // A generated poster frame is this server's own JPEG, not the upload.
        (format!("{filename}.jpg"), "image/jpeg".to_string())
    } else {
        (filename, row.get::<String, _>("mime"))
    };

    let Some(object) = state
        .storage
        .read_object(&key)
        .await
        .map_err(ApiError::from)?
    else {
        return Err(ApiError::not_found("No such file."));
    };

    let (body, length) = match object {
        ObjectBody::Redirect(url) => {
            return Ok(axum::response::Redirect::temporary(&url).into_response())
        }
        ObjectBody::File(path, length) => {
            let file = tokio::fs::File::open(&path).await.map_err(|err| {
                tracing::error!(error = %err, "opening stored object");
                ApiError::internal()
            })?;
            (
                Body::from_stream(tokio_util::io::ReaderStream::new(file)),
                length,
            )
        }
    };

    let inline = media::is_inline_mime(&mime);
    let mut headers = HeaderMap::new();
    let served_type = if inline {
        mime.as_str()
    } else {
        // Refuse to repeat the uploader's claim about what this is. A browser
        // that is told nothing useful, and told not to guess, cannot be talked
        // into running it.
        "application/octet-stream"
    };
    insert(&mut headers, header::CONTENT_TYPE, served_type);
    insert(&mut headers, header::CONTENT_LENGTH, &length.to_string());
    insert(&mut headers, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    insert(
        &mut headers,
        header::CONTENT_DISPOSITION,
        &content_disposition(inline, &filename),
    );
    // Objects are immutable: the key contains the id, and re-encoding happens
    // once, before the key is ever handed out.
    insert(
        &mut headers,
        header::CACHE_CONTROL,
        "public, max-age=31536000, immutable",
    );
    // The client is a webview on a different origin, and T-503 moves these
    // responses to a different host again, so say plainly that loading them
    // from elsewhere is allowed.
    insert(&mut headers, "cross-origin-resource-policy", "cross-origin");

    Ok((headers, body).into_response())
}

fn insert(headers: &mut HeaderMap, name: impl axum::http::header::IntoHeaderName, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        headers.insert(name, value);
    }
}

/// `Content-Disposition`, with the filename twice: a plain ASCII version every
/// browser understands, and the real one percent-encoded for the rest of the
/// alphabet (RFC 6266).
fn content_disposition(inline: bool, filename: &str) -> String {
    let kind = if inline { "inline" } else { "attachment" };
    let ascii: String = filename
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let encoded: String = filename
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_') {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect();
    format!("{kind}; filename=\"{ascii}\"; filename*=UTF-8''{encoded}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_download_header_cannot_be_talked_into_a_second_line() {
        let header = content_disposition(false, "report \"final\".pdf");
        assert!(header.starts_with("attachment; "));
        assert!(!header.contains('\n') && !header.contains('\r'));
        assert_eq!(header.matches("filename=\"").count(), 1);
        assert!(header.contains("filename=\"report _final_.pdf\""));
    }

    #[test]
    fn non_ascii_names_survive_in_the_encoded_form() {
        let header = content_disposition(true, "naïve.png");
        assert!(header.starts_with("inline; "));
        assert!(header.contains("filename*=UTF-8''na%C3%AFve.png"));
    }
}
