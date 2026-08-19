//! First-run setup (PROTOCOL §2.1): one token, one shot.

mod common;

use linger_core::wire::{AuthResponse, StoopInfo};

#[tokio::test]
async fn fresh_stoop_completes_setup_exactly_once() {
    let stoop = common::spawn_stoop().await;
    let token = stoop.state.setup.peek().expect("fresh stoop must arm a setup token");
    let client = reqwest::Client::new();

    // Preview: right token is valid, wrong token isn't.
    let ok: serde_json::Value = client
        .get(stoop.url(&format!("/setup/{token}")))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(ok["valid"], true);
    let bad: serde_json::Value = client
        .get(stoop.url("/setup/not-the-token"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(bad["valid"], false);

    // A bad form must NOT burn the token.
    let short_pw = client
        .post(stoop.url("/setup"))
        .json(&serde_json::json!({
            "token": token, "stoop_name": "home", "username": "matt",
            "display_name": "Matt", "password": "short",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(short_pw.status(), 422);
    assert!(stoop.state.setup.peek().is_some(), "validation failure must not consume the token");

    // Complete for real: host account + stoop name.
    let auth: AuthResponse = client
        .post(stoop.url("/setup"))
        .json(&serde_json::json!({
            "token": token, "stoop_name": "the garage", "username": "matt",
            "display_name": "Matt", "password": "correct horse battery",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(auth.user.is_host);
    assert_eq!(auth.user.username, "matt");

    // Token is dead: preview 404s, a second completion 404s.
    let again = client
        .post(stoop.url("/setup"))
        .json(&serde_json::json!({
            "token": token, "stoop_name": "x", "username": "mallory",
            "display_name": "M", "password": "a long enough password",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(again.status(), 404);
    let preview = client.get(stoop.url(&format!("/setup/{token}"))).send().await.unwrap();
    assert_eq!(preview.status(), 404);

    // The stoop knows its name.
    let info: StoopInfo = client
        .get(stoop.url("/stoop"))
        .bearer_auth(&auth.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(info.name, "the garage");
    assert_eq!(info.member_count, 1);
}
