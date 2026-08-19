//! Integration-test harness pattern (AGENTS.md): real HTTP against the real
//! router with a temp SQLite file. Every M1+ endpoint test starts from
//! [`spawn_stoop`] — copy this shape, don't mock the router.

use linger_core::wire::{ErrorCode, ErrorEnvelope};
use linger_server::{config::Config, db, AppState};

/// Boot a fully wired server on an ephemeral port; returns its base URL.
/// The `TempDir` is returned so the database outlives the test body.
async fn spawn_stoop() -> (String, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        bind: "127.0.0.1:0".parse().unwrap(),
        domain: None,
        storage: linger_server::config::Storage::Local,
    };
    let database = db::init(&config.db_path()).await.expect("db init");
    let app = linger_server::app(AppState::new(database, config));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), dir)
}

#[tokio::test]
async fn health_reports_ok_with_a_live_database() {
    let (base, _dir) = spawn_stoop().await;
    let body: serde_json::Value = reqwest::get(format!("{base}/api/v1/health"))
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
    let (base, _dir) = spawn_stoop().await;
    let resp = reqwest::get(format!("{base}/api/v1/definitely-not-a-route"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let env: ErrorEnvelope = resp.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::NotFound);
    assert!(!env.error.message.is_empty());
}
