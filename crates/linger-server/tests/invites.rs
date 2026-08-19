//! Invites (PROTOCOL §7): expiry, exhaustion, revocation, preview.

mod common;

use linger_core::wire::{ErrorCode, ErrorEnvelope, Invite, InvitePreview};

async fn make_invite(
    stoop: &common::TestStoop,
    token: &str,
    body: serde_json::Value,
) -> Invite {
    reqwest::Client::new()
        .post(stoop.url("/invites"))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn register_with(stoop: &common::TestStoop, code: &str, username: &str) -> reqwest::Response {
    reqwest::Client::new()
        .post(stoop.url("/auth/register"))
        .json(&serde_json::json!({
            "invite_code": code, "username": username,
            "display_name": username, "password": "a long enough password",
        }))
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn single_use_default_exhausts_after_one_join() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let invite = make_invite(&stoop, &host.access_token, serde_json::json!({})).await;
    assert_eq!(invite.max_uses, Some(1), "invites are single-use by default");

    assert_eq!(register_with(&stoop, &invite.code, "callie").await.status(), 200);

    let resp = register_with(&stoop, &invite.code, "dave").await;
    assert_eq!(resp.status(), 422);
    let env: ErrorEnvelope = resp.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::InviteInvalid);
}

#[tokio::test]
async fn multi_use_invite_counts_down() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let invite = make_invite(&stoop, &host.access_token, serde_json::json!({ "max_uses": 2 })).await;

    assert_eq!(register_with(&stoop, &invite.code, "callie").await.status(), 200);
    assert_eq!(register_with(&stoop, &invite.code, "dave").await.status(), 200);
    assert_eq!(register_with(&stoop, &invite.code, "jen").await.status(), 422);
}

#[tokio::test]
async fn expired_invite_says_expired() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    // 0 hours ⇒ expires_at = now: already dead by the time it's used.
    let invite =
        make_invite(&stoop, &host.access_token, serde_json::json!({ "expires_in_hours": 0 })).await;

    let resp = register_with(&stoop, &invite.code, "callie").await;
    assert_eq!(resp.status(), 422);
    let env: ErrorEnvelope = resp.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::InviteExpired);

    let preview: InvitePreview = reqwest::get(stoop.url(&format!("/auth/invite/{}", invite.code)))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!preview.valid);
}

#[tokio::test]
async fn revocation_kills_an_invite_and_permissions_hold() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let member = common::join_member(&stoop, &host.access_token, "callie").await;
    let client = reqwest::Client::new();

    // Member-created invite: a third member can't revoke it, the host can.
    let invite = make_invite(&stoop, &member.access_token, serde_json::json!({})).await;
    let other = common::join_member(&stoop, &host.access_token, "dave").await;

    let denied = client
        .delete(stoop.url(&format!("/invites/{}", invite.code)))
        .bearer_auth(&other.access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 403);

    let revoked = client
        .delete(stoop.url(&format!("/invites/{}", invite.code)))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(revoked.status(), 204);

    assert_eq!(register_with(&stoop, &invite.code, "jen").await.status(), 422);
}

#[tokio::test]
async fn preview_shows_stoop_name_for_valid_invites_only() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let invite = make_invite(&stoop, &host.access_token, serde_json::json!({})).await;

    let preview: InvitePreview = reqwest::get(stoop.url(&format!("/auth/invite/{}", invite.code)))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(preview.valid);
    assert_eq!(preview.stoop_name.as_deref(), Some("test stoop"));

    let missing: InvitePreview = reqwest::get(stoop.url("/auth/invite/nope00000000"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!missing.valid);
    assert!(missing.stoop_name.is_none());
}
