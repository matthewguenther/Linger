//! Users, styling, signs, notify rules (PROTOCOL §5).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::{Json, Router};
use linger_core::gateway::ServerEvent;
use linger_core::wire::{
    ChangePasswordRequest, Fill, NotifyRule, UpdateMeRequest, User,
};
use linger_core::UserId;

use crate::auth::{self, AuthedUser};
use crate::db::now_ms;
use crate::error::ApiError;
use crate::state::AppState;
use crate::{repo, validate};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/{id}", get(get_user))
        .route("/me", get(me).patch(patch_me))
        .route("/me/password", patch(change_password))
        .route(
            "/me/notify-rules",
            get(list_notify_rules).put(put_notify_rule).delete(delete_notify_rule),
        )
}

async fn list_users(
    State(state): State<AppState>,
    _auth: AuthedUser,
) -> Result<Json<Vec<User>>, ApiError> {
    repo::users::all(&state.db.read).await.map(Json)
}

async fn get_user(
    State(state): State<AppState>,
    _auth: AuthedUser,
    Path(id): Path<UserId>,
) -> Result<Json<User>, ApiError> {
    repo::users::expect(&state.db.read, id).await.map(Json)
}

async fn me(State(state): State<AppState>, auth: AuthedUser) -> Result<Json<User>, ApiError> {
    repo::users::expect(&state.db.read, auth.id).await.map(Json)
}

