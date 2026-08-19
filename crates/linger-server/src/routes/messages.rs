//! Messages, reactions, and read markers (PROTOCOL §4).
//!
//! Deletes are tombstones — the row stays so reply chains survive. And per the
//! AGENTS.md hard rule: nothing here computes or returns an unread count, and
//! nothing ever will.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use linger_core::gateway::ServerEvent;
use linger_core::limits::RATE_MESSAGE_SEND;
use linger_core::wire::{CreateMessageRequest, EditMessageRequest, Message, UpdateReadMarkerRequest};
use linger_core::{MessageId, RoomId, UserId};
use serde::Deserialize;

use crate::auth::AuthedUser;
use crate::db::now_ms;
use crate::error::ApiError;
use crate::state::AppState;
use crate::{repo, validate};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/rooms/{id}/messages", get(page).post(create))
        .route("/rooms/{id}/read", put(put_read_marker))
        .route("/read", get(read_map))
        .route("/messages/{id}", axum::routing::patch(edit).delete(delete))
        .route("/messages/{id}/pin", post(pin).delete(unpin))
        .route(
            "/messages/{id}/reactions/{key}",
            put(add_reaction).delete(remove_reaction),
        )
}

#[derive(Deserialize)]
struct PageQuery {
    before: Option<MessageId>,
    after: Option<MessageId>,
    limit: Option<u32>,
}

async fn page(
    State(state): State<AppState>,
    _auth: AuthedUser,
    Path(room_id): Path<RoomId>,
    Query(query): Query<PageQuery>,
) -> Result<Json<Vec<Message>>, ApiError> {
    repo::rooms::expect(&state.db.read, room_id).await?;
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    repo::messages::page(&state.db.read, room_id, query.before, query.after, limit)
        .await
        .map(Json)
}

