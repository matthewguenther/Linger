//! Smoke tests for the harness itself: health, and the error envelope on
//! unknown paths. The endpoint suites live in their own files.

mod common;

use linger_core::wire::{ErrorCode, ErrorEnvelope};

#[tokio::test]
async fn health_reports_ok_with_a_live_database() {
    let stoop = common::spawn_stoop().await;
    let body: serde_json::Value = reqwest::get(stoop.url("/health"))
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
    let stoop = common::spawn_stoop().await;
    let resp = reqwest::get(stoop.url("/definitely-not-a-route"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let env: ErrorEnvelope = resp.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::NotFound);
    assert!(!env.error.message.is_empty());
}

#[tokio::test]
async fn protected_routes_reject_missing_and_garbage_tokens() {
    let stoop = common::spawn_stoop().await;
    let client = reqwest::Client::new();

    let bare = client.get(stoop.url("/users")).send().await.unwrap();
    assert_eq!(bare.status(), 401);
    let env: ErrorEnvelope = bare.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::Unauthenticated);

    let garbage = client
        .get(stoop.url("/users"))
        .bearer_auth("not-a-jwt")
        .send()
        .await
        .unwrap();
    assert_eq!(garbage.status(), 401);
}
