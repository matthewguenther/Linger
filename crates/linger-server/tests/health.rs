//! Smoke tests for the harness itself: health, and the error envelope on
//! unknown paths. The endpoint suites live in their own files.

mod common;

use linger_core::wire::{ErrorCode, ErrorEnvelope};

#[tokio::test]
async fn health_reports_ok_with_a_live_database() {
    let server = common::spawn_server().await;
    let body: serde_json::Value = reqwest::get(server.url("/health"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["ok"], true);
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn unknown_api_paths_return_the_protocol_error_envelope() {
    let server = common::spawn_server().await;
    let resp = reqwest::get(server.url("/definitely-not-a-route"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let env: ErrorEnvelope = resp.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::NotFound);
    assert!(!env.error.message.is_empty());
}

#[tokio::test]
async fn protected_routes_reject_missing_and_garbage_tokens() {
    let server = common::spawn_server().await;
    let client = reqwest::Client::new();

    let bare = client.get(server.url("/users")).send().await.unwrap();
    assert_eq!(bare.status(), 401);
    let env: ErrorEnvelope = bare.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::Unauthenticated);

    let garbage = client
        .get(server.url("/users"))
        .bearer_auth("not-a-jwt")
        .send()
        .await
        .unwrap();
    assert_eq!(garbage.status(), 401);
}

/// The desktop client is a webview page, so every call it makes is cross-origin
/// and the browser will not hand it a response without these headers. Without
/// this the whole client silently fails to reach the server, which is exactly
/// what happened during T-301 — hence a test rather than a comment.
#[tokio::test]
async fn the_desktop_client_origin_is_allowed_and_others_are_not() {
    let server = common::spawn_server().await;
    let client = reqwest::Client::new();

    for origin in ["tauri://localhost", "http://tauri.localhost"] {
        // The preflight the browser sends before a JSON POST with a bearer token.
        let preflight = client
            .request(reqwest::Method::OPTIONS, server.url("/auth/login"))
            .header("origin", origin)
            .header("access-control-request-method", "POST")
            .header(
                "access-control-request-headers",
                "authorization,content-type",
            )
            .send()
            .await
            .unwrap();
        assert!(preflight.status().is_success(), "preflight for {origin}");
        assert_eq!(
            preflight
                .headers()
                .get("access-control-allow-origin")
                .and_then(|v| v.to_str().ok()),
            Some(origin),
        );

        let actual = client
            .get(server.url("/health"))
            .header("origin", origin)
            .send()
            .await
            .unwrap();
        assert_eq!(
            actual
                .headers()
                .get("access-control-allow-origin")
                .and_then(|v| v.to_str().ok()),
            Some(origin),
        );
    }

    // A web page somewhere else gets no such permission: it can send the
    // request, but the browser won't let it read the answer.
    let stranger = client
        .get(server.url("/health"))
        .header("origin", "https://example.com")
        .send()
        .await
        .unwrap();
    assert!(stranger
        .headers()
        .get("access-control-allow-origin")
        .is_none());
}
