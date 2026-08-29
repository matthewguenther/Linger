//! Knock (SPEC §4.9, PROTOCOL §7, T-1101) over real HTTP and real WebSockets.
//!
//! The thing worth testing here is the *audience*. Every other frame on this
//! gateway goes to everybody (or to a room); a knock goes to one person, and
//! getting that wrong is not a visible bug — it is somebody's nudge landing on
//! a stranger's screen. So every assertion below is either "it arrived" or
//! "it did not arrive anywhere else".

mod common;

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn recv_json(ws: &mut Ws) -> Value {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("gateway frame within 5s")
            .expect("socket still open")
            .expect("clean frame");
        if let WsMessage::Text(text) = msg {
            return serde_json::from_str(text.as_str()).expect("valid frame json");
        }
    }
}

/// Read frames until one matches, or give up. Returns everything seen.
async fn wait_for(ws: &mut Ws, op: &str) -> Value {
    loop {
        let frame = recv_json(ws).await;
        if frame["op"] == op {
            return frame;
        }
    }
}

/// Everything that arrives inside `window`. The only way to assert a frame did
/// *not* go somewhere is to wait and look at what did.
async fn drain_for(ws: &mut Ws, window: Duration) -> Vec<Value> {
    let mut frames = Vec::new();
    loop {
        match tokio::time::timeout(window, ws.next()).await {
            Ok(Some(Ok(WsMessage::Text(text)))) => {
                frames.push(serde_json::from_str(text.as_str()).expect("valid frame json"));
            }
            Ok(Some(Ok(_))) => {}
            _ => return frames,
        }
    }
}

async fn send_json(ws: &mut Ws, value: Value) {
    ws.send(WsMessage::Text(value.to_string().into()))
        .await
        .expect("ws send");
}

/// Full handshake: hello → identify → ready.
async fn connect_ready(server: &common::TestServer, token: &str) -> Ws {
    let (mut ws, _) = connect_async(server.gateway_url())
        .await
        .expect("ws connect");
    let hello = recv_json(&mut ws).await;
    assert_eq!(hello["op"], "hello");
    send_json(
        &mut ws,
        json!({ "op": "identify", "d": { "token": token, "client": "test/0" } }),
    )
    .await;
    wait_for(&mut ws, "ready").await;
    ws
}

async fn knock(
    server: &common::TestServer,
    token: &str,
    target: linger_core::UserId,
) -> reqwest::Response {
    reqwest::Client::new()
        .post(server.url("/knock"))
        .bearer_auth(token)
        .json(&json!({ "target_user_id": target.to_string() }))
        .send()
        .await
        .expect("POST /knock")
}

/// The audience test: it lands on the person it names, on every machine they
/// have open, and on nobody else — not the sender, not a bystander.
#[tokio::test]
async fn a_knock_reaches_only_its_target() {
    let (server, host, _room) = common::server_with_room("garage").await;
    let callie = common::join_member(&server, &host.access_token, "callie").await;
    let dev = common::join_member(&server, &host.access_token, "dev").await;

    let mut knocker = connect_ready(&server, &host.access_token).await;
    let mut target = connect_ready(&server, &callie.access_token).await;
    // The same person, signed in twice. A knock has to reach both, or it is a
    // knock somebody misses by having picked up the wrong laptop.
    let mut target_laptop = connect_ready(&server, &callie.access_token).await;
    let mut bystander = connect_ready(&server, &dev.access_token).await;

    let resp = knock(&server, &host.access_token, callie.user.id).await;
    assert_eq!(resp.status(), 204);

    for ws in [&mut target, &mut target_laptop] {
        let frame = wait_for(ws, "knock").await;
        assert_eq!(frame["d"]["from_user_id"], host.user.id.to_string());
        // Sequenced like every other fan-out frame, so a resume inside the
        // window replays it rather than leaving a hole in the numbering.
        assert!(frame["s"].as_u64().is_some_and(|s| s >= 1));
        // Who it was addressed to is not on the wire: they know.
        assert!(frame["d"].get("target_user_id").is_none());
    }

    for ws in [&mut bystander, &mut knocker] {
        let frames = drain_for(ws, Duration::from_millis(400)).await;
        assert!(
            !frames.iter().any(|f| f["op"] == "knock"),
            "a knock must reach nobody but its target, got {frames:?}"
        );
    }
}

