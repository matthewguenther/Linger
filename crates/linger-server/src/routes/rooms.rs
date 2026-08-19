//! Rooms (PROTOCOL §3). Creation and reshaping are host-only; sitting in them
//! is everyone's business (and the gateway's).

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use linger_core::gateway::ServerEvent;
use linger_core::wire::{CreateRoomRequest, Room, UpdateRoomRequest};
use linger_core::RoomId;

use crate::auth::{AuthedUser, HostUser};
use crate::db::now_ms;
use crate::error::ApiError;
use crate::state::AppState;
use crate::{repo, validate};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/rooms", get(list).post(create))
        .route("/rooms/{id}", axum::routing::patch(update))
        .route("/rooms/{id}/archive", post(archive))
}

async fn list(State(state): State<AppState>, _auth: AuthedUser) -> Result<Json<Vec<Room>>, ApiError> {
    repo::rooms::all(&state.db.read).await.map(Json)
}

async fn create(
    State(state): State<AppState>,
    _host: HostUser,
    Json(req): Json<CreateRoomRequest>,
) -> Result<Json<Room>, ApiError> {
    validate::room_slug(&req.slug)?;
    let name = req.name.trim();
    if name.is_empty() || name.chars().count() > 48 {
        return Err(ApiError::validation("Room names are 1–48 characters."));
    }

    let id = RoomId::new();
    let position: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(position), -1) + 1 FROM rooms")
            .fetch_one(&state.db.read)
            .await?;

    let inserted = sqlx::query(
        "INSERT INTO rooms (id, slug, name, topic, position, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_vec())
    .bind(&req.slug)
    .bind(name)
    .bind(&req.topic)
    .bind(position)
    .bind(now_ms())
    .execute(&state.db.write)
    .await;

    match inserted {
        Ok(_) => {}
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            return Err(ApiError::conflict("A room with that slug already exists."));
        }
        Err(e) => return Err(e.into()),
    }

    let room = repo::rooms::expect(&state.db.read, id).await?;
    state.gateway.publish(ServerEvent::RoomCreate(room.clone()));
    Ok(Json(room))
}

async fn update(
    State(state): State<AppState>,
    _host: HostUser,
    Path(id): Path<RoomId>,
    Json(req): Json<UpdateRoomRequest>,
) -> Result<Json<Room>, ApiError> {
    repo::rooms::expect(&state.db.read, id).await?;
    if let Some(name) = &req.name {
        let trimmed = name.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 48 {
            return Err(ApiError::validation("Room names are 1–48 characters."));
        }
    }

    let mut tx = state.db.write.begin().await.map_err(ApiError::from)?;
    if let Some(name) = &req.name {
        sqlx::query("UPDATE rooms SET name = ? WHERE id = ?")
            .bind(name.trim())
            .bind(id.to_vec())
            .execute(&mut *tx)
            .await?;
    }
    if let Some(topic) = &req.topic {
        sqlx::query("UPDATE rooms SET topic = ? WHERE id = ?")
            .bind(topic)
            .bind(id.to_vec())
            .execute(&mut *tx)
            .await?;
    }
    if let Some(position) = req.position {
        sqlx::query("UPDATE rooms SET position = ? WHERE id = ?")
            .bind(i64::from(position))
            .bind(id.to_vec())
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await.map_err(ApiError::from)?;

    let room = repo::rooms::expect(&state.db.read, id).await?;
    state.gateway.publish(ServerEvent::RoomUpdate(room.clone()));
    Ok(Json(room))
}

async fn archive(
    State(state): State<AppState>,
    _host: HostUser,
    Path(id): Path<RoomId>,
) -> Result<Json<Room>, ApiError> {
    repo::rooms::expect(&state.db.read, id).await?;
    sqlx::query("UPDATE rooms SET archived_at = ? WHERE id = ? AND archived_at IS NULL")
        .bind(now_ms())
        .bind(id.to_vec())
        .execute(&state.db.write)
        .await?;
    let room = repo::rooms::expect(&state.db.read, id).await?;
    state.gateway.publish(ServerEvent::RoomUpdate(room.clone()));
    Ok(Json(room))
}
