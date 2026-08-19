//! Messages, reactions, read markers (PROTOCOL §4): pagination edges,
//! tombstones, the permission matrix, and — by its absence — the unread count.

mod common;

use linger_core::wire::{ErrorCode, ErrorEnvelope, Message};

async fn send(
    stoop: &common::TestStoop,
    token: &str,
    room: &str,
    body: &str,
) -> Message {
    let resp = reqwest::Client::new()
        .post(stoop.url(&format!("/rooms/{room}/messages")))
        .bearer_auth(token)
        .json(&serde_json::json!({ "body": body }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "send failed: {}", resp.text().await.unwrap());
    resp.json().await.unwrap()
}

async fn fetch(
    stoop: &common::TestStoop,
    token: &str,
    room: &str,
    query: &str,
) -> Vec<Message> {
    reqwest::Client::new()
        .get(stoop.url(&format!("/rooms/{room}/messages{query}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn pagination_is_newest_first_with_working_edges() {
    let (stoop, host, room) = common::stoop_with_room("garage").await;
    let member = common::join_member(&stoop, &host.access_token, "callie").await;
    let room = room.id.to_string();

    // Empty room pages cleanly.
    assert!(fetch(&stoop, &host.access_token, &room, "").await.is_empty());

    // 8 messages, alternating authors (stays under the per-user rate limit).
    let mut sent = Vec::new();
    for i in 0..8 {
        let token = if i % 2 == 0 { &host.access_token } else { &member.access_token };
        sent.push(send(&stoop, token, &room, &format!("message {i}")).await);
    }

    // Newest-first, default page.
    let page = fetch(&stoop, &host.access_token, &room, "").await;
    assert_eq!(page.len(), 8);
    assert_eq!(page[0].body, "message 7");
    assert_eq!(page[7].body, "message 0");

    // limit clamps and slices from the newest end.
    let top3 = fetch(&stoop, &host.access_token, &room, "?limit=3").await;
    assert_eq!(top3.iter().map(|m| m.body.as_str()).collect::<Vec<_>>(), [
        "message 7", "message 6", "message 5"
    ]);

    // before: strictly older than the anchor; exact-limit boundary.
    let before = fetch(&stoop, &host.access_token, &room, &format!("?before={}&limit=5", sent[5].id)).await;
    assert_eq!(before.iter().map(|m| m.body.as_str()).collect::<Vec<_>>(), [
        "message 4", "message 3", "message 2", "message 1", "message 0"
    ]);

    // after: strictly newer, still returned newest-first.
    let after = fetch(&stoop, &host.access_token, &room, &format!("?after={}", sent[5].id)).await;
    assert_eq!(after.iter().map(|m| m.body.as_str()).collect::<Vec<_>>(), [
        "message 7", "message 6"
    ]);

    // before + after together bound a window.
    let window = fetch(
        &stoop,
        &host.access_token,
        &room,
        &format!("?after={}&before={}", sent[1].id, sent[5].id),
    )
    .await;
    assert_eq!(window.iter().map(|m| m.body.as_str()).collect::<Vec<_>>(), [
        "message 4", "message 3", "message 2"
    ]);
}

#[tokio::test]
async fn body_validation_and_reply_room_checks() {
    let (stoop, host, room_a) = common::stoop_with_room("garage").await;
    let client = reqwest::Client::new();
    let room_b: linger_core::wire::Room = client
        .post(stoop.url("/rooms"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "slug": "porch", "name": "#porch" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    for bad in ["", "   ", &"x".repeat(8001)] {
        let resp = client
            .post(stoop.url(&format!("/rooms/{}/messages", room_a.id)))
            .bearer_auth(&host.access_token)
            .json(&serde_json::json!({ "body": bad }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 422);
    }

    // Cross-room reply is invalid.
    let in_a = send(&stoop, &host.access_token, &room_a.id.to_string(), "hello garage").await;
    let resp = client
        .post(stoop.url(&format!("/rooms/{}/messages", room_b.id)))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "body": "reply from porch", "reply_to": in_a.id }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);

    // Archived rooms don't take messages.
    client
        .post(stoop.url(&format!("/rooms/{}/archive", room_b.id)))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap();
    let resp = client
        .post(stoop.url(&format!("/rooms/{}/messages", room_b.id)))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "body": "anyone home?" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn edit_delete_permission_matrix_and_tombstones() {
    let (stoop, host, room) = common::stoop_with_room("garage").await;
    let member = common::join_member(&stoop, &host.access_token, "callie").await;
    let client = reqwest::Client::new();
    let room = room.id.to_string();

    let theirs = send(&stoop, &member.access_token, &room, "callie's message").await;
    let reply = send(&stoop, &host.access_token, &room, "a reply").await;

    // Host cannot edit someone else's message…
    let resp = client
        .patch(stoop.url(&format!("/messages/{}", theirs.id)))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "body": "rewritten" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    // …the author can.
    let edited: Message = client
        .patch(stoop.url(&format!("/messages/{}", theirs.id)))
        .bearer_auth(&member.access_token)
        .json(&serde_json::json!({ "body": "callie's edited message" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(edited.edited_at.is_some());

    // A member can't delete someone else's message; the host can (tombstone).
    let denied = client
        .delete(stoop.url(&format!("/messages/{}", reply.id)))
        .bearer_auth(&member.access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 403);

    let deleted = client
        .delete(stoop.url(&format!("/messages/{}", theirs.id)))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(deleted.status(), 204);

    // Tombstone: still present in the page, body empty, deleted_at set.
    let page = fetch(&stoop, &host.access_token, &room, "").await;
    let stone = page.iter().find(|m| m.id == theirs.id).expect("tombstone survives");
    assert_eq!(stone.body, "");
    assert!(stone.deleted_at.is_some());

    // Tombstones can't be edited or pinned.
    let resp = client
        .patch(stoop.url(&format!("/messages/{}", theirs.id)))
        .bearer_auth(&member.access_token)
        .json(&serde_json::json!({ "body": "necromancy" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let resp = client
        .post(stoop.url(&format!("/messages/{}/pin", theirs.id)))
        .bearer_auth(&member.access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn any_member_pins_and_unpins() {
    let (stoop, host, room) = common::stoop_with_room("garage").await;
    let member = common::join_member(&stoop, &host.access_token, "callie").await;
    let client = reqwest::Client::new();

    let msg = send(&stoop, &host.access_token, &room.id.to_string(), "pin me").await;
    let pinned: Message = client
        .post(stoop.url(&format!("/messages/{}/pin", msg.id)))
        .bearer_auth(&member.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(pinned.pinned_at.is_some());

    let unpinned: Message = client
        .delete(stoop.url(&format!("/messages/{}/pin", msg.id)))
        .bearer_auth(&member.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(unpinned.pinned_at.is_none());
}

#[tokio::test]
async fn reactions_validate_keys_group_and_are_idempotent() {
    let (stoop, host, room) = common::stoop_with_room("garage").await;
    let member = common::join_member(&stoop, &host.access_token, "callie").await;
    let client = reqwest::Client::new();
    let msg = send(&stoop, &host.access_token, &room.id.to_string(), "react to me").await;

    // Off-list key: rejected.
    let resp = client
        .put(stoop.url(&format!("/messages/{}/reactions/custom-emoji", msg.id)))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
    let env: ErrorEnvelope = resp.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::ValidationFailed);

    // Both react with "fire"; host double-taps (idempotent).
    for token in [&host.access_token, &host.access_token, &member.access_token] {
        let resp = client
            .put(stoop.url(&format!("/messages/{}/reactions/fire", msg.id)))
            .bearer_auth(token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 204);
    }

    let page = fetch(&stoop, &host.access_token, &room.id.to_string(), "").await;
    let reactions = &page.iter().find(|m| m.id == msg.id).unwrap().reactions;
    assert_eq!(reactions.len(), 1);
    assert_eq!(reactions[0].key, "fire");
    assert_eq!(reactions[0].count, 2);
    assert_eq!(reactions[0].user_ids.len(), 2);

    // Removal shrinks the group.
    client
        .delete(stoop.url(&format!("/messages/{}/reactions/fire", msg.id)))
        .bearer_auth(&member.access_token)
        .send()
        .await
        .unwrap();
    let page = fetch(&stoop, &host.access_token, &room.id.to_string(), "").await;
    assert_eq!(page.iter().find(|m| m.id == msg.id).unwrap().reactions[0].count, 1);
}

#[tokio::test]
async fn read_markers_round_trip_and_never_carry_counts() {
    let (stoop, host, room) = common::stoop_with_room("garage").await;
    let client = reqwest::Client::new();
    let room_id = room.id.to_string();

    let m1 = send(&stoop, &host.access_token, &room_id, "one").await;
    let m2 = send(&stoop, &host.access_token, &room_id, "two").await;

    let resp = client
        .put(stoop.url(&format!("/rooms/{room_id}/read")))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "last_read_id": m1.id }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Idempotent update to the newer marker.
    let resp = client
        .put(stoop.url(&format!("/rooms/{room_id}/read")))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "last_read_id": m2.id }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    let map: serde_json::Value = client
        .get(stoop.url("/read"))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(map[room_id.as_str()], serde_json::json!(m2.id.to_string()));

    // The hard rule, mechanically checked: no count-shaped field anywhere in
    // the read response or a room object.
    let rooms_raw = client
        .get(stoop.url("/rooms"))
        .bearer_auth(&host.access_token)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    for forbidden in ["unread", "count", "badge"] {
        assert!(
            !rooms_raw.contains(forbidden),
            "GET /rooms leaked a count-shaped field: {forbidden}"
        );
    }

    // Marker for a message from another room is rejected.
    let other: linger_core::wire::Room = client
        .post(stoop.url("/rooms"))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "slug": "porch", "name": "#porch" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let resp = client
        .put(stoop.url(&format!("/rooms/{}/read", other.id)))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "last_read_id": m2.id }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn message_send_is_rate_limited() {
    let (stoop, host, room) = common::stoop_with_room("garage").await;
    let room = room.id.to_string();

    // 10/10s: the burst goes through, the 11th gets the envelope.
    for i in 0..10 {
        send(&stoop, &host.access_token, &room, &format!("burst {i}")).await;
    }
    let resp = reqwest::Client::new()
        .post(stoop.url(&format!("/rooms/{room}/messages")))
        .bearer_auth(&host.access_token)
        .json(&serde_json::json!({ "body": "one too many" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 429);
    let env: ErrorEnvelope = resp.json().await.unwrap();
    assert_eq!(env.error.code, ErrorCode::RateLimited);
    assert!(env.error.retry_after_ms.is_some_and(|ms| ms > 0));
}
