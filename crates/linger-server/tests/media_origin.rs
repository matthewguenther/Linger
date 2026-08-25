//! The media origin (T-503): uploaded files are served from a host of their
//! own, and the two hosts serve different things.
//!
//! ARCHITECTURE §7 "user content is hostile". An upload is somebody else's
//! bytes, so the defence is not only the headers it comes back with — it is
//! that it comes back from `cdn.<domain>`, where there is no app and no API to
//! be same-origin with. These tests drive real HTTP with the `Host` header set
//! the way a reverse proxy would set it.

mod common;

use common::{bootstrap_host, spawn_named_server, TestServer};
use linger_core::wire::{Attachment, UploadSlot};

const APP: &str = "linger.example";
const CDN: &str = "cdn.linger.example";

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

/// The path out of an absolute URL, so a request the server addressed to
/// `https://cdn.linger.example/...` can actually be sent to the test port.
fn path_of(url: &str) -> String {
    let Some((_, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    match rest.split_once('/') {
        Some((_, path)) => format!("/{path}"),
        None => "/".to_string(),
    }
}

/// Ask this server for `path` as though the request had arrived for `host`.
async fn get_as(server: &TestServer, host: &str, path: &str) -> reqwest::Response {
    client()
        .get(format!("{}{path}", server.base))
        .header(reqwest::header::HOST, host)
        .send()
        .await
        .unwrap()
}

/// Slot, PUT, complete — the whole upload, with the slot URL pointed back at
/// the ephemeral test port.
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
            "size_bytes": bytes.len(),
            "mime": mime,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let resp = client()
        .put(format!("{}{}", server.base, path_of(&slot.url)))
        .header(reqwest::header::HOST, APP)
        .body(bytes)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "PUT of the bytes was refused");

    let resp = client()
        .post(server.url(&format!("/uploads/{}/complete", slot.upload_id)))
        .bearer_auth(token)
        .json(&serde_json::json!({ "parts": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "complete refused: {}",
        resp.text().await.unwrap()
    );
    resp.json().await.unwrap()
}

fn png() -> Vec<u8> {
    let mut canvas = image::RgbaImage::new(8, 8);
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        *pixel = image::Rgba([(x * 8) as u8, (y * 8) as u8, 128, 255]);
    }
    let mut out = std::io::Cursor::new(Vec::new());
    canvas
        .write_to(&mut out, image::ImageFormat::Png)
        .expect("encode");
    out.into_inner()
}

#[tokio::test]
async fn a_file_is_served_from_the_media_host_and_only_from_there() {
    let server = spawn_named_server(APP, CDN).await;
    let host = bootstrap_host(&server).await;
    let attachment = upload(
        &server,
        &host.access_token,
        "holiday.png",
        "image/png",
        png(),
    )
    .await;

    assert_eq!(
        attachment.url,
        format!("https://{CDN}{}", path_of(&attachment.url)),
        "an attachment URL points at the media origin"
    );

    let served = get_as(&server, CDN, &path_of(&attachment.url)).await;
    assert_eq!(served.status(), 200);
    assert_eq!(served.headers()["content-type"], "image/png");

    // The same bytes, asked for on the app's own name. There is nothing there.
    let refused = get_as(&server, APP, &path_of(&attachment.url)).await;
    assert_eq!(
        refused.status(),
        404,
        "an upload must not be reachable from the app origin"
    );
}

#[tokio::test]
async fn the_media_host_has_no_api_on_it() {
    let server = spawn_named_server(APP, CDN).await;

    let app = get_as(&server, APP, "/api/v1/health").await;
    assert_eq!(app.status(), 200);

    // A file that talked a browser into running it would find nothing to call.
    for path in ["/api/v1/health", "/api/v1/server", "/api/v1/gateway"] {
        let resp = get_as(&server, CDN, path).await;
        assert_eq!(resp.status(), 404, "{path} answered on the media host");
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["error"]["code"], "NOT_FOUND");
    }
}

#[tokio::test]
async fn a_served_file_says_it_may_do_nothing() {
    let server = spawn_named_server(APP, CDN).await;
    let host = bootstrap_host(&server).await;

    let image = upload(
        &server,
        &host.access_token,
        "holiday.png",
        "image/png",
        png(),
    )
    .await;
    let served = get_as(&server, CDN, &path_of(&image.url)).await;
    assert_eq!(served.headers()["x-content-type-options"], "nosniff");
    assert_eq!(
        served.headers()["content-security-policy"],
        "default-src 'none'; sandbox"
    );
    assert_eq!(
        served.headers()["cross-origin-resource-policy"],
        "cross-origin"
    );
    assert!(served.headers()["content-disposition"]
        .to_str()
        .unwrap()
        .starts_with("inline;"));

    // Anything off the inline allowlist is a download, and is not described as
    // whatever the uploader called it.
    let notes = upload(
        &server,
        &host.access_token,
        "notes.txt",
        "text/plain",
        b"just some notes".to_vec(),
    )
    .await;
    let served = get_as(&server, CDN, &path_of(&notes.url)).await;
    assert_eq!(served.headers()["content-type"], "application/octet-stream");
    assert!(served.headers()["content-disposition"]
        .to_str()
        .unwrap()
        .starts_with("attachment;"));
    assert_eq!(served.headers()["x-content-type-options"], "nosniff");
}

#[tokio::test]
async fn the_upload_listener_stays_on_the_app_host() {
    let server = spawn_named_server(APP, CDN).await;
    let host = bootstrap_host(&server).await;
    let slot: UploadSlot = client()
        .post(server.url("/uploads"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({
            "filename": "notes.txt", "size_bytes": 4, "mime": "text/plain",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(
        slot.url.starts_with(&format!("https://{APP}/upload/")),
        "bytes go to the app host, not the media one: {}",
        slot.url
    );
    let resp = client()
        .put(format!("{}{}", server.base, path_of(&slot.url)))
        .header(reqwest::header::HOST, CDN)
        .body(b"four".to_vec())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404, "the media host takes no uploads");
}
