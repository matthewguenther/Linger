//! What links a message holds, and what the web said about them (T-504).
//!
//! Two lifetimes, two tables. `message_links` is part of the message and dies
//! with it; `link_previews` is a cache shared by every message that mentions the
//! same URL, and may be thrown away and refetched at any time.

use std::collections::HashMap;

use linger_core::limits::{LINK_PREVIEW_RETRY_MS, LINK_PREVIEW_TTL_MS};
use linger_core::wire::LinkPreview;
use linger_core::MessageId;
use sqlx::{Row, SqlitePool};

use crate::error::ApiError;
use crate::links;

/// A cached preview row.
pub struct Cached {
    pub state: String,
    pub title: Option<String>,
    pub icon: Option<String>,
    pub fetched_at: i64,
}

impl Cached {
    /// Whether this row is old enough to be worth asking again.
    ///
    /// A success stands for a week; a refusal is retried after an hour, so a
    /// site that was down for ten minutes gets another chance without every
    /// reader triggering a fetch in the meantime.
    #[must_use]
    pub fn stale(&self, now: i64) -> bool {
        let age = now - self.fetched_at;
        if self.state == "ok" {
            age > LINK_PREVIEW_TTL_MS
        } else {
            age > LINK_PREVIEW_RETRY_MS
        }
    }

    /// The card this row draws. A refusal keeps its domain and loses everything
    /// else, so a failed fetch never puts a stale or wrong title on screen.
    #[must_use]
    pub fn card(&self, url: &str) -> LinkPreview {
        let ok = self.state == "ok";
        LinkPreview {
            url: url.to_string(),
            domain: links::domain_of(url),
            title: if ok { self.title.clone() } else { None },
            icon: if ok { self.icon.clone() } else { None },
        }
    }
}

/// Replace a message's recorded links. Called on create and on every edit, so
/// the archive follows what the message currently says.
pub async fn replace_for_message(
    db: &SqlitePool,
    message_id: MessageId,
    urls: &[String],
) -> Result<(), ApiError> {
    sqlx::query("DELETE FROM message_links WHERE message_id = ?")
        .bind(message_id.to_vec())
        .execute(db)
        .await?;
    for (position, url) in urls.iter().enumerate() {
        #[allow(clippy::cast_possible_wrap)]
        sqlx::query("INSERT INTO message_links (message_id, position, url) VALUES (?, ?, ?)")
            .bind(message_id.to_vec())
            .bind(position as i64)
            .bind(url)
            .execute(db)
            .await?;
    }
    Ok(())
}

/// Whatever is cached for these URLs.
pub async fn cached(db: &SqlitePool, urls: &[String]) -> Result<HashMap<String, Cached>, ApiError> {
    if urls.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = vec!["?"; urls.len()].join(",");
    let sql =
        format!("SELECT url, state, title, icon, fetched_at FROM link_previews WHERE url IN ({placeholders})");
    let mut query = sqlx::query(&sql);
    for url in urls {
        query = query.bind(url);
    }
    let rows = query.fetch_all(db).await?;
    Ok(rows
        .iter()
        .map(|row| {
            (
                row.get::<String, _>("url"),
                Cached {
                    state: row.get("state"),
                    title: row.get("title"),
                    icon: row.get("icon"),
                    fetched_at: row.get("fetched_at"),
                },
            )
        })
        .collect())
}

/// Record what a fetch found — including that it found nothing, which is the
/// row that stops the next reader repeating the attempt.
pub async fn store(
    db: &SqlitePool,
    url: &str,
    found: &links::Fetched,
    now: i64,
) -> Result<(), ApiError> {
    let state = if found.title.is_some() || found.icon.is_some() {
        "ok"
    } else {
        "failed"
    };
    sqlx::query(
        "INSERT INTO link_previews (url, state, title, icon, fetched_at)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(url) DO UPDATE SET
           state = excluded.state, title = excluded.title,
           icon = excluded.icon, fetched_at = excluded.fetched_at",
    )
    .bind(url)
    .bind(state)
    .bind(found.title.as_deref())
    .bind(found.icon.as_deref())
    .bind(now)
    .execute(db)
    .await?;
    Ok(())
}
