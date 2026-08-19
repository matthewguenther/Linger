//! The stoop itself (PROTOCOL §3): its name, accent, and headline numbers.

use std::collections::HashMap;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use linger_core::wire::{ColorKey, StoopInfo, UpdateStoopRequest};

use crate::auth::{AuthedUser, HostUser};
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/stoop", get(info).patch(update))
}

async fn read_config(state: &AppState) -> Result<HashMap<String, String>, ApiError> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM stoop_config")
        .fetch_all(&state.db.read)
        .await?;
    Ok(rows.into_iter().collect())
}

async fn build_info(state: &AppState) -> Result<StoopInfo, ApiError> {
    let config = read_config(state).await?;
    let (member_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE deactivated_at IS NULL")
            .fetch_one(&state.db.read)
            .await?;
    Ok(StoopInfo {
        name: config.get("name").cloned().unwrap_or_default(),
        accent_key: config.get("accent_key").cloned().map(ColorKey),
        icon_key: config.get("icon_key").cloned(),
        member_count: u32::try_from(member_count).unwrap_or(0),
        created_at: config
            .get("created_at")
            .and_then(|v| v.parse().ok())
            .unwrap_or_default(),
    })
}

async fn info(
    State(state): State<AppState>,
    _auth: AuthedUser,
) -> Result<Json<StoopInfo>, ApiError> {
    build_info(&state).await.map(Json)
}

async fn update(
    State(state): State<AppState>,
    _host: HostUser,
    Json(req): Json<UpdateStoopRequest>,
) -> Result<Json<StoopInfo>, ApiError> {
    if let Some(name) = &req.name {
        let trimmed = name.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 48 {
            return Err(ApiError::validation("Stoop names are 1–48 characters."));
        }
    }
    if let Some(accent) = &req.accent_key {
        if !accent.is_valid() {
            return Err(ApiError::validation(
                "The accent comes from the named palette.",
            ));
        }
    }

    let mut tx = state.db.write.begin().await.map_err(ApiError::from)?;
    let pairs = [
        ("name", req.name.map(|n| n.trim().to_string())),
        ("accent_key", req.accent_key.map(|c| c.0)),
        ("icon_key", req.icon_key),
    ];
    for (key, value) in pairs {
        if let Some(value) = value {
            sqlx::query(
                "INSERT INTO stoop_config (key, value) VALUES (?, ?)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )
            .bind(key)
            .bind(value)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await.map_err(ApiError::from)?;

    build_info(&state).await.map(Json)
}
