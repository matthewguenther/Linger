-- Initial schema (ARCHITECTURE §5). IDs are UUIDv7 stored as BLOB(16).
-- Presence is never persisted: it lives in memory in the gateway, on purpose.

CREATE TABLE users (
  id              BLOB PRIMARY KEY,
  username        TEXT NOT NULL UNIQUE,        -- lowercase, [a-z0-9_]{2,24}
  display_name    TEXT NOT NULL,
  password_hash   TEXT NOT NULL,               -- argon2id
  is_host         INTEGER NOT NULL DEFAULT 0,
  created_at      INTEGER NOT NULL,
  last_seen_at    INTEGER,
  deactivated_at  INTEGER
);

-- name and message styling; see SPEC §4.5
CREATE TABLE user_style (
  user_id         BLOB PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  font_key        TEXT NOT NULL DEFAULT 'geist-sans',
  weight          INTEGER NOT NULL DEFAULT 500,
  italic          INTEGER NOT NULL DEFAULT 0,
  fill_kind       TEXT NOT NULL DEFAULT 'solid',   -- solid | gradient
  fill_from       TEXT NOT NULL DEFAULT 'slate',   -- palette key; solid uses this alone
  fill_to         TEXT,                            -- palette key; gradient only
  effect          TEXT NOT NULL DEFAULT 'none',    -- none | shimmer | glow
  msg_font_key    TEXT
);

-- the user status card; see SPEC §4.6
CREATE TABLE user_status (
  user_id         BLOB PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  line            TEXT,                        -- 240 chars
  reading         TEXT,
  listening       TEXT,
  working_on      TEXT,
  image_key       TEXT,
  away_message    TEXT,                        -- supersedes `line` when set
  away_since      INTEGER,
  updated_at      INTEGER NOT NULL
);

CREATE TABLE entrance_sounds (
  user_id         BLOB PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  sound_key       TEXT NOT NULL                -- bundled key, or object key for custom
);

CREATE TABLE rooms (
  id              BLOB PRIMARY KEY,
  slug            TEXT NOT NULL UNIQUE,        -- [a-z0-9-]{1,32}
  name            TEXT NOT NULL,
  topic           TEXT,
  position        INTEGER NOT NULL,
  archived_at     INTEGER,
  created_at      INTEGER NOT NULL
);

CREATE TABLE messages (
  id              BLOB PRIMARY KEY,            -- UUIDv7: chronological
  room_id         BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  author_id       BLOB NOT NULL REFERENCES users(id),
  body            TEXT NOT NULL,
  reply_to        BLOB REFERENCES messages(id) ON DELETE SET NULL,
  pinned_at       INTEGER,
  edited_at       INTEGER,
  deleted_at      INTEGER,
  created_at      INTEGER NOT NULL
);
CREATE INDEX idx_messages_room ON messages(room_id, id DESC);
CREATE INDEX idx_messages_pinned ON messages(room_id, pinned_at) WHERE pinned_at IS NOT NULL;

CREATE TABLE attachments (
  id              BLOB PRIMARY KEY,
  message_id      BLOB REFERENCES messages(id) ON DELETE CASCADE,
  uploader_id     BLOB NOT NULL REFERENCES users(id),
  object_key      TEXT NOT NULL,
  filename        TEXT NOT NULL,
  mime            TEXT NOT NULL,
  size_bytes      INTEGER NOT NULL,
  width           INTEGER,
  height          INTEGER,
  duration_ms     INTEGER,
  blurhash        TEXT,
  poster_key      TEXT,                        -- video poster frame
  starred_at      INTEGER,                     -- starred => never expires
  state           TEXT NOT NULL,               -- pending | complete | failed
  created_at      INTEGER NOT NULL
);
CREATE INDEX idx_attachments_media ON attachments(created_at DESC) WHERE state='complete';

CREATE TABLE reactions (
  message_id      BLOB NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  user_id         BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  key             TEXT NOT NULL,               -- one of 12 fixed keys
  created_at      INTEGER NOT NULL,
  PRIMARY KEY (message_id, user_id, key)
);

CREATE TABLE read_markers (
  user_id         BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  room_id         BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  last_read_id    BLOB NOT NULL,
  updated_at      INTEGER NOT NULL,
  PRIMARY KEY (user_id, room_id)
);

CREATE TABLE notify_rules (
  user_id         BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  target_user_id  BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  room_id         BLOB REFERENCES rooms(id) ON DELETE CASCADE,  -- NULL = all rooms
  PRIMARY KEY (user_id, target_user_id, room_id)
);

CREATE TABLE invites (
  code            TEXT PRIMARY KEY,            -- 12 chars, base32, CSPRNG
  created_by      BLOB NOT NULL REFERENCES users(id),
  expires_at      INTEGER,
  max_uses        INTEGER,
  uses            INTEGER NOT NULL DEFAULT 0,
  revoked_at      INTEGER,
  created_at      INTEGER NOT NULL
);

CREATE TABLE refresh_tokens (
  id              BLOB PRIMARY KEY,
  user_id         BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  family_id       BLOB NOT NULL,               -- login lineage: rotation keeps it,
                                               -- reuse of a rotated token revokes it
  token_hash      TEXT NOT NULL,               -- sha256 of the token
  device_label    TEXT,
  expires_at      INTEGER NOT NULL,
  revoked_at      INTEGER,
  created_at      INTEGER NOT NULL
);
CREATE INDEX idx_refresh_hash ON refresh_tokens(token_hash);

CREATE TABLE server_config (
  key             TEXT PRIMARY KEY,
  value           TEXT NOT NULL
);
