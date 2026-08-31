-- DMs and group DMs (SPEC §4.13, PROTOCOL §3.1, T-1301).
--
-- A DM is a room with members. There is no second messages table, no second
-- attachments table and no second anything: everything downstream of `room_id`
-- works unchanged, and the entire difference is who the room is fanned out to.
-- That is the point of doing it this way — a parallel structure would mean
-- every surface growing a second code path, and the second path is the one
-- that gets forgotten.
--
-- Two columns of state, and both are deliberate.
--
-- `kind` is on `rooms` rather than inferred from whether membership rows exist.
-- "A room with no members row" and "a DM whose members failed to insert" look
-- identical otherwise, and one of those is a private conversation visible to
-- the whole server. It is `NOT NULL DEFAULT 'room'`, so every row that existed
-- before this migration is a public room, which is what they are.
--
-- `member_key` is what makes create-or-find work (PROTOCOL §3.1): the member
-- ids, sorted, hex, joined by commas. Sorting is what makes it canonical —
-- asking for a DM with Callie and Dave has to find the one made by asking for
-- Dave and Callie. It is UNIQUE, so two people asking at the same instant end
-- up with one DM and a conflict rather than two DMs, which is a race a check
-- in Rust cannot close.
ALTER TABLE rooms ADD COLUMN kind TEXT NOT NULL DEFAULT 'room';
ALTER TABLE rooms ADD COLUMN member_key TEXT;

CREATE UNIQUE INDEX idx_rooms_member_key ON rooms(member_key)
  WHERE member_key IS NOT NULL;

-- Who is in a DM. Rooms have no rows here — their members are everybody, and
-- writing that out would be a copy of the user table that goes stale the moment
-- somebody joins.
--
-- `ON DELETE CASCADE` on both sides is belt-and-braces for a row that is really
-- deleted, and it is **not** what handles a removed member. Removal from the
-- server is a soft delete — `users.deactivated_at` gets a timestamp and the row
-- stays (T-413) — so the cascade never fires for it and every membership query
-- joins `users` and drops the deactivated ones instead.
--
-- That is the right way round rather than a workaround: `restore` clears the
-- column, and clearing it puts somebody back into the DMs they were in without
-- anything having to remember what they were. Deleting the rows would have
-- thrown that away, and restore is explicitly not an undo of everything else.
CREATE TABLE room_members (
  room_id         BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  user_id         BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at      INTEGER NOT NULL,
  PRIMARY KEY (room_id, user_id)
);

CREATE INDEX idx_room_members_user ON room_members(user_id);