async fn patch_me(
    State(state): State<AppState>,
    auth: AuthedUser,
    Json(req): Json<UpdateMeRequest>,
) -> Result<Json<User>, ApiError> {
    // Validate everything before writing anything — a PATCH is all-or-nothing.
    if let Some(name) = &req.display_name {
        validate::display_name(name)?;
    }
    if let Some(style) = &req.style {
        validate::style(style)?;
    }
    if let Some(sign) = &req.sign {
        validate::sign(sign)?;
    }
    if let Some(sound) = &req.entrance_sound {
        if !sound.is_empty() && !linger_core::is_valid_entrance_sound_key(sound) {
            return Err(ApiError::validation("That entrance sound isn't in the bundled set."));
        }
    }

    let mut tx = state.db.write.begin().await.map_err(ApiError::from)?;

    if let Some(name) = &req.display_name {
        sqlx::query("UPDATE users SET display_name = ? WHERE id = ?")
            .bind(name.trim())
            .bind(auth.id.to_vec())
            .execute(&mut *tx)
            .await?;
    }

    if let Some(style) = &req.style {
        let (fill_kind, fill_from, fill_to) = match &style.fill {
            Fill::Solid { color } => ("solid", color.0.clone(), None),
            Fill::Gradient { from, to } => ("gradient", from.0.clone(), Some(to.0.clone())),
        };
        let effect = match style.effect {
            linger_core::wire::NameEffect::None => "none",
            linger_core::wire::NameEffect::Shimmer => "shimmer",
            linger_core::wire::NameEffect::Glow => "glow",
        };
        sqlx::query(
            "INSERT INTO user_style
               (user_id, font_key, weight, italic, fill_kind, fill_from, fill_to, effect, msg_font_key)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(user_id) DO UPDATE SET
               font_key = excluded.font_key, weight = excluded.weight,
               italic = excluded.italic, fill_kind = excluded.fill_kind,
               fill_from = excluded.fill_from, fill_to = excluded.fill_to,
               effect = excluded.effect, msg_font_key = excluded.msg_font_key",
        )
        .bind(auth.id.to_vec())
        .bind(&style.font_key)
        .bind(i64::from(style.weight))
        .bind(i64::from(style.italic))
        .bind(fill_kind)
        .bind(&fill_from)
        .bind(&fill_to)
        .bind(effect)
        .bind(&style.msg_font_key)
        .execute(&mut *tx)
        .await?;
    }

    if let Some(sign) = &req.sign {
        // `away_since` is server-owned: stamped when an away message appears or
        // changes, cleared with it.
        let prev_away: Option<(Option<String>, Option<i64>)> =
            sqlx::query_as("SELECT away_message, away_since FROM user_sign WHERE user_id = ?")
                .bind(auth.id.to_vec())
                .fetch_optional(&mut *tx)
                .await?;
        let away_since = match (&sign.away_message, prev_away) {
            (None, _) => None,
            (Some(new), Some((Some(old), Some(since)))) if new == &old => Some(since),
            (Some(_), _) => Some(now_ms()),
        };
        sqlx::query(
            "INSERT INTO user_sign
               (user_id, line, reading, listening, working_on, image_key,
                away_message, away_since, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(user_id) DO UPDATE SET
               line = excluded.line, reading = excluded.reading,
               listening = excluded.listening, working_on = excluded.working_on,
               image_key = excluded.image_key, away_message = excluded.away_message,
               away_since = excluded.away_since, updated_at = excluded.updated_at",
        )
        .bind(auth.id.to_vec())
        .bind(&sign.line)
        .bind(&sign.reading)
        .bind(&sign.listening)
        .bind(&sign.working_on)
        .bind(&sign.image_key)
        .bind(&sign.away_message)
        .bind(away_since)
        .bind(now_ms())
        .execute(&mut *tx)
        .await?;
    }

    if let Some(sound) = &req.entrance_sound {
        if sound.is_empty() {
            sqlx::query("DELETE FROM entrance_sounds WHERE user_id = ?")
                .bind(auth.id.to_vec())
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query(
                "INSERT INTO entrance_sounds (user_id, sound_key) VALUES (?, ?)
                 ON CONFLICT(user_id) DO UPDATE SET sound_key = excluded.sound_key",
            )
            .bind(auth.id.to_vec())
            .bind(sound)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await.map_err(ApiError::from)?;

    let user = repo::users::expect(&state.db.read, auth.id).await?;
    state.gateway.publish(ServerEvent::UserUpdate(user.clone()));
    Ok(Json(user))
}

async fn change_password(
    State(state): State<AppState>,
    auth: AuthedUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<StatusCode, ApiError> {
    validate::password(&req.new_password)?;

    let hash: Option<String> = sqlx::query_scalar(
        "SELECT password_hash FROM users WHERE id = ? AND deactivated_at IS NULL",
    )
    .bind(auth.id.to_vec())
    .fetch_optional(&state.db.read)
    .await?;
    let Some(hash) = hash else { return Err(ApiError::unauthenticated()) };

    if !auth::verify_password(req.current_password, hash).await? {
        return Err(ApiError::forbidden("Current password doesn't match."));
    }

    let new_hash = auth::hash_password(req.new_password).await?;
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(new_hash)
        .bind(auth.id.to_vec())
        .execute(&state.db.write)
        .await?;

    // Anyone holding an old refresh token (including a thief) re-logs-in.
    auth::revoke_all_for_user(&state.db.write, auth.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_notify_rules(
    State(state): State<AppState>,
    auth: AuthedUser,
) -> Result<Json<Vec<NotifyRule>>, ApiError> {
    let rows: Vec<(Vec<u8>, Option<Vec<u8>>)> = sqlx::query_as(
        "SELECT target_user_id, room_id FROM notify_rules WHERE user_id = ?",
    )
    .bind(auth.id.to_vec())
    .fetch_all(&state.db.read)
    .await?;
    let rules = rows
        .into_iter()
        .map(|(target, room)| {
            Ok(NotifyRule {
                target_user_id: UserId::from_slice(&target).map_err(anyhow::Error::from)?,
                room_id: room
                    .map(|r| linger_core::RoomId::from_slice(&r))
                    .transpose()
                    .map_err(anyhow::Error::from)?,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;
    Ok(Json(rules))
}

async fn put_notify_rule(
    State(state): State<AppState>,
    auth: AuthedUser,
    Json(rule): Json<NotifyRule>,
) -> Result<StatusCode, ApiError> {
    repo::users::expect(&state.db.read, rule.target_user_id).await?;
    if let Some(room) = rule.room_id {
        repo::rooms::expect(&state.db.read, room).await?;
    }
    // SQLite treats NULLs as distinct in primary keys, so "delete then insert"
    // is the only reliable upsert for the all-rooms (NULL) rule.
    let mut tx = state.db.write.begin().await.map_err(ApiError::from)?;
    sqlx::query("DELETE FROM notify_rules WHERE user_id = ? AND target_user_id = ? AND room_id IS ?")
        .bind(auth.id.to_vec())
        .bind(rule.target_user_id.to_vec())
        .bind(rule.room_id.map(|r| r.to_vec()))
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO notify_rules (user_id, target_user_id, room_id) VALUES (?, ?, ?)")
        .bind(auth.id.to_vec())
        .bind(rule.target_user_id.to_vec())
        .bind(rule.room_id.map(|r| r.to_vec()))
        .execute(&mut *tx)
        .await?;
    tx.commit().await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_notify_rule(
    State(state): State<AppState>,
    auth: AuthedUser,
    Json(rule): Json<NotifyRule>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("DELETE FROM notify_rules WHERE user_id = ? AND target_user_id = ? AND room_id IS ?")
        .bind(auth.id.to_vec())
        .bind(rule.target_user_id.to_vec())
        .bind(rule.room_id.map(|r| r.to_vec()))
        .execute(&state.db.write)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