/// Three an hour, per target (`RATE_KNOCK_PER_TARGET`). The fourth is refused,
/// and the refusal says how long to wait.
#[tokio::test]
async fn the_fourth_knock_inside_an_hour_is_refused() {
    let (server, host, _room) = common::server_with_room("garage").await;
    let callie = common::join_member(&server, &host.access_token, "callie").await;

    for attempt in 1..=3 {
        let resp = knock(&server, &host.access_token, callie.user.id).await;
        assert_eq!(resp.status(), 204, "knock {attempt} should be allowed");
    }

    let resp = knock(&server, &host.access_token, callie.user.id).await;
    assert_eq!(resp.status(), 429);
    let body: Value = resp.json().await.expect("error envelope");
    assert_eq!(body["error"]["code"], "RATE_LIMITED");
    assert!(
        body["error"]["retry_after_ms"]
            .as_u64()
            .is_some_and(|ms| ms > 0),
        "a refusal has to say how long to wait: {body}"
    );
}

/// The bucket is keyed by both ends, so knocking five different people is five
/// separate buckets — being unable to knock Callie must not mean being unable
/// to knock anybody.
#[tokio::test]
async fn the_limit_is_per_target_not_per_knocker() {
    let (server, host, _room) = common::server_with_room("garage").await;
    let callie = common::join_member(&server, &host.access_token, "callie").await;
    let dev = common::join_member(&server, &host.access_token, "dev").await;

    for _ in 0..3 {
        assert_eq!(
            knock(&server, &host.access_token, callie.user.id)
                .await
                .status(),
            204
        );
    }
    assert_eq!(
        knock(&server, &host.access_token, callie.user.id)
            .await
            .status(),
        429
    );
    assert_eq!(
        knock(&server, &host.access_token, dev.user.id)
            .await
            .status(),
        204,
        "a different person is a different bucket"
    );
}

/// Somebody the host removed is not on this server any more, and the answer is
/// the same one every other endpoint gives about them.
#[tokio::test]
async fn knocking_a_removed_member_is_not_found() {
    let (server, host, _room) = common::server_with_room("garage").await;
    let callie = common::join_member(&server, &host.access_token, "callie").await;

    let removed = reqwest::Client::new()
        .post(server.url(&format!("/users/{}/remove", callie.user.id)))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .expect("remove");
    assert_eq!(removed.status(), 204);

    let resp = knock(&server, &host.access_token, callie.user.id).await;
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.expect("error envelope");
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// A knock at somebody who has never existed is the same answer, and a knock at
/// yourself is a validation error rather than a card from yourself.
#[tokio::test]
async fn a_stranger_is_not_found_and_you_cannot_knock_on_yourself() {
    let (server, host, _room) = common::server_with_room("garage").await;

    let nobody = linger_core::UserId::new();
    assert_eq!(
        knock(&server, &host.access_token, nobody).await.status(),
        404
    );

    let resp = knock(&server, &host.access_token, host.user.id).await;
    assert_eq!(resp.status(), 422);
    let body: Value = resp.json().await.expect("error envelope");
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
}

/// Signed out, there is nothing to knock with.
#[tokio::test]
async fn a_knock_needs_a_token() {
    let (server, host, _room) = common::server_with_room("garage").await;
    let callie = common::join_member(&server, &host.access_token, "callie").await;

    let resp = reqwest::Client::new()
        .post(server.url("/knock"))
        .json(&json!({ "target_user_id": callie.user.id.to_string() }))
        .send()
        .await
        .expect("POST /knock");
    assert_eq!(resp.status(), 401);
}

/// Nothing is written down (SPEC §4.9). The database after a knock is the
/// database before it — there is no knocks table, and this test is here to fail
/// loudly if somebody ever adds one.
#[tokio::test]
async fn a_knock_is_not_a_row() {
    let (server, host, _room) = common::server_with_room("garage").await;
    let callie = common::join_member(&server, &host.access_token, "callie").await;

    assert_eq!(
        knock(&server, &host.access_token, callie.user.id)
            .await
            .status(),
        204
    );

    let tables: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM sqlite_master WHERE type = 'table'")
            .fetch_all(&server.state.db.read)
            .await
            .expect("read the schema");
    assert!(
        !tables.iter().any(|(name,)| name.contains("knock")),
        "a knock is not stored: {tables:?}"
    );
}
