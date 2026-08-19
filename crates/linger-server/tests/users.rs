//! Users, styling, signs (PROTOCOL §5). The heart of it: every closed-set key
//! is validated server-side — the AGENTS.md hard rule.

mod common;

use linger_core::wire::User;

#[tokio::test]
async fn patch_me_round_trips_style_and_sign() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    let updated: User = client
        .patch(stoop.url("/me"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({
            "display_name": "Matt 💾",
            "style": {
                "font_key": "departure-mono",
                "weight": 700,
                "italic": false,
                "fill": { "kind": "gradient", "from": "teal", "to": "violet" },
                "effect": "shimmer",
                "msg_font_key": "newsreader"
            },
            "sign": {
                "line": "mounting the drive",
                "reading": null, "listening": "Bill Evans", "working_on": "the shelf",
                "image_key": null, "away_message": null, "away_since": null
            },
            "entrance_sound": "screen-door"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(updated.display_name, "Matt 💾");
    assert_eq!(updated.style.font_key, "departure-mono");
    assert_eq!(updated.style.weight, 700);
    assert_eq!(updated.entrance_sound.as_deref(), Some("screen-door"));
    let sign = updated.sign.expect("sign saved");
    assert_eq!(sign.listening.as_deref(), Some("Bill Evans"));

    // Everyone else sees the styled name through GET /users.
    let users: Vec<User> = client
        .get(stoop.url("/users"))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(users[0].style.font_key, "departure-mono");

    // Clearing the entrance sound with "".
    let cleared: User = client
        .patch(stoop.url("/me"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "entrance_sound": "" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(cleared.entrance_sound.is_none());
}

#[tokio::test]
async fn server_rejects_every_off_list_key() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    let base_style = serde_json::json!({
        "font_key": "geist-sans", "weight": 500, "italic": false,
        "fill": { "kind": "solid", "color": "slate" },
        "effect": "none", "msg_font_key": null
    });

    let mut cases: Vec<serde_json::Value> = Vec::new();
    let mut with = |patch: serde_json::Value| {
        let mut style = base_style.clone();
        style
            .as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        cases.push(serde_json::json!({ "style": style }));
    };
    with(serde_json::json!({ "font_key": "comic-sans" }));
    with(serde_json::json!({ "msg_font_key": "papyrus" }));
    with(serde_json::json!({ "weight": 600 }));
    with(serde_json::json!({ "fill": { "kind": "solid", "color": "#ff00ff" } }));
    with(serde_json::json!({ "fill": { "kind": "gradient", "from": "teal", "to": "hotpink" } }));
    cases.push(serde_json::json!({ "entrance_sound": "airhorn" }));
    cases.push(serde_json::json!({ "sign": { "line": "x".repeat(241) } }));
    cases.push(serde_json::json!({ "display_name": "" }));

    for case in cases {
        let resp = client
            .patch(stoop.url("/me"))
            .bearer_auth(&host.access_token)
            .json(&case)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 422, "should reject {case}");
    }
}

#[tokio::test]
async fn away_message_stamps_away_since_server_side() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    // Client-sent away_since is ignored; the server stamps it.
    let user: User = client
        .patch(stoop.url("/me"))
        .bearer_auth(&host.access_token)
        .json(
            &serde_json::json!({ "sign": { "away_message": "back after work", "away_since": 1 } }),
        )
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let since = user
        .sign
        .unwrap()
        .away_since
        .expect("server stamps away_since");
    assert!(
        since > 1_600_000_000_000,
        "must be a real timestamp, not the client's value"
    );

    // Same message ⇒ stamp is stable; clearing ⇒ stamp clears.
    let user: User = client
        .patch(stoop.url("/me"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "sign": { "away_message": "back after work" } }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(user.sign.unwrap().away_since, Some(since));

    let user: User = client
        .patch(stoop.url("/me"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "sign": { "away_message": null } }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(user.sign.unwrap().away_since.is_none());
}

#[tokio::test]
async fn password_change_verifies_current_and_revokes_refresh_tokens() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    let wrong = client
        .patch(stoop.url("/me/password"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({
            "current_password": "not it", "new_password": "a brand new passphrase"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(wrong.status(), 403);

    let ok = client
        .patch(stoop.url("/me/password"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({
            "current_password": "correct horse battery", "new_password": "a brand new passphrase"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 204);

    // Old refresh token is dead; new password logs in.
    let refresh = client
        .post(stoop.url("/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": host.refresh_token }))
        .send()
        .await
        .unwrap();
    assert_eq!(refresh.status(), 401);

    let login = client
        .post(stoop.url("/auth/login"))
        .json(&serde_json::json!({ "username": "matt", "password": "a brand new passphrase" }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
}

#[tokio::test]
async fn notify_rules_upsert_and_delete_including_all_rooms() {
    let (stoop, host, room) = common::stoop_with_room("garage").await;
    let member = common::join_member(&stoop, &host.access_token, "callie").await;
    let client = reqwest::Client::new();

    // All-rooms rule (room_id null), twice — must not duplicate.
    for _ in 0..2 {
        let resp = client
            .put(stoop.url("/me/notify-rules"))
            .bearer_auth(&host.access_token)
            .json(&serde_json::json!({ "target_user_id": member.user.id, "room_id": null }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 204);
    }
    // Plus a per-room rule.
    let resp = client
        .put(stoop.url("/me/notify-rules"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "target_user_id": member.user.id, "room_id": room.id }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    let rules: Vec<serde_json::Value> = client
        .get(stoop.url("/me/notify-rules"))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rules.len(), 2);

    let resp = client
        .delete(stoop.url("/me/notify-rules"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "target_user_id": member.user.id, "room_id": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    let rules: Vec<serde_json::Value> = client
        .get(stoop.url("/me/notify-rules"))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rules.len(), 1);
}
