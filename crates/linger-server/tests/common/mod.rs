//! Shared integration-test harness (AGENTS.md): every test drives real HTTP
//! against the production router with a temp SQLite file. No mocks.
#![allow(dead_code)] // each test binary uses a different subset of helpers

use linger_core::wire::{AuthResponse, Invite};
use linger_server::config::{Config, Storage};
use linger_server::{db, AppState};

pub struct TestServer {
    pub base: String,
    pub state: AppState,
    _dir: tempfile::TempDir,
}

impl TestServer {
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
pub async fn spawn_server() -> TestServer {
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
    TestServer {
        base: format!("http://{addr}"),
        state,
        _dir: dir,
    }
}

/// Complete first-run setup: creates the host account and names the server.
pub async fn bootstrap_host(server: &TestServer) -> AuthResponse {
    let token = server
        .state
        .setup
        .peek()
        .expect("fresh server has a setup token");
    let resp = reqwest::Client::new()
        .post(server.url("/setup"))
        .json(&serde_json::json!({
            "token": token,
            "server_name": "test server",
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
pub async fn join_member(server: &TestServer, host_access: &str, username: &str) -> AuthResponse {
    let client = reqwest::Client::new();
    let invite: Invite = client
        .post(server.url("/invites"))
        .bearer_auth(host_access)
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let resp = client
        .post(server.url("/auth/register"))
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
pub async fn server_with_room(slug: &str) -> (TestServer, AuthResponse, linger_core::wire::Room) {
    let server = spawn_server().await;
    let host = bootstrap_host(&server).await;
    let room: linger_core::wire::Room = reqwest::Client::new()
        .post(server.url("/rooms"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "slug": slug, "name": format!("#{slug}") }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    (server, host, room)
}
