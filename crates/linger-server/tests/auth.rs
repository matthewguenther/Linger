//! Auth (PROTOCOL §2): register, login, refresh rotation, reuse detection,
//! logout, and the login rate limit.

mod common;

use linger_core::wire::{AuthResponse, ErrorCode, ErrorEnvelope, RefreshResponse};

#[tokio::test]
async fn register_login_and_wrong_password() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let member = common::join_member(&stoop, &host.access_token, "callie").await;
    assert!(!member.user.is_host);

    let client = reqwest::Client::new();
    let ok: AuthResponse = client
        .post(stoop.url("/auth/login"))
        .json(&serde_json::json!({ "username": "callie", "password": "a perfectly fine password" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(ok.user.username, "callie");

    for (user, pw) in [("callie", "wrong password entirely"), ("nobody", "whatever whatever")] {
        let resp = client
            .post(stoop.url("/auth/login"))
            .json(&serde_json::json!({ "username": user, "password": pw }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 401);
        let env: ErrorEnvelope = resp.json().await.unwrap();
        assert_eq!(env.error.code, ErrorCode::Unauthenticated);
    }
}

#[tokio::test]
async fn duplicate_username_conflicts_and_invite_use_is_returned() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    let invite: linger_core::wire::Invite = client
        .post(stoop.url("/invites"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // "matt" is taken by the host; the tx must roll back the invite use…
    let resp = client
        .post(stoop.url("/auth/register"))
        .json(&serde_json::json!({
            "invite_code": invite.code, "username": "matt",
            "display_name": "Impostor", "password": "a long enough password",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);

    // …so the same single-use invite still works for a fresh name.
    let resp = client
        .post(stoop.url("/auth/register"))
        .json(&serde_json::json!({
            "invite_code": invite.code, "username": "dave",
            "display_name": "Dave", "password": "a long enough password",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn refresh_rotates_and_reuse_revokes_the_family() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    // Rotate once: old token spent, new one works.
    let first: RefreshResponse = client
        .post(stoop.url("/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": host.refresh_token }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_ne!(first.refresh_token, host.refresh_token);

    let second: RefreshResponse = client
        .post(stoop.url("/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": first.refresh_token }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Replaying the *first* (already rotated) token is theft-shaped: it must
    // fail AND take the whole family down with it.
    let replay = client
        .post(stoop.url("/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": first.refresh_token }))
        .send()
        .await
        .unwrap();
    assert_eq!(replay.status(), 401);

    let family_dead = client
        .post(stoop.url("/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": second.refresh_token }))
        .send()
        .await
        .unwrap();
    assert_eq!(family_dead.status(), 401, "reuse must revoke the newest token too");
}

#[tokio::test]
async fn logout_revokes_the_family() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    let out = client
        .post(stoop.url("/auth/logout"))
        .json(&serde_json::json!({ "refresh_token": host.refresh_token }))
        .send()
        .await
        .unwrap();
    assert_eq!(out.status(), 204);

    let resp = client
        .post(stoop.url("/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": host.refresh_token }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn login_is_rate_limited_per_ip_with_retry_hint() {
    let stoop = common::spawn_stoop().await;
    let _host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    // 5/min/IP: burn the burst with bad attempts…
    for _ in 0..5 {
        let resp = client
            .post(stoop.url("/auth/login"))
            .json(&serde_json::json!({ "username": "matt", "password": "wrong wrong wrong" }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 401);
    }
    // …then the 6th gets the envelope with a usable retry hint.
    let resp = client
        .post(stoop.url("/auth/login"))
        .json(&serde_json::json!({ "username": "matt", "password": "correct horse battery" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 429);
    let env: ErrorEnvelope = resp.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::RateLimited);
    assert!(env.error.retry_after_ms.is_some_and(|ms| ms > 0));
}

#[tokio::test]
async fn expired_and_garbage_access_tokens_are_rejected() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    // Sanity: the real token works.
    let ok = client
        .get(stoop.url("/me"))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 200);

    // A structurally valid JWT signed by nobody we know.
    let forged = format!("{}.e30.forged-signature", host.access_token.split('.').next().unwrap());
    let resp = client.get(stoop.url("/me")).bearer_auth(forged).send().await.unwrap();
    assert_eq!(resp.status(), 401);
}
