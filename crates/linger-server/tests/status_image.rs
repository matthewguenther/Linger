//! The image on a status (T-506, SPEC §4.6), end to end over real HTTP.
//!
//! Everything else on a status is somebody's own words. The image is a *name*
//! for a file, so the tests here are mostly about the four questions the server
//! has to ask about that name — does it exist, is it theirs, is it an image, is
//! it small enough — plus what happens to the bytes when a status stops
//! pointing at them.

mod common;

use common::{bootstrap_host, join_member, server_with_room, spawn_server, TestServer};
use linger_core::limits::MAX_STATUS_IMAGE_BYTES;
use linger_core::wire::{Attachment, Message, Room, UploadSlot, User};
use linger_server::expiry;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

/// A PNG of pseudo-random pixels, which is the only kind that does not shrink
/// to nothing: the server re-encodes every image it takes, and a gradient comes
/// back a few hundred bytes long however big it started.
fn noisy_png(side: u32) -> Vec<u8> {
    let mut seed: u32 = 0x1234_5678;
    let mut canvas = image::RgbImage::new(side, side);
    for pixel in canvas.pixels_mut() {
        let mut next = || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            #[allow(clippy::cast_possible_truncation)]
            {
                (seed >> 16) as u8
            }
        };
        *pixel = image::Rgb([next(), next(), next()]);
    }
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(canvas)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// Slot, PUT, complete. Every file in this module fits in one part.
async fn upload(
    server: &TestServer,
    token: &str,
    filename: &str,
    mime: &str,
    bytes: Vec<u8>,
) -> Attachment {
    let slot: UploadSlot = client()
        .post(server.url("/uploads"))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "filename": filename,
            "size_bytes": bytes.len() as u64,
            "mime": mime,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let put = client()
        .put(format!("{}{}", server.base, slot.url))
        .body(bytes)
        .send()
        .await
        .unwrap();
    assert_eq!(put.status(), 200);
    let done = client()
        .post(server.url(&format!("/uploads/{}/complete", slot.upload_id)))
        .bearer_auth(token)
        .json(&serde_json::json!({ "parts": serde_json::Value::Null }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        done.status(),
        200,
        "complete: {}",
        done.text().await.unwrap()
    );
    done.json().await.unwrap()
}

/// A whole status with just the image on it — `PATCH /me` replaces the object,
/// so every save has to send all of it.
fn status_with(image: Option<&Attachment>) -> serde_json::Value {
    serde_json::json!({
        "status": {
            "line": "mounting the drive",
            "reading": null, "listening": null, "working_on": null,
            "image_id": image.map(|a| a.id.to_string()),
            "image_url": null,
            "away_message": null, "away_since": null
        }
    })
}

async fn save_status(
    server: &TestServer,
    token: &str,
    body: serde_json::Value,
) -> reqwest::Response {
    client()
        .patch(server.url("/me"))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .unwrap()
}

/// Save an image onto a status and expect it to stick.
async fn set_image(server: &TestServer, token: &str, image: Option<&Attachment>) -> User {
    let resp = save_status(server, token, status_with(image)).await;
    assert_eq!(
        resp.status(),
        200,
        "status refused: {}",
        resp.text().await.unwrap()
    );
    resp.json().await.unwrap()
}

async fn error_code(resp: reqwest::Response) -> String {
    let body: serde_json::Value = resp.json().await.unwrap();
    body["error"]["code"].as_str().unwrap().to_string()
}

/// Is the object still being served? The bytes are what matters, not the row.
async fn object_status(server: &TestServer, url: &str) -> u16 {
    client()
        .get(format!("{}{url}", server.base))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

async fn share(server: &TestServer, token: &str, room: &Room, attachment: &Attachment) -> Message {
    client()
        .post(server.url(&format!("/rooms/{}/messages", room.id)))
        .bearer_auth(token)
        .json(&serde_json::json!({ "body": "look", "attachment_ids": [attachment.id] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// The whole point: set one, and everybody else's roster can draw it.
#[tokio::test]
async fn a_status_image_is_set_once_and_served_to_everyone() {
    let server = spawn_server().await;
    let host = bootstrap_host(&server).await;
    let member = join_member(&server, &host.access_token, "jo").await;

    let image = upload(
        &server,
        &host.access_token,
        "me.png",
        "image/png",
        noisy_png(40),
    )
    .await;
    let saved = set_image(&server, &host.access_token, Some(&image)).await;

    let status = saved.status.expect("status saved");
    assert_eq!(status.image_id, Some(image.id));
    let url = status.image_url.expect("a URL to draw it from");
    assert_eq!(object_status(&server, &url).await, 200);

    // The URL is built from the stored key, never from the string the client
    // sent, and it is the same URL the attachment itself is served from.
    assert_eq!(url, image.url);

    // Somebody else sees the same thing through the roster.
    let users: Vec<User> = client()
        .get(server.url("/users"))
        .bearer_auth(&member.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let seen = users
        .iter()
        .find(|user| user.id == saved.id)
        .expect("the host is in the roster");
    assert_eq!(
        seen.status.as_ref().and_then(|s| s.image_url.as_deref()),
        Some(url.as_str())
    );
}

/// An id is a string somebody chose. Naming a file that belongs to another
/// member has to be a refusal, or a status is a way to read anybody's uploads.
#[tokio::test]
async fn an_image_that_is_not_yours_is_refused() {
    let server = spawn_server().await;
    let host = bootstrap_host(&server).await;
    let member = join_member(&server, &host.access_token, "jo").await;

    let theirs = upload(
        &server,
        &member.access_token,
        "jo.png",
        "image/png",
        noisy_png(40),
    )
    .await;
    let resp = save_status(&server, &host.access_token, status_with(Some(&theirs))).await;
    assert_eq!(resp.status(), 403);
    assert_eq!(error_code(resp).await, "FORBIDDEN");
}

#[tokio::test]
async fn an_id_that_names_nothing_is_refused() {
    let server = spawn_server().await;
    let host = bootstrap_host(&server).await;

    let resp = save_status(
        &server,
        &host.access_token,
        serde_json::json!({
            "status": {
                "line": null, "reading": null, "listening": null, "working_on": null,
                "image_id": linger_core::AttachmentId::new().to_string(),
                "image_url": null, "away_message": null, "away_since": null
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), 422);
    assert_eq!(error_code(resp).await, "VALIDATION_FAILED");
}

/// SPEC §4.6 says 512 KB. The client counts before it uploads; this is the
/// answer for a client that does not.
#[tokio::test]
async fn an_image_over_the_cap_is_refused() {
    let server = spawn_server().await;
    let host = bootstrap_host(&server).await;

    let big = upload(
        &server,
        &host.access_token,
        "big.png",
        "image/png",
        noisy_png(600),
    )
    .await;
    assert!(
        big.size_bytes > MAX_STATUS_IMAGE_BYTES,
        "the fixture has to be over the cap after re-encoding, got {}",
        big.size_bytes
    );

    let resp = save_status(&server, &host.access_token, status_with(Some(&big))).await;
    assert_eq!(resp.status(), 422);
    assert_eq!(error_code(resp).await, "VALIDATION_FAILED");
}

#[tokio::test]
async fn a_file_that_is_not_an_image_is_refused() {
    let server = spawn_server().await;
    let host = bootstrap_host(&server).await;

    let notes = upload(
        &server,
        &host.access_token,
        "notes.txt",
        "text/plain",
        b"nothing to look at".to_vec(),
    )
    .await;
    let resp = save_status(&server, &host.access_token, status_with(Some(&notes))).await;
    assert_eq!(resp.status(), 422);
    assert_eq!(error_code(resp).await, "VALIDATION_FAILED");
}

/// Replacing an image is the ordinary way to change one, and the old file is
/// then unreachable — nothing draws it and the sweeper skips status images.
#[tokio::test]
async fn replacing_an_image_takes_the_old_one_with_it() {
    let server = spawn_server().await;
    let host = bootstrap_host(&server).await;

    let first = upload(
        &server,
        &host.access_token,
        "a.png",
        "image/png",
        noisy_png(40),
    )
    .await;
    let second = upload(
        &server,
        &host.access_token,
        "b.png",
        "image/png",
        noisy_png(48),
    )
    .await;

    set_image(&server, &host.access_token, Some(&first)).await;
    assert_eq!(object_status(&server, &first.url).await, 200);

    set_image(&server, &host.access_token, Some(&second)).await;
    assert_eq!(object_status(&server, &first.url).await, 404);
    assert_eq!(object_status(&server, &second.url).await, 200);

    // And the row goes with the bytes, so the pool stops counting it.
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attachments WHERE id = ?")
        .bind(first.id.to_vec())
        .fetch_one(&server.state.db.read)
        .await
        .unwrap();
    assert_eq!(left, 0);

    // Clearing it takes the second one too.
    set_image(&server, &host.access_token, None).await;
    assert_eq!(object_status(&server, &second.url).await, 404);
}

/// Saving a status without touching the image must not throw the image away —
/// `PATCH /me` replaces the whole object, so an unchanged field arrives looking
/// exactly like a deliberate one.
#[tokio::test]
async fn saving_the_same_image_again_leaves_it_alone() {
    let server = spawn_server().await;
    let host = bootstrap_host(&server).await;

    let image = upload(
        &server,
        &host.access_token,
        "me.png",
        "image/png",
        noisy_png(40),
    )
    .await;
    set_image(&server, &host.access_token, Some(&image)).await;
    let again = set_image(&server, &host.access_token, Some(&image)).await;

    assert_eq!(
        again.status.and_then(|s| s.image_id),
        Some(image.id),
        "the image survived a second save"
    );
    assert_eq!(object_status(&server, &image.url).await, 200);
}

/// A file somebody also shared in a room belongs to that message. Dropping it
/// from a status is not a reason to delete what a room is still showing.
#[tokio::test]
async fn an_image_on_a_message_is_not_deleted_with_the_status() {
    let (server, host, room) = server_with_room("den").await;

    let image = upload(
        &server,
        &host.access_token,
        "me.png",
        "image/png",
        noisy_png(40),
    )
    .await;
    share(&server, &host.access_token, &room, &image).await;

    set_image(&server, &host.access_token, Some(&image)).await;
    set_image(&server, &host.access_token, None).await;

    assert_eq!(object_status(&server, &image.url).await, 200);
}

/// T-505's sweeper skips a status image whatever its age. This is that promise
/// from the other end: through the real endpoint, on a status a year old.
#[tokio::test]
async fn a_status_image_survives_a_year() {
    let server = spawn_server().await;
    let host = bootstrap_host(&server).await;

    let image = upload(
        &server,
        &host.access_token,
        "me.png",
        "image/png",
        noisy_png(40),
    )
    .await;
    set_image(&server, &host.access_token, Some(&image)).await;

    sqlx::query("UPDATE attachments SET created_at = created_at - ?")
        .bind(400 * DAY_MS)
        .execute(&server.state.db.write)
        .await
        .unwrap();

    assert_eq!(expiry::sweep(&server.state).await.unwrap().files, 0);
    assert_eq!(object_status(&server, &image.url).await, 200);
}
