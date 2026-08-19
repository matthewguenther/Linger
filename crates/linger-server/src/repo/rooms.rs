//! Assembling `wire::Room`. `last_message_id` is the newest non-tombstone
//! message — deleting something must not make a room look newly active.

use linger_core::wire::Room;
use linger_core::{MessageId, RoomId};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

use crate::error::ApiError;

const ROOM_SELECT: &str = "
    SELECT r.id, r.slug, r.name, r.topic, r.position, r.archived_at,
           (SELECT MAX(m.id) FROM messages m
             WHERE m.room_id = r.id AND m.deleted_at IS NULL) AS last_message_id
    FROM rooms r";

fn row_to_room(row: &SqliteRow) -> Result<Room, ApiError> {
    Ok(Room {
        id: RoomId::from_slice(&row.get::<Vec<u8>, _>("id")).map_err(anyhow::Error::from)?,
        slug: row.get("slug"),
        name: row.get("name"),
        topic: row.get("topic"),
        position: row.get::<i64, _>("position") as i32,
        archived_at: row.get("archived_at"),
        last_message_id: row
            .get::<Option<Vec<u8>>, _>("last_message_id")
            .map(|b| MessageId::from_slice(&b))
            .transpose()
            .map_err(anyhow::Error::from)?,
    })
}

pub async fn all(db: &SqlitePool) -> Result<Vec<Room>, ApiError> {
    let rows = sqlx::query(&format!("{ROOM_SELECT} ORDER BY r.position, r.slug"))
        .fetch_all(db)
        .await?;
    rows.iter().map(row_to_room).collect()
}

pub async fn by_id(db: &SqlitePool, id: RoomId) -> Result<Option<Room>, ApiError> {
    let row = sqlx::query(&format!("{ROOM_SELECT} WHERE r.id = ?"))
        .bind(id.to_vec())
        .fetch_optional(db)
        .await?;
    row.as_ref().map(row_to_room).transpose()
}

pub async fn expect(db: &SqlitePool, id: RoomId) -> Result<Room, ApiError> {
    by_id(db, id)
        .await?
        .ok_or_else(|| ApiError::not_found("No such room on this stoop."))
}
