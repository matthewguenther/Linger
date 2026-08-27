-- Export jobs (SPEC §4.11, PROTOCOL §7, T-801).
--
-- The row is the job. It is written once when somebody asks, updated as the
-- archive is built, and read by `GET /export/:job_id` — there is no in-memory
-- registry, so a job that was running when the server stopped is visible as a
-- stalled row rather than vanishing, and a restart can mark it failed instead
-- of leaving a client polling forever.
--
-- One archive per member at a time: starting a new export deletes the previous
-- one's bytes and row. Otherwise a server accumulates a full copy of itself per
-- member per request, which is the one thing an export must not do to a host's
-- disk. Export objects are deliberately not `attachments` rows — they are not
-- part of the media collection, they do not count against LINGER_POOL_BYTES,
-- and nothing should ever draw one in the grid.

CREATE TABLE exports (
  id              BLOB PRIMARY KEY,            -- UUIDv7
  user_id         BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  state           TEXT NOT NULL,               -- queued | running | complete | failed
  progress        REAL NOT NULL DEFAULT 0,     -- 0.0 .. 1.0
  object_key      TEXT,                        -- set once there is an archive
  size_bytes      INTEGER,
  filename        TEXT,                        -- what it downloads as
  error           TEXT,                        -- why it failed, for the log
  created_at      INTEGER NOT NULL,
  finished_at     INTEGER
);

CREATE INDEX idx_exports_by_user ON exports(user_id, created_at DESC);
