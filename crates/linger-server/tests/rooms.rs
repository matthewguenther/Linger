//! Stoop + rooms (PROTOCOL §3): host-only mutations, slug validation,
//! palette-key validation, ordering.

mod common;

use linger_core::wire::{ErrorCode, ErrorEnvelope, Room, StoopInfo};

#[tokio::test]
async fn host_creates_rooms_members_do_not() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let member = common::join_member(&stoop, &host.access_token, "callie").await;
    let client = reqwest::Client::new();

    let created: Room = client
        .post(stoop.url("/rooms"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "slug": "garage", "name": "#garage", "topic": "projects" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(created.slug, "garage");
    assert_eq!(created.position, 0);
    assert!(created.last_message_id.is_none());

    let denied = client
        .post(stoop.url("/rooms"))
        .bearer_auth(&member.access_token)
        .json(&serde_json::json!({ "slug": "shed", "name": "#shed" }))
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 403);
    let env: ErrorEnvelope = denied.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::Forbidden);

    // Members can still see the rooms.
    let rooms: Vec<Room> = client
        .get(stoop.url("/rooms"))
        .bearer_auth(&member.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rooms.len(), 1);
}

#[tokio::test]
async fn slug_rules_and_uniqueness() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    for bad_slug in ["", "Garage", "the garage", "way-too-long-for-a-slug-way-too-long"] {
        let resp = client
            .post(stoop.url("/rooms"))
            .bearer_auth(&host.access_token)
            .json(&serde_json::json!({ "slug": bad_slug, "name": "x" }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 422, "slug {bad_slug:?} should be rejected");
    }

    let ok = client
        .post(stoop.url("/rooms"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "slug": "porch", "name": "#porch" }))
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 200);

    let dup = client
        .post(stoop.url("/rooms"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "slug": "porch", "name": "#porch again" }))
        .send()
        .await
        .unwrap();
    assert_eq!(dup.status(), 409);
}

#[tokio::test]
async fn patch_and_archive() {
    let (stoop, host, room) = common::stoop_with_room("garage").await;
    let client = reqwest::Client::new();

    let updated: Room = client
        .patch(stoop.url(&format!("/rooms/{}", room.id)))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "topic": "drives and drivers", "position": 3 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(updated.topic.as_deref(), Some("drives and drivers"));
    assert_eq!(updated.position, 3);

    let archived: Room = client
        .post(stoop.url(&format!("/rooms/{}/archive", room.id)))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(archived.archived_at.is_some());
}

#[tokio::test]
async fn stoop_patch_validates_the_palette_key() {
    let stoop = common::spawn_stoop().await;
    let host = common::bootstrap_host(&stoop).await;
    let client = reqwest::Client::new();

    // Hex is exactly what the palette rule forbids on the wire.
    let bad = client
        .patch(stoop.url("/stoop"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "accent_key": "#6E9BFF" }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 422);

    let good: StoopInfo = client
        .patch(stoop.url("/stoop"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "accent_key": "azure", "name": "the garage" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(good.accent_key.map(|c| c.0), Some("azure".to_string()));
    assert_eq!(good.name, "the garage");
}
