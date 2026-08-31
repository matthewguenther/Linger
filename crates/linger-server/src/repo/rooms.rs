//! Assembling `wire::Room`, for both kinds of room (SPEC §4.13).
//!
//! `last_message_id` is the newest non-tombstone message — deleting something
//! must not make a room look newly active.
//!
//! **Every query here that a member could reach is membership-aware**, and the
//! ones that are not say so in their name. `all` is the server's public rooms;
//! `dms_for` is one person's DMs; `visible_to` answers "may this person see this
//! room at all", which is the check every other surface asks before it answers.

use linger_core::wire::{Room, RoomKind};
use linger_core::{MessageId, RoomId, UserId};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

use crate::error::ApiError;

const ROOM_SELECT: &str = "
    SELECT r.id, r.slug, r.name, r.topic, r.kind, r.position, r.archived_at,
           (SELECT MAX(m.id) FROM messages m
             WHERE m.room_id = r.id AND m.deleted_at IS NULL) AS last_message_id
    FROM rooms r";

fn row_to_room(row: &SqliteRow) -> Result<Room, ApiError> {
    let kind = match row.get::<String, _>("kind").as_str() {
        "dm" => RoomKind::Dm,
        _ => RoomKind::Room,
    };
    Ok(Room {
        id: RoomId::from_slice(&row.get::<Vec<u8>, _>("id")).map_err(anyhow::Error::from)?,
        slug: row.get("slug"),
        name: row.get("name"),
        topic: row.get("topic"),
        kind,
        // Filled in by the callers that return DMs. A room's is `None` and
        // stays `None` — its members are everybody (see `wire::Room`).
        member_ids: None,
        position: row.get::<i64, _>("position") as i32,
        archived_at: row.get("archived_at"),
        last_message_id: row
            .get::<Option<Vec<u8>>, _>("last_message_id")
            .map(|b| MessageId::from_slice(&b))
            .transpose()
            .map_err(anyhow::Error::from)?,
    })
}

/// The server's public rooms. Never a DM — `GET /rooms` and the `ready` frame's
/// `rooms` both come through here, and neither has ever been able to carry a
/// private conversation because this query cannot produce one.
pub async fn all(db: &SqlitePool) -> Result<Vec<Room>, ApiError> {
    let rows = sqlx::query(&format!(
        "{ROOM_SELECT} WHERE r.kind = 'room' ORDER BY r.position, r.slug"
    ))
    .fetch_all(db)
    .await?;
    rows.iter().map(row_to_room).collect()
}

/// One room by id, whatever kind it is and whoever is asking.
///
/// **This one does not check membership**, so it is not the function a route
/// should reach for. Use [`visible_to`], which is this plus the check.
pub async fn by_id(db: &SqlitePool, id: RoomId) -> Result<Option<Room>, ApiError> {
    let row = sqlx::query(&format!("{ROOM_SELECT} WHERE r.id = ?"))
        .bind(id.to_vec())
        .fetch_optional(db)
        .await?;
    let Some(row) = row else { return Ok(None) };
    let mut room = row_to_room(&row)?;
    if room.kind == RoomKind::Dm {
        room.member_ids = Some(members(db, id).await?);
    }
    Ok(Some(room))
}

pub async fn expect(db: &SqlitePool, id: RoomId) -> Result<Room, ApiError> {
    by_id(db, id)
        .await?
        .ok_or_else(|| ApiError::not_found("No such room on this server."))
}

/// The room, if this person is allowed to see it at all.
///
/// **This is the function routes should call**, and the refusal is deliberately
/// `NOT_FOUND` rather than `FORBIDDEN`: telling somebody "you may not see this
/// DM" tells them the DM exists, who it is between if they guessed the id, and
/// that there is something to be curious about. There is nothing a non-member
/// can ask that distinguishes a DM they are not in from a room that was never
/// there (PROTOCOL §3.1).
pub async fn visible_to(db: &SqlitePool, id: RoomId, user_id: UserId) -> Result<Room, ApiError> {
    let room = expect(db, id).await?;
    if room.kind == RoomKind::Room {
        return Ok(room);
    }
    let is_member = room
        .member_ids
        .as_ref()
        .is_some_and(|ids| ids.contains(&user_id));
    if is_member {
        Ok(room)
    } else {
        Err(ApiError::not_found("No such room on this server."))
    }
}

/// The DMs one person is in, newest conversation first.
///
/// Ordered by the newest message rather than by when the DM was made: a DM is
/// found by who you were just talking to, and a list ordered by creation puts
/// the conversation you have not touched in months at the top forever.
pub async fn dms_for(db: &SqlitePool, user_id: UserId) -> Result<Vec<Room>, ApiError> {
    let rows = sqlx::query(&format!(
        "{ROOM_SELECT}
         JOIN room_members me ON me.room_id = r.id AND me.user_id = ?
         WHERE r.kind = 'dm'
         ORDER BY last_message_id DESC, r.id DESC"
    ))
    .bind(user_id.to_vec())
    .fetch_all(db)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in &rows {
        let mut room = row_to_room(row)?;
        room.member_ids = Some(members(db, room.id).await?);
        out.push(room);
    }
    Ok(out)
}

/// Who is in a DM. Empty for a public room, which has no rows here — its
/// members are everybody (migration `0005_dms.sql`).
///
/// **Deactivated accounts are dropped.** Removal from the server is a soft
/// delete, so the membership row survives it on purpose: `restore` puts
/// somebody back into the DMs they were in without anything having had to
/// remember what they were (T-413). Until then they are not a member here, and
/// this is the single place that decides it.
pub async fn members(db: &SqlitePool, room_id: RoomId) -> Result<Vec<UserId>, ApiError> {
    let rows = sqlx::query(
        "SELECT rm.user_id FROM room_members rm
         JOIN users u ON u.id = rm.user_id AND u.deactivated_at IS NULL
         WHERE rm.room_id = ?
         ORDER BY rm.user_id",
    )
    .bind(room_id.to_vec())
    .fetch_all(db)
    .await?;
    rows.iter()
        .map(|row| {
            UserId::from_slice(&row.get::<Vec<u8>, _>("user_id"))
                .map_err(|e| ApiError::from(anyhow::Error::from(e)))
        })
        .collect()
}

/// Every DM on the server with its members, for the gateway's audience index
/// (`Gateway::load_rooms`). Only called at startup and after a membership
/// change, never per frame.
pub async fn all_dm_members(db: &SqlitePool) -> Result<Vec<(RoomId, Vec<UserId>)>, ApiError> {
    let rows = sqlx::query("SELECT id FROM rooms WHERE kind = 'dm'")
        .fetch_all(db)
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in &rows {
        let id = RoomId::from_slice(&row.get::<Vec<u8>, _>("id")).map_err(anyhow::Error::from)?;
        out.push((id, members(db, id).await?));
    }
    Ok(out)
}
