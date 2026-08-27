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
//! a hostile upload harmless is not a login check but where it is served from
//! and how: `/objects` answers only on the media host (`cdn.<domain>`, see
//! `super::media_origin_gate`), so nothing served here is ever same-origin with
//! the app, and anything that is not an ordinary image, video or audio file is
//! handed over as a download with sniffing turned off and a CSP that permits
//! nothing at all.
//!
//! On the S3 backend this route answers with a redirect and the *bucket* sends
//! the response, so the two headers that decide whether a file can render are
//! both stored on the object and signed into the presigned URL
//! ([`crate::storage::ServeAs`]). S3 has no `response-` override for
//! `X-Content-Type-Options` or `Content-Security-Policy`, so those two do not
//! make that trip; what stands in for them there is that the content type is
//! never the uploader's claim — it is one of the thirteen media types this
//! server sniffed for itself, or `application/octet-stream` with
//! `Content-Disposition: attachment`, which no browser renders whatever it
//! decides the bytes are. Active content (SVG, HTML, scripts) cannot be stored
//! in the first place (`linger_core::media`).

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::Router;
use linger_core::{AttachmentId, UploadId};
use serde::Deserialize;
use sqlx::Row;

use crate::error::ApiError;
use crate::state::AppState;
use crate::storage::local::PartError;
use crate::storage::{part_plan, ObjectBody, ServeAs};

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
///
/// Two kinds of thing live in the store and both are served from here, because
/// both are somebody's bytes and both belong on the media host rather than on
/// the app's own name: uploads, and the export archives in [`crate::export`].
/// An export key never looks like an attachment key (`exports/<id>.zip`), so
/// the two lookups cannot collide.
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
    .await?;

    let Some(row) = row else {
        return get_export(&state, &key).await;
    };

    let is_poster = row.get::<Option<String>, _>("poster_key").as_deref() == Some(key.as_str());
    let filename: String = row.get("filename");
    let serve = if is_poster {
        ServeAs::poster(&filename)
    } else {
        ServeAs::for_object(&row.get::<String, _>("mime"), &filename)
    };

    // Handed down as well as sent, because a store that answers with a redirect
    // has no response of its own to put them on: S3 signs these two into the
    // URL, and the bucket sends them back.
    let Some(object) = state
        .storage
        .read_object(&key, &serve)
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

    let mut headers = HeaderMap::new();
    insert(&mut headers, header::CONTENT_TYPE, &serve.content_type);
    insert(&mut headers, header::CONTENT_LENGTH, &length.to_string());
    insert(&mut headers, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    insert(
        &mut headers,
        header::CONTENT_DISPOSITION,
        &serve.disposition,
    );
    // Belt and braces on top of the type and the disposition: if a browser ever
    // did render one of these, it would render it with no scripts, no network
    // and no origin of its own to reach anything from.
    insert(
        &mut headers,
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; sandbox",
    );
    // Objects are immutable: the key contains the id, and re-encoding happens
    // once, before the key is ever handed out.
    insert(
        &mut headers,
        header::CACHE_CONTROL,
        "public, max-age=31536000, immutable",
    );
    // The client is a webview on one origin and this is another host again, so
    // say plainly that loading these from elsewhere is allowed.
    insert(&mut headers, "cross-origin-resource-policy", "cross-origin");

    Ok((headers, body).into_response())
}

/// Serve a finished export archive.
///
/// Unauthenticated for the same reason an upload is: the key holds a UUIDv7
/// with 74 random bits, so the URL is the secret. That matters more here than
/// it does for one photo — this is the whole server in a file — which is why
/// asking for somebody else's *job* is a 404 (see [`crate::export::job`]) and
/// why a new export deletes the previous archive rather than leaving old URLs
/// working forever.
async fn get_export(state: &AppState, key: &str) -> Result<Response, ApiError> {
    let row = sqlx::query(
        "SELECT filename, size_bytes FROM exports WHERE state = 'complete' AND object_key = ?",
    )
    .bind(key)
    .fetch_optional(&state.db.read)
    .await?
    .ok_or_else(|| ApiError::not_found("No such file."))?;

    let filename: String = row
        .get::<Option<String>, _>("filename")
        .unwrap_or_else(|| "linger-export.zip".to_string());
    let serve = ServeAs {
        content_type: "application/zip".to_string(),
        disposition: format!("attachment; filename=\"{filename}\""),
    };

    let Some(object) = state
        .storage
        .read_object(key, &serve)
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
                tracing::error!(error = %err, "opening an export archive");
                ApiError::internal()
            })?;
            (
                Body::from_stream(tokio_util::io::ReaderStream::new(file)),
                length,
            )
        }
    };

    let mut headers = HeaderMap::new();
    insert(&mut headers, header::CONTENT_TYPE, &serve.content_type);
    insert(&mut headers, header::CONTENT_LENGTH, &length.to_string());
    insert(&mut headers, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    insert(
        &mut headers,
        header::CONTENT_DISPOSITION,
        &serve.disposition,
    );
    insert(
        &mut headers,
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; sandbox",
    );
    // An archive is a snapshot and is replaced rather than revised, but the URL
    // stops working the moment its owner asks for another one — so it is not
    // the year-long immutable cache an upload gets.
    insert(&mut headers, header::CACHE_CONTROL, "private, no-store");
    insert(&mut headers, "cross-origin-resource-policy", "cross-origin");

    Ok((headers, body).into_response())
}

fn insert(headers: &mut HeaderMap, name: impl axum::http::header::IntoHeaderName, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        headers.insert(name, value);
    }
}
