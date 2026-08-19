//! Assembling `wire::User` from users + user_style + user_sign + entrance_sounds.

use linger_core::wire::{ColorKey, Fill, NameEffect, Sign, Style, User};
use linger_core::UserId;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

use crate::error::ApiError;

const USER_SELECT: &str = "
    SELECT u.id, u.username, u.display_name, u.is_host, u.last_seen_at,
           s.font_key, s.weight, s.italic, s.fill_kind, s.fill_from, s.fill_to,
           s.effect, s.msg_font_key,
           g.user_id AS sign_user, g.line, g.reading, g.listening, g.working_on,
           g.image_key, g.away_message, g.away_since,
           e.sound_key
    FROM users u
    LEFT JOIN user_style s      ON s.user_id = u.id
    LEFT JOIN user_sign g       ON g.user_id = u.id
    LEFT JOIN entrance_sounds e ON e.user_id = u.id
    WHERE u.deactivated_at IS NULL";

fn row_to_user(row: &SqliteRow) -> Result<User, ApiError> {
    let id = UserId::from_slice(&row.get::<Vec<u8>, _>("id")).map_err(anyhow::Error::from)?;

    // Missing style row = defaults; the columns mirror Style::default().
    let style = match row.get::<Option<String>, _>("font_key") {
        Some(font_key) => {
            let fill_from: String = row.get("fill_from");
            let fill = if row.get::<String, _>("fill_kind") == "gradient" {
                Fill::Gradient {
                    from: ColorKey(fill_from.clone()),
                    to: ColorKey(row.get::<Option<String>, _>("fill_to").unwrap_or(fill_from)),
                }
            } else {
                Fill::Solid { color: ColorKey(fill_from) }
            };
            Style {
                font_key,
                weight: row.get::<i64, _>("weight") as u16,
                italic: row.get::<i64, _>("italic") != 0,
                fill,
                effect: match row.get::<String, _>("effect").as_str() {
                    "shimmer" => NameEffect::Shimmer,
                    "glow" => NameEffect::Glow,
                    _ => NameEffect::None,
                },
                msg_font_key: row.get("msg_font_key"),
            }
        }
        None => Style::default(),
    };

    let sign = row.get::<Option<Vec<u8>>, _>("sign_user").map(|_| Sign {
        line: row.get("line"),
        reading: row.get("reading"),
        listening: row.get("listening"),
        working_on: row.get("working_on"),
        image_key: row.get("image_key"),
        away_message: row.get("away_message"),
        away_since: row.get("away_since"),
    });

    Ok(User {
        id,
        username: row.get("username"),
        display_name: row.get("display_name"),
        is_host: row.get::<i64, _>("is_host") != 0,
        style,
        sign,
        entrance_sound: row.get("sound_key"),
        last_seen_at: row.get("last_seen_at"),
    })
}

/// Every active member, stable order (by username).
pub async fn all(db: &SqlitePool) -> Result<Vec<User>, ApiError> {
    let rows = sqlx::query(&format!("{USER_SELECT} ORDER BY u.username"))
        .fetch_all(db)
        .await?;
    rows.iter().map(row_to_user).collect()
}

pub async fn by_id(db: &SqlitePool, id: UserId) -> Result<Option<User>, ApiError> {
    let row = sqlx::query(&format!("{USER_SELECT} AND u.id = ?"))
        .bind(id.to_vec())
        .fetch_optional(db)
        .await?;
    row.as_ref().map(row_to_user).transpose()
}

/// `by_id` that 404s, for handlers where the user must exist.
pub async fn expect(db: &SqlitePool, id: UserId) -> Result<User, ApiError> {
    by_id(db, id)
        .await?
        .ok_or_else(|| ApiError::not_found("No such person on this stoop."))
}
