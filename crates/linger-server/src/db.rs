//! SQLite access (ARCHITECTURE §5). WAL mode, `foreign_keys=ON`,
//! `synchronous=NORMAL`.
//!
//! WAL permits exactly one writer. Rather than hoping a pool serializes writes,
//! the structure makes it impossible to get wrong: [`Db::write`] is a pool with
//! **one** connection (all writes queue behind it), and [`Db::read`] is a
//! read-only pool for everything else. Do not "fix" contention by raising the
//! write pool size — that reintroduces `SQLITE_BUSY` under load (AGENTS.md).

use std::path::Path;
use std::time::Duration;

use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};

#[derive(Debug, Clone)]
pub struct Db {
    /// Single-connection pool: the one WAL writer. Use for INSERT/UPDATE/DELETE.
    pub write: SqlitePool,
    /// Read-only pool for queries. WAL readers never block the writer.
    pub read: SqlitePool,
}

/// Open (creating if missing) the server database and run pending migrations.
pub async fn init(db_path: &Path) -> anyhow::Result<Db> {
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let base = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    let write = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(base.clone())
        .await?;

    sqlx::migrate!("./migrations").run(&write).await?;

    let read = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(base.read_only(true).create_if_missing(false))
        .await?;

    Ok(Db { write, read })
}

/// Current wall-clock time as Unix milliseconds — the only timestamp format on
/// the wire and in the database (PROTOCOL preamble).
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn init_creates_schema_and_both_pools_work() {
        let dir = tempfile::tempdir().unwrap();
        let db = init(&dir.path().join("linger.db")).await.unwrap();

        // Writer can write.
        sqlx::query("INSERT INTO server_config (key, value) VALUES ('name', 'test server')")
            .execute(&db.write)
            .await
            .unwrap();

        // Reader sees it and is actually read-only.
        let (value,): (String,) =
            sqlx::query_as("SELECT value FROM server_config WHERE key = 'name'")
                .fetch_one(&db.read)
                .await
                .unwrap();
        assert_eq!(value, "test server");

        let denied = sqlx::query("INSERT INTO server_config (key, value) VALUES ('x', 'y')")
            .execute(&db.read)
            .await;
        assert!(denied.is_err(), "read pool must reject writes");
    }

    #[tokio::test]
    async fn migrations_are_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("linger.db");
        let _ = init(&path).await.unwrap();
        let _ = init(&path).await.unwrap(); // second run must be a no-op, not an error
    }
}
