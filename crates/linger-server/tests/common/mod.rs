//! Shared integration-test harness (AGENTS.md): every test drives real HTTP
//! against the production router with a temp SQLite file. No mocks.
#![allow(dead_code)] // each test binary uses a different subset of helpers

use linger_core::wire::{AuthResponse, Invite};
use linger_server::config::{Config, Storage};
use linger_server::{db, AppState};

pub struct TestStoop {
    pub base: String,
    pub state: AppState,
    _dir: tempfile::TempDir,
}

impl TestStoop {
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("{}/api/v1{path}", self.base)
    }

    /// The WS gateway URL.
    #[must_use]
    pub fn gateway_url(&self) -> String {
        format!("{}/api/v1/gateway", self.base.replace("http://", "ws://"))
    }
}

/// Boot a fully wired server on an ephemeral port.
pub async fn spawn_stoop() -> TestStoop {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        bind: "127.0.0.1:0".parse().unwrap(),
        domain: None,
        storage: Storage::Local,
    };
    let database = db::init(&config.db_path()).await.expect("db init");
    let state = AppState::build(database, config).await.expect("state");
    let app = linger_server::app(state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    TestStoop {
        base: format!("http://{addr}"),
        state,
        _dir: dir,
    }
}

/// Complete first-run setup: creates the host account and names the stoop.
pub async fn bootstrap_host(stoop: &TestStoop) -> AuthResponse {
    let token = stoop
        .state
        .setup
        .peek()
        .expect("fresh stoop has a setup token");
    let resp = reqwest::Client::new()
        .post(stoop.url("/setup"))
        .json(&serde_json::json!({
            "token": token,
            "stoop_name": "test stoop",
            "username": "matt",
            "display_name": "Matt",
            "password": "correct horse battery",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "setup failed: {}",
        resp.text().await.unwrap()
    );
    resp.json().await.unwrap()
}

/// Invite + register a member, via the real endpoints.
pub async fn join_member(stoop: &TestStoop, host_access: &str, username: &str) -> AuthResponse {
    let client = reqwest::Client::new();
    let invite: Invite = client
        .post(stoop.url("/invites"))
        .bearer_auth(host_access)
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let resp = client
        .post(stoop.url("/auth/register"))
        .json(&serde_json::json!({
            "invite_code": invite.code,
            "username": username,
            "display_name": username,
            "password": "a perfectly fine password",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "register failed: {}",
        resp.text().await.unwrap()
    );
    resp.json().await.unwrap()
}

/// Host + one room, the most common fixture.
pub async fn stoop_with_room(slug: &str) -> (TestStoop, AuthResponse, linger_core::wire::Room) {
    let stoop = spawn_stoop().await;
    let host = bootstrap_host(&stoop).await;
    let room: linger_core::wire::Room = reqwest::Client::new()
        .post(stoop.url("/rooms"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "slug": slug, "name": format!("#{slug}") }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    (stoop, host, room)
}
