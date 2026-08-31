//! Making and finding DMs (SPEC §4.13, PROTOCOL §3.1, T-1301).
//!
//! One function does the work, and the thing it has to get right is that asking
//! twice for a DM with the same people gives you the same DM. Otherwise a group
//! of three ends up with four conversations, none of which has all of what was
//! said.
//!
//! **The uniqueness is the database's, not this module's.** `member_key` is a
//! `UNIQUE` column (migration `0005_dms.sql`), so two people asking for the same
//! DM in the same instant produce one row and one conflict rather than two
//! rooms. A "check, then insert" in Rust cannot promise that, and a race that
//! rare is one nobody would ever reproduce from a bug report.

use linger_core::wire::{Room, RoomKind};
use linger_core::{RoomId, UserId};
use sqlx::SqlitePool;

use crate::db::now_ms;
use crate::error::ApiError;
use crate::repo;

/// The canonical name for a set of people.
///
/// Sorted, hex, comma-joined. Sorting is the whole trick: a DM with Callie and
/// Dave has to be the same row as a DM with Dave and Callie, and the only way
/// to make two sets compare equal in a `UNIQUE` index is to write them down the
/// same way every time.
fn member_key(members: &[UserId]) -> String {
    let mut hexes: Vec<String> = members.iter().map(|id| hex::encode(id.to_vec())).collect();
    hexes.sort();
    hexes.join(",")
}

/// A DM's slug and name.
///
/// Both are generated and neither is for reading. `rooms.slug` is `NOT NULL
/// UNIQUE` and a good deal of the server addresses rooms by it, so a DM needs
/// one; but a DM is named by who is in it (SPEC §4.13), and a name computed
/// from its members would be wrong for everybody in it — Callie's DM with Dave
/// is "Dave" to Callie and "Callie" to Dave. So the client draws `member_ids`
/// and ignores both of these.
///
/// The `dm-` prefix is reserved: `validate::room_slug` refuses it, so a host
/// cannot make a public room whose slug collides with one of these.
fn generated_slug(id: RoomId) -> String {
    format!("dm-{}", hex::encode(id.to_vec()))
}

/// Find the DM for exactly this set of people, or make it.
///
/// `members` must already be validated: two to eight of them, all real, no
/// duplicates, and including the caller. This function does not re-check,
/// because the route is where a refusal has a sentence to go with it.
pub async fn create_or_find(
    read: &SqlitePool,
    write: &SqlitePool,
    members: &[UserId],
) -> Result<(Room, bool), ApiError> {
    let key = member_key(members);

    if let Some(existing) = by_key(read, &key).await? {
        return Ok((existing, false));
    }

    let id = RoomId::new();
    let now = now_ms();
    let mut tx = write.begin().await.map_err(ApiError::from)?;

    // `position` is what orders the room list, and a DM is not in it. Zero
    // rather than "after the last room": a DM that quietly moved every room's
    // position when it was made would be a private conversation with a visible
    // side effect.
    let inserted = sqlx::query(
        "INSERT INTO rooms (id, slug, name, topic, kind, member_key, position, created_at)
         VALUES (?, ?, ?, NULL, 'dm', ?, 0, ?)",
    )
    .bind(id.to_vec())
    .bind(generated_slug(id))
    .bind(generated_slug(id))
    .bind(&key)
    .bind(now)
    .execute(&mut *tx)
    .await;

    match inserted {
        Ok(_) => {}
        // Somebody else asked for the same DM between the lookup above and
        // here. Their row is the one that exists, so use it — this is the race
        // the UNIQUE index is there to lose gracefully.
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            tx.rollback().await.map_err(ApiError::from)?;
            let existing = by_key(read, &key)
                .await?
                .ok_or_else(|| ApiError::from(anyhow::anyhow!("dm vanished after conflict")))?;
            return Ok((existing, false));
        }
        Err(e) => return Err(e.into()),
    }

    for member in members {
        sqlx::query("INSERT INTO room_members (room_id, user_id, created_at) VALUES (?, ?, ?)")
            .bind(id.to_vec())
            .bind(member.to_vec())
            .bind(now)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await.map_err(ApiError::from)?;

    Ok((repo::rooms::expect(read, id).await?, true))
}

async fn by_key(db: &SqlitePool, key: &str) -> Result<Option<Room>, ApiError> {
    let row: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT id FROM rooms WHERE member_key = ? AND kind = 'dm'")
            .bind(key)
            .fetch_optional(db)
            .await?;
    let Some(bytes) = row else { return Ok(None) };
    let id = RoomId::from_slice(&bytes).map_err(anyhow::Error::from)?;
    let room = repo::rooms::expect(db, id).await?;
    debug_assert_eq!(room.kind, RoomKind::Dm);
    Ok(Some(room))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_member_key_does_not_depend_on_the_order_asked_for() {
        let a = UserId::new();
        let b = UserId::new();
        let c = UserId::new();
        assert_eq!(member_key(&[a, b, c]), member_key(&[c, a, b]));
        assert_eq!(member_key(&[a, b]), member_key(&[b, a]));
    }

    #[test]
    fn different_sets_get_different_keys() {
        let a = UserId::new();
        let b = UserId::new();
        let c = UserId::new();
        assert_ne!(member_key(&[a, b]), member_key(&[a, c]));
        assert_ne!(member_key(&[a, b]), member_key(&[a, b, c]));
    }
}
