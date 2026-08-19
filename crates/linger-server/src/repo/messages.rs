//! Assembling `wire::Message`: page fetch plus reaction grouping.
//! Attachments join lands in M6; until then every message carries `[]`.

use std::collections::HashMap;

use linger_core::wire::{Message, ReactionGroup};
use linger_core::{MessageId, RoomId, UserId, REACTIONS};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

use crate::error::ApiError;

fn row_to_message(row: &SqliteRow) -> Result<Message, ApiError> {
    Ok(Message {
        id: MessageId::from_slice(&row.get::<Vec<u8>, _>("id")).map_err(anyhow::Error::from)?,
        room_id: RoomId::from_slice(&row.get::<Vec<u8>, _>("room_id"))
            .map_err(anyhow::Error::from)?,
        author_id: UserId::from_slice(&row.get::<Vec<u8>, _>("author_id"))
            .map_err(anyhow::Error::from)?,
        body: row.get("body"),
        reply_to: row
            .get::<Option<Vec<u8>>, _>("reply_to")
            .map(|b| MessageId::from_slice(&b))
            .transpose()
            .map_err(anyhow::Error::from)?,
        attachments: Vec::new(),
        reactions: Vec::new(),
        pinned_at: row.get("pinned_at"),
        edited_at: row.get("edited_at"),
        deleted_at: row.get("deleted_at"),
        created_at: row.get("created_at"),
    })
}

/// Attach reaction groups to a batch of messages. Groups are emitted in the
/// canonical `REACTIONS` order; user_ids in first-reacted order (for hover).
async fn hydrate_reactions(db: &SqlitePool, messages: &mut [Message]) -> Result<(), ApiError> {
    if messages.is_empty() {
        return Ok(());
    }
    let placeholders = vec!["?"; messages.len()].join(",");
    let sql = format!(
        "SELECT message_id, key, user_id FROM reactions
         WHERE message_id IN ({placeholders}) ORDER BY created_at"
    );
    let mut query = sqlx::query(&sql);
    for m in messages.iter() {
        query = query.bind(m.id.to_vec());
    }
    let rows = query.fetch_all(db).await?;

    let mut grouped: HashMap<(MessageId, String), Vec<UserId>> = HashMap::new();
    for row in &rows {
        let mid = MessageId::from_slice(&row.get::<Vec<u8>, _>("message_id"))
            .map_err(anyhow::Error::from)?;
        let uid = UserId::from_slice(&row.get::<Vec<u8>, _>("user_id"))
            .map_err(anyhow::Error::from)?;
        grouped.entry((mid, row.get("key"))).or_default().push(uid);
    }

    for m in messages.iter_mut() {
        m.reactions = REACTIONS
            .iter()
            .filter_map(|key| {
                grouped.remove(&(m.id, (*key).to_string())).map(|user_ids| ReactionGroup {
                    key: (*key).to_string(),
                    count: user_ids.len() as u32,
                    user_ids,
                })
            })
            .collect();
    }
    Ok(())
}

/// One fully assembled message.
pub async fn by_id(db: &SqlitePool, id: MessageId) -> Result<Option<Message>, ApiError> {
    let row = sqlx::query("SELECT * FROM messages WHERE id = ?")
        .bind(id.to_vec())
        .fetch_optional(db)
        .await?;
    let Some(row) = row else { return Ok(None) };
    let mut messages = vec![row_to_message(&row)?];
    hydrate_reactions(db, &mut messages).await?;
    Ok(messages.pop())
}

pub async fn expect(db: &SqlitePool, id: MessageId) -> Result<Message, ApiError> {
    by_id(db, id)
        .await?
        .ok_or_else(|| ApiError::not_found("No such message."))
}

/// A page of messages, newest-first (PROTOCOL §4). `before`/`after` are ids;
/// UUIDv7 blobs compare chronologically so this is a pure range scan.
pub async fn page(
    db: &SqlitePool,
    room_id: RoomId,
    before: Option<MessageId>,
    after: Option<MessageId>,
    limit: u32,
) -> Result<Vec<Message>, ApiError> {
    let mut sql = String::from("SELECT * FROM messages WHERE room_id = ?");
    if before.is_some() {
        sql.push_str(" AND id < ?");
    }
    if after.is_some() {
        sql.push_str(" AND id > ?");
    }
    sql.push_str(" ORDER BY id DESC LIMIT ?");

    let mut query = sqlx::query(&sql).bind(room_id.to_vec());
    if let Some(b) = before {
        query = query.bind(b.to_vec());
    }
    if let Some(a) = after {
        query = query.bind(a.to_vec());
    }
    query = query.bind(i64::from(limit));

    let rows = query.fetch_all(db).await?;
    let mut messages = rows.iter().map(row_to_message).collect::<Result<Vec<_>, _>>()?;
    hydrate_reactions(db, &mut messages).await?;
    Ok(messages)
}

/// One reaction key's current state on a message, for `reaction.update` fan-out.
pub async fn reaction_group(
    db: &SqlitePool,
    message_id: MessageId,
    key: &str,
) -> Result<ReactionGroup, ApiError> {
    let rows = sqlx::query(
        "SELECT user_id FROM reactions WHERE message_id = ? AND key = ? ORDER BY created_at",
    )
    .bind(message_id.to_vec())
    .bind(key)
    .fetch_all(db)
    .await?;
    let user_ids = rows
        .iter()
        .map(|r| UserId::from_slice(&r.get::<Vec<u8>, _>("user_id")).map_err(anyhow::Error::from))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ReactionGroup { key: key.to_string(), count: user_ids.len() as u32, user_ids })
}
