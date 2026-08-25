-- Link cards and the media grid's link shelf (SPEC §4.4/§5.6, T-504).
--
-- Two tables, because the two facts have different lifetimes. `message_links`
-- is what a message said and dies with it; `link_previews` is what the web
-- said about a URL, is shared by every message that mentions it, and is a
-- cache the server may throw away and refetch at any time.
--
-- A message's links are re-extracted whenever its body changes, so `position`
-- is the index within that message's current text, not a permanent id.
CREATE TABLE message_links (
  message_id      BLOB NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  position        INTEGER NOT NULL,            -- 0-based, order of appearance
  url             TEXT NOT NULL,
  PRIMARY KEY (message_id, position)
);
CREATE INDEX idx_message_links_message ON message_links(message_id DESC);

CREATE TABLE link_previews (
  url             TEXT PRIMARY KEY,
  state           TEXT NOT NULL,               -- ok | failed
  title           TEXT,
  icon            TEXT,                        -- small data: URI, or NULL
  fetched_at      INTEGER NOT NULL
);
