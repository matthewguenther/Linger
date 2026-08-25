# T-006 — the vocabulary change (closed 2026-08-19)

Archived from `TASKS.md` on 2026-08-25, verbatim, when the queue was slimmed
so a task session no longer pays to read finished history. The landing notes
here are the project's memory — read this file when a task in `TASKS.md`
points at it. Do not rewrite history; add corrections as new dated lines.

## How it ended

- ✅ **T-006 — the vocabulary change** (2026-08-19): the coined words are gone,
  ahead of M3 writing any UI copy. Full mapping and the presence-naming call are
  recorded under the task below.

---

## Vocabulary change — **done, ahead of T-301**

- ✅ **T-006 · Drop the coined vocabulary** — effort: **medium** *(Opus 5: high)*
  **Decided by Matt 2026-08-19:** the invented words go. The product is Linger;
  an instance is a *server*. Doing this before M3 is deliberate — M3 writes the
  UI copy and component names, and renaming after it lands costs several times
  more.

  This is one atomic change: AGENTS.md requires docs and behavior to move in the
  same commit, so SPEC/PROTOCOL/ARCHITECTURE/README/code/tests all go together.
  There are no real deployments, so **edit `0001_init.sql` in place** rather than
  adding a migration; delete any local `data/linger.db` afterwards.

  | Old | New | Notes |
  |---|---|---|
  | a stoop | **a server** | "One person runs a Linger server." |
  | `GET/PATCH /stoop` | `/server` | breaking, but no client ships yet |
  | `StoopInfo` / `UpdateStoopRequest` | `ServerInfo` / `UpdateServerRequest` | |
  | `stoop_config` table | `server_config` | |
  | `stoop_name` (setup, invite preview) | `server_name` | |
  | the shelf | **media** | `/shelf` → `/media`, `ShelfItem` → `MediaItem` |
  | their sign | **status** | `Sign` → `UserStatus`, `user_sign` → `user_status`, `User.sign` → `User.status` |
  | sitting in | **in the room** | see the call below |
  | a room, the host | *unchanged* | already plain English |

  **The one judgment call — presence naming.** `PresenceState::Sitting` and the
  client op `room.sit` both encode "sitting". Recommended: state `sitting` →
  **`in_room`**, client op `room.sit` → **`room.focus`** (it fires on focus, and
  `null` still means "left"). Do *not* reuse `room.enter` for the client op — it
  already exists as a server→client event that triggers entrance sounds. Rename
  `Gateway::apply_sit` → `apply_room_focus` to match. If any of this reads badly
  in context, pick better and change the docs in the same commit.

  **Do not rename:** the `linger-server` crate/process, `LINGER_*` env vars, or
  the repo. Watch for accidental "linger-server server" phrasings and for
  `spawn_stoop`/`TestStoop`/`stoop_with_room` in `tests/common/mod.rs`, which the
  whole suite calls.

  *Accept:* `grep -ri "stoop\|shelf\|sitting in" --exclude-dir=target
  --exclude-dir=node_modules --exclude-dir=.git .` returns nothing outside a
  historical note; `cargo test --workspace`, `cargo fmt --all --check`, and
  `cd client && pnpm check` green; CI green after push. SPEC §1's vocabulary
  table and AGENTS.md hard rule 6 must describe the *new* words, not the old.

  ### **Done 2026-08-19.** Every rename above landed as specified.

  *The recommended presence naming was taken as-is: `PresenceState::Sitting` →
  `InRoom` (wire `"in_room"`), `ClientFrame::RoomSit` → `RoomFocus` (op
  `room.focus`), `Gateway::apply_sit` → `apply_room_focus`. `InRoom` needed an
  explicit `#[serde(rename = "in_room")]` — the enum's `rename_all =
  "lowercase"` would have emitted `"inroom"`, which no doc anywhere describes.
  That is the one spot in this change where a silent wrong answer was possible;
  `tests/gateway.rs` now asserts the wire string.*

  *`0001_init.sql` was edited in place per the task (`stoop_config` →
  `server_config`, `user_sign` → `user_status`, and the incidental
  `idx_attachments_shelf` → `idx_attachments_media`). There was no local
  `data/linger.db` to delete — `.gitignore` covers `data/`, and the dev box's
  copy was already gone. **Anyone holding a database created before this commit
  must delete it**; there is no migration path and won't be one.*

  *Two judgment calls beyond the spec, both recorded in the docs they touch:*
  - *`ShelfItem`'s file `client/src/generated/ShelfItem.ts` (and `Sign.ts`,
    `StoopInfo.ts`, `UpdateStoopRequest.ts`) had to be deleted by hand — **ts-rs
    only writes, it never removes stale exports**, so a rename leaves the old
    binding on disk and CI's drift check would not have caught it. Worth
    remembering for any future wire-type rename.*
  - *SPEC §1 previously argued *for* the coined words; it now records that they
    were dropped and why, rather than silently deleting the paragraph. AGENTS.md
    rule 6 lists each retired word against its replacement so the rule is
    enforceable by grep.*

  *`linger-server`, the `LINGER_*` env vars, and the repo name are untouched, as
  required. The `linger-server` doc comment now reads "the server process" to
  keep the binary distinct from the instance it hosts, and SPEC §1 states that
  distinction explicitly.*