async fn create(
    State(state): State<AppState>,
    auth: AuthedUser,
    Path(room_id): Path<RoomId>,
    Json(req): Json<CreateMessageRequest>,
) -> Result<Json<Message>, ApiError> {
    if let Err(retry) = state.limiter.check(&format!("msg:{}", auth.id), RATE_MESSAGE_SEND) {
        return Err(ApiError::rate_limited(retry));
    }

    let room = repo::rooms::expect(&state.db.read, room_id).await?;
    if room.archived_at.is_some() {
        return Err(ApiError::validation("That room is archived."));
    }
    let body = validate::message_body(&req.body)?;
    if req.attachment_ids.as_ref().is_some_and(|ids| !ids.is_empty()) {
        return Err(ApiError::validation("Attachments arrive with M6."));
    }
    if let Some(reply_to) = req.reply_to {
        let parent = repo::messages::expect(&state.db.read, reply_to).await?;
        if parent.room_id != room_id {
            return Err(ApiError::validation("Replies stay in their room."));
        }
    }

    let id = MessageId::new();
    sqlx::query(
        "INSERT INTO messages (id, room_id, author_id, body, reply_to, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_vec())
    .bind(room_id.to_vec())
    .bind(auth.id.to_vec())
    .bind(&body)
    .bind(req.reply_to.map(|r| r.to_vec()))
    .bind(now_ms())
    .execute(&state.db.write)
    .await?;

    let message = repo::messages::expect(&state.db.read, id).await?;
    state.gateway.publish(ServerEvent::MessageCreate(message.clone()));
    Ok(Json(message))
}

async fn edit(
    State(state): State<AppState>,
    auth: AuthedUser,
    Path(id): Path<MessageId>,
    Json(req): Json<EditMessageRequest>,
) -> Result<Json<Message>, ApiError> {
    let message = repo::messages::expect(&state.db.read, id).await?;
    if message.deleted_at.is_some() {
        return Err(ApiError::not_found("That message is gone."));
    }
    if message.author_id != auth.id {
        return Err(ApiError::forbidden("Only the author can edit a message."));
    }
    let body = validate::message_body(&req.body)?;

    sqlx::query("UPDATE messages SET body = ?, edited_at = ? WHERE id = ?")
        .bind(&body)
        .bind(now_ms())
        .bind(id.to_vec())
        .execute(&state.db.write)
        .await?;

    let message = repo::messages::expect(&state.db.read, id).await?;
    state.gateway.publish(ServerEvent::MessageUpdate(message.clone()));
    Ok(Json(message))
}

async fn is_host(state: &AppState, user_id: UserId) -> Result<bool, ApiError> {
    Ok(sqlx::query_scalar("SELECT is_host FROM users WHERE id = ?")
        .bind(user_id.to_vec())
        .fetch_optional(&state.db.read)
        .await?
        .unwrap_or(false))
}

async fn delete(
    State(state): State<AppState>,
    auth: AuthedUser,
    Path(id): Path<MessageId>,
) -> Result<StatusCode, ApiError> {
    let message = repo::messages::expect(&state.db.read, id).await?;
    if message.deleted_at.is_some() {
        return Ok(StatusCode::NO_CONTENT); // idempotent
    }
    if message.author_id != auth.id && !is_host(&state, auth.id).await? {
        return Err(ApiError::forbidden("Only the author or the host can delete a message."));
    }

    // Tombstone, not removal: body empties, the row (and reply chains) survive.
    sqlx::query("UPDATE messages SET body = '', deleted_at = ?, pinned_at = NULL WHERE id = ?")
        .bind(now_ms())
        .bind(id.to_vec())
        .execute(&state.db.write)
        .await?;

    state
        .gateway
        .publish(ServerEvent::MessageDelete { id, room_id: message.room_id });
    Ok(StatusCode::NO_CONTENT)
}

async fn set_pin(
    state: &AppState,
    id: MessageId,
    pinned: bool,
) -> Result<Json<Message>, ApiError> {
    let message = repo::messages::expect(&state.db.read, id).await?;
    if message.deleted_at.is_some() {
        return Err(ApiError::not_found("That message is gone."));
    }
    sqlx::query("UPDATE messages SET pinned_at = ? WHERE id = ?")
        .bind(pinned.then(now_ms))
        .bind(id.to_vec())
        .execute(&state.db.write)
        .await?;
    let message = repo::messages::expect(&state.db.read, id).await?;
    state.gateway.publish(ServerEvent::MessageUpdate(message.clone()));
    Ok(Json(message))
}

async fn pin(
    State(state): State<AppState>,
    _auth: AuthedUser,
    Path(id): Path<MessageId>,
) -> Result<Json<Message>, ApiError> {
    set_pin(&state, id, true).await
}

async fn unpin(
    State(state): State<AppState>,
    _auth: AuthedUser,
    Path(id): Path<MessageId>,
) -> Result<Json<Message>, ApiError> {
    set_pin(&state, id, false).await
}

async fn add_reaction(
    State(state): State<AppState>,
    auth: AuthedUser,
    Path((id, key)): Path<(MessageId, String)>,
) -> Result<StatusCode, ApiError> {
    if !linger_core::is_valid_reaction_key(&key) {
        return Err(ApiError::validation("Reactions come from the fixed set of 12."));
    }
    let message = repo::messages::expect(&state.db.read, id).await?;
    if message.deleted_at.is_some() {
        return Err(ApiError::not_found("That message is gone."));
    }

    sqlx::query(
        "INSERT OR IGNORE INTO reactions (message_id, user_id, key, created_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(id.to_vec())
    .bind(auth.id.to_vec())
    .bind(&key)
    .bind(now_ms())
    .execute(&state.db.write)
    .await?;

    publish_reaction(&state, id, &key).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_reaction(
    State(state): State<AppState>,
    auth: AuthedUser,
    Path((id, key)): Path<(MessageId, String)>,
) -> Result<StatusCode, ApiError> {
    if !linger_core::is_valid_reaction_key(&key) {
        return Err(ApiError::validation("Reactions come from the fixed set of 12."));
    }
    sqlx::query("DELETE FROM reactions WHERE message_id = ? AND user_id = ? AND key = ?")
        .bind(id.to_vec())
        .bind(auth.id.to_vec())
        .bind(&key)
        .execute(&state.db.write)
        .await?;

    publish_reaction(&state, id, &key).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn publish_reaction(state: &AppState, id: MessageId, key: &str) -> Result<(), ApiError> {
    let group = repo::messages::reaction_group(&state.db.read, id, key).await?;
    state.gateway.publish(ServerEvent::ReactionUpdate {
        message_id: id,
        key: group.key,
        count: group.count,
        user_ids: group.user_ids,
    });
    Ok(())
}

async fn put_read_marker(
    State(state): State<AppState>,
    auth: AuthedUser,
    Path(room_id): Path<RoomId>,
    Json(req): Json<UpdateReadMarkerRequest>,
) -> Result<StatusCode, ApiError> {
    let message = repo::messages::expect(&state.db.read, req.last_read_id).await?;
    if message.room_id != room_id {
        return Err(ApiError::validation("That message isn't in that room."));
    }
    sqlx::query(
        "INSERT INTO read_markers (user_id, room_id, last_read_id, updated_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(user_id, room_id) DO UPDATE SET
           last_read_id = excluded.last_read_id, updated_at = excluded.updated_at",
    )
    .bind(auth.id.to_vec())
    .bind(room_id.to_vec())
    .bind(req.last_read_id.to_vec())
    .bind(now_ms())
    .execute(&state.db.write)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn read_map(
    State(state): State<AppState>,
    auth: AuthedUser,
) -> Result<Json<linger_core::wire::ReadMap>, ApiError> {
    let rows: Vec<(Vec<u8>, Vec<u8>)> =
        sqlx::query_as("SELECT room_id, last_read_id FROM read_markers WHERE user_id = ?")
            .bind(auth.id.to_vec())
            .fetch_all(&state.db.read)
            .await?;
    let map = rows
        .into_iter()
        .map(|(room, msg)| {
            Ok((
                RoomId::from_slice(&room).map_err(anyhow::Error::from)?,
                MessageId::from_slice(&msg).map_err(anyhow::Error::from)?,
            ))
        })
        .collect::<Result<linger_core::wire::ReadMap, ApiError>>()?;
    Ok(Json(map))
}
