-- Full-text search over messages (SPEC §4.12, PROTOCOL §6, T-1201).
--
-- One FTS5 row per message, keyed by the message's implicit `rowid`, holding
-- two columns: what somebody typed, and the names of the files on that message,
-- joined by `char(31)`. That separator is not a stylistic choice: a filename may
-- contain spaces, and a search hit has to be able to say *which* file it
-- matched. `validate::filename` strips control characters, so U+001F cannot
-- appear inside one, and the tokenizer treats it as a word break like any other
-- punctuation.
-- SPEC §4.12 decides that pair — a photo is findable by its filename, and a
-- link is findable only by the text somebody wrote around it, because a link's
-- title is a cache this server refetches on its own schedule and an index that
-- rewrites itself in the background is an index nobody can reason about.
--
-- The table carries its own copy of the text rather than reading it back out of
-- `messages` (FTS5's `content=` option). That costs a second copy of every
-- message body, which at the scale this product is for is a few megabytes, and
-- it buys two things: `snippet()` works (a contentless table cannot produce
-- one), and the index cannot be silently wrong about a row it can no longer
-- read.
--
-- **Tombstones are not searchable.** Deleting a message empties its body and
-- sets `deleted_at` (it stays a row so reply chains survive), and the triggers
-- below drop its index row outright — body and filenames both. A deleted
-- message is deleted, the same rule the export follows. An edit *replaces* the
-- row rather than adding to it, so a word taken out of a message stops matching
-- it.
--
-- `porter` stems English endings, so searching for "photo" finds "photos" and
-- searching for "running" finds "run". The query is stemmed the same way it was
-- indexed, so the two always agree. `remove_diacritics 2` folds accents
-- correctly for characters outside the BMP as well.

CREATE VIRTUAL TABLE message_fts USING fts5(
  body,
  filenames,
  tokenize = "porter unicode61 remove_diacritics 2"
);

-- Backfill: a server that already has history is searchable the moment it comes
-- up on this version, without anybody running anything.
INSERT INTO message_fts (rowid, body, filenames)
SELECT m.rowid,
       m.body,
       COALESCE((SELECT group_concat(a.filename, char(31))
                 FROM attachments a
                 WHERE a.message_id = m.id AND a.state = 'complete'), '')
FROM messages m
WHERE m.deleted_at IS NULL;

-- Every trigger below is "throw the row away and write what is true now",
-- rather than an incremental edit. It is a handful of statements either way and
-- the rebuild cannot drift from the table it indexes.

CREATE TRIGGER message_fts_ai AFTER INSERT ON messages BEGIN
  INSERT INTO message_fts (rowid, body, filenames)
  SELECT new.rowid, new.body, ''
  WHERE new.deleted_at IS NULL;
END;

-- `OF body, deleted_at` and not every column: pinning a message does not change
-- a word of it, and neither does anything else that writes this table.
CREATE TRIGGER message_fts_au AFTER UPDATE OF body, deleted_at ON messages BEGIN
  DELETE FROM message_fts WHERE rowid = old.rowid;
  INSERT INTO message_fts (rowid, body, filenames)
  SELECT new.rowid,
         new.body,
         COALESCE((SELECT group_concat(a.filename, char(31))
                   FROM attachments a
                   WHERE a.message_id = new.id AND a.state = 'complete'), '')
  WHERE new.deleted_at IS NULL;
END;

CREATE TRIGGER message_fts_ad AFTER DELETE ON messages BEGIN
  DELETE FROM message_fts WHERE rowid = old.rowid;
END;

-- A file becomes part of a message some time after both exist: the bytes are
-- uploaded and checked first, and posting the message sets `message_id` on a
-- row that is already `complete` (ARCHITECTURE §8). So the filenames column is
-- maintained from this side, and both ends of a move are re-indexed — the
-- message that lost the file and the one that gained it.
CREATE TRIGGER message_fts_attach_au
AFTER UPDATE OF message_id, filename, state ON attachments BEGIN
  DELETE FROM message_fts
  WHERE rowid = (SELECT m.rowid FROM messages m WHERE m.id = old.message_id);
  INSERT INTO message_fts (rowid, body, filenames)
  SELECT m.rowid,
         m.body,
         COALESCE((SELECT group_concat(a.filename, char(31))
                   FROM attachments a
                   WHERE a.message_id = m.id AND a.state = 'complete'), '')
  FROM messages m
  WHERE m.id = old.message_id AND m.deleted_at IS NULL;

  DELETE FROM message_fts
  WHERE rowid = (SELECT m.rowid FROM messages m WHERE m.id = new.message_id);
  INSERT INTO message_fts (rowid, body, filenames)
  SELECT m.rowid,
         m.body,
         COALESCE((SELECT group_concat(a.filename, char(31))
                   FROM attachments a
                   WHERE a.message_id = m.id AND a.state = 'complete'), '')
  FROM messages m
  WHERE m.id = new.message_id AND m.deleted_at IS NULL;
END;

-- A file swept at 365 days, or an upload thrown away, stops being a way to find
-- the message it was on. When the *message* is what went, its own delete
-- trigger has already dropped the index row and the re-insert here finds no
-- message to write, whichever order the two fire in.
CREATE TRIGGER message_fts_attach_ad AFTER DELETE ON attachments BEGIN
  DELETE FROM message_fts
  WHERE rowid = (SELECT m.rowid FROM messages m WHERE m.id = old.message_id);
  INSERT INTO message_fts (rowid, body, filenames)
  SELECT m.rowid,
         m.body,
         COALESCE((SELECT group_concat(a.filename, char(31))
                   FROM attachments a
                   WHERE a.message_id = m.id AND a.state = 'complete'), '')
  FROM messages m
  WHERE m.id = old.message_id AND m.deleted_at IS NULL;
END;
