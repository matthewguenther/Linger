# TASKS.md — the running work queue

This file is the handoff surface between the architect session (which maintains
structure and this file) and implementation sessions run on **Opus 5**. It is a live
document: check tasks off, add discoveries, and keep it truthful.

Tasks are marked **⬜ not started** or **✅ done** — emoji, not markdown `- [ ]`
checkboxes, so the state of the queue is visible at a glance while scrolling. Use
the same two characters when you add or close a task.

Task numbers match the milestone: T-5xx is M5, T-6xx is M6, and so on. Work that
is still V1 but not on the critical path lives at the end as T-9xx (sounds) and
T-91x (activity detection). A 2026-08-23 renumber, after activity detection left
the main sequence: old M6–M9 are M5–M8; entrance sounds T-403/404/408 are
T-901/902/903; activity T-501…T-507 are T-911…T-917.

## How to run a task

**Model:** One task per fresh session — a
clean context follows the task spec better and costs less than a long-running
one. The prompt that works: *"Read AGENTS.md and TASKS.md, then do task T-xxx.
State the current milestone first."*

1. Set the session's **effort** from the task's label using the mapping below.
2. Read `AGENTS.md` in full, then the spec sections the task references. State
   the current milestone before writing code.
3. Do not start a milestone until the previous one passes its check
   (`ARCHITECTURE.md` §10). Do not pull work from a later milestone "while you're
   in there."
4. When done: all listed acceptance criteria pass, `cargo test --workspace` and
   `cd client && pnpm check` are green, CI is green after push, the task is
   flipped ⬜ → ✅ here, and any surprises are noted under the task.
5. **Never add AI attribution anywhere** — commits, comments, metadata. Author is
   Matt Guenther. No exceptions.

**Effort mapping (task label → Opus 5 effort setting):**

| Task label | Opus 5 effort | Why |
|---|---|---|
| **low** | **medium** | Mechanical, tightly specified; high effort just burns credits re-deriving what the task text already decides |
| **medium** | **high** | Real features with judgment in the details; high is the sweet spot |
| **high** | **high**, except the four below | Cross-cutting but well-scaffolded — the architecture docs carry a lot of the load |
| T-302, T-911, T-501, T-701+T-702 | **xhigh** | The genuinely treacherous ones: realtime client resume, Wayland/KWin, resumable uploads + media pipeline, signing/notarization — AGENTS.md §"Where you will be wrong" territory |

Running everything at xhigh is not better — it's slower, pricier, and prone to
overbuilding simple tasks. Match the effort to the label and escalate only if a
task fails its acceptance criteria twice.

---

## Status

- ✅ **SPIKE (Linux)** — retired 2026-08-19 on this repo's primary target. KWin
  scripting over D-Bus works on Plasma 6.6 Wayland: got
  `resource_class="code", pid → /proc/<pid>/exe` with zero title exposure.
  The working recipe is documented in `crates/linger-activity/src/backend.rs`.
- ✅ **SPIKE (Windows)** — retired 2026-08-19 on a real Windows runner (T-004).
  Foreground window → pid → exe path → registry hit, working first try, no
  window titles anywhere. Recipe in `crates/linger-activity/src/backend.rs`.
- ✅ **M0** — scaffold complete, CI green (fmt gated, clippy -D warnings, 96
  tests, bindings drift check, web build, WebKitGTK shell check), the shell
  builds on all three OSes, and the **`oklch()` gate is answered: yes** (T-002).
  *One human errand is outstanding and does not block anything: the visual
  "a window opens" sign-off on Linux/Windows/macOS — see T-002 and T-003. It
  must be closed before M7.*
- ✅ **Real-binary smoke test** (2026-08-19): booted `linger-server` from a clean
  data dir; the printed setup URL created the host, login → room → message all
  worked over curl, and the setup link died after use (404).
- ✅ **M1 — server REST** (2026-08-19): auth (argon2id, EdDSA JWT, rotating
  refresh with family-reuse revocation), first-run setup, invites, server/rooms,
  messages/reactions/read markers, users/styling/statuses, rate limiting. Every
  endpoint has an integration test driving real HTTP (T-101…T-107 below).
- ✅ **M2 — gateway** (2026-08-19): WS with hello/identify/ready, heartbeat,
  per-session sequence numbers, presence + room focus + occupancy + entrance-sound
  fan-out, typing limits, and resume via a 500-frame/120s session ring. The
  milestone check passes: a forced mid-stream disconnect resumes with **no gaps
  and no duplicates**, asserted by sequence accounting over real sockets
  (`tests/gateway.rs`).
- ✅ **T-006 — the vocabulary change** (2026-08-19): the coined words are gone,
  ahead of M3 writing any UI copy. Full mapping and the presence-naming call are
  recorded under the task below.
- ✅ **M3 — client: message stream** (2026-08-21). The milestone check passed on
  2026-08-20. T-301 landed 2026-08-19 (sign in, and stay signed in).
  T-302 landed 2026-08-19: the client holds a live gateway connection from the
  Tauri core, survives the server being killed and restarted with no user
  action, and resumes without gaps or duplicates against a real server. T-303
  landed 2026-08-20: the stream is real, two clients on one machine exchanged
  messages in real time, and 10,000 messages of scrollback held 24–43 rows in
  the DOM. T-304 landed 2026-08-20: markdown that cannot become markup, a
  composer you can write a paragraph in, edit/delete/reply, and reactions that
  read as weight instead of a tally. T-305 landed 2026-08-21 and finished the
  milestone: a "you left off here" line, rooms that change weight instead of
  wearing a number, a "since you were gone" view you pull from the header, and
  the one notification the product allows.
- ✅ **M4 — presence, roster, statuses** (2026-08-21) — the milestone check
  passed: the roster moves live across two clients, and a status set on one
  shows up on the other. T-405 also found and fixed a T-402 bug that had been
  stopping presence dead a few seconds after every launch — details under the
  task. T-401 landed
  2026-08-21: the roster is the card stack the product is about, it moves live
  across clients, and on a narrow window it becomes a strip above the composer
  instead of hiding. **T-402 landed 2026-08-21: focusing a room puts you in it,
  90 seconds of background or sitting still takes you out, ten minutes with no
  input is idle, and occupancy is names in the header and a small stack on the
  rail.** Verified independently on 2026-08-21: 158 client tests, 124 workspace
  tests, `pnpm check` clean, no binding drift. T-405 landed 2026-08-21 and
  finished the milestone: a status editor, an away message that supersedes the
  status, and a card on any name in the stream. The entrance-sound tasks are on
  the Backburner (below), so **M4 is done**.
- ⬜ **M4.5 — the shell's missing surfaces** — new, added 2026-08-21. There is no
  way to add a room from inside the client, no invite screen, no server settings,
  no member settings, and no server list. T-410, T-411, T-412. **T-413 and T-414
  were added later the same day**, after reading T-410 with Matt turned up two
  holes in running a server: nobody can be removed once they are in, and a host
  who forgets their password has no way back in at all. The decisions behind both
  are recorded under *Decided — the host's side* below.
  **T-410 landed 2026-08-21**: the host can make, rename, reorder and archive rooms,
  hand out and revoke invite links, and rename the server, all from inside the app —
  and a member sees none of those controls.
  **T-411 landed 2026-08-23**: a member can change their display name, password and
  density from inside the app, and the first-run copy no longer assumes they already
  know how it works.
  **T-412 landed 2026-08-25**: the rail has a server list, you can be signed into
  several servers at once, and each one has its own connection, keyring entry,
  people and rooms. **M4.5's milestone check passes** — the remaining tasks,
  T-413, T-414 and T-415, never gated it.
  **T-414 landed 2026-08-25**: `linger-server reset-password <username>` is a
  subcommand on the server binary, so a host who forgets their password has a way
  back in that does not need a reset email Linger cannot send. It prints a new
  password (or takes one on stdin), signs that account out everywhere, and the
  README has a "Locked out" section next to Backup.
  **T-415 was added the same day as T-410**, from something T-410 hit on
  a real server: a person who joins while you are connected never appears until you
  restart the app.
- ⬜ **M5 … M8** — queued below. M5 starts when M4.5's check passes.
- ⏬ **Backburner (after M8)** — entrance sounds (T-901, T-902, T-903) and
  activity detection (T-911…T-917). Sounds: Matt, 2026-08-21. Activity: Matt,
  2026-08-23 — still V1, not needed for a usable product, and large enough that
  it would sit in the way of the rest. Do not start T-911.
- 🚫 **AI is off the roadmap** (Matt, 2026-08-19). The local-model features and the
  agent surface that used to sit behind V1 are cut — SPEC §8 records why, AGENTS
  rule 13 is the enforceable version. Do not build any of it back.

**The one thing that bit us in T-301:** a webview page is a cross-origin caller,
so the server had to start sending CORS headers before the client could read a
single response. The allowed origins are a fixed list in
`crates/linger-server/src/routes/mod.rs`. The gateway WebSocket is *not* subject
to CORS and T-302 confirmed that — but if a future browser-side call mysteriously
"can't reach the server", that list is the first place to look.

Two decisions styling and the activity backends no longer have to make: **use
`oklch()` directly** (WebKitGTK 2.52.3 supports it, T-002), and **T-913's Win32
backend is a known quantity** (T-004). Both recipes are recorded next to the
code that will need them.

What already exists (do not rebuild): workspace + CI; `linger-core` with typed
UUIDv7 ids, the full REST + gateway wire contract, palette/fonts/reactions/limits,
ts-rs export to `client/src/generated/` (committed, drift-checked in CI);
`linger-server` with config/env, WAL SQLite with **single-writer pool discipline**
(`db.write` is a 1-connection pool — keep it that way), migrations (full §5 schema),
error envelope, health route, integration-test harness pattern
(`crates/linger-server/tests/health.rs` — copy its `spawn_server` shape);
`linger-activity` with the resolution pipeline, registry loader (+41 seed entries),
backend classifier; Tauri 2 shell with the Console-token M0 frame; deploy files.

---

## M0 wrap-up

- ✅ **T-001 · First CI run goes green** — effort: **low**
  Push to GitHub, watch `ci.yml`. Fix anything `clippy --workspace --all-targets
  -- -D warnings` finds (none expected to be structural). Run `cargo fmt --all`
  from a rustfmt-equipped toolchain (dev box has none — CI's does; a devcontainer
  or `rustup component add rustfmt` on any machine works), commit the formatting,
  then delete the `continue-on-error: true` line from the fmt step so it gates.
  *Accept:* all three CI jobs green with fmt gating.
  *Done 2026-08-19: all three jobs green, fmt gating, clippy -D warnings clean
  (fixed 4 type_complexity findings). Tree formatted with rustfmt 1.9.0-stable.
  Note: the dev box has no rustfmt — a scratchpad rustup toolchain was used;
  future formatting runs need the same or CI's word is final.*

- ✅ **T-002 · Shell opens on Linux + `oklch()` gate** — effort: **low** *(Matt's machine)*
  `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
  librsvg2-dev`, then `cd client && pnpm tauri dev`. Verify the Console frame
  renders. Then the SPEC §5.4 M0 gate: temporarily set a token to an `oklch()`
  literal and confirm this WebKitGTK renders it; note the result here. If it does
  not render, M6 ships the generated hex fallbacks only (they already exist:
  `linger-core::palette::PaletteColor::hex`).
  *Accept:* screenshot of the frame; a one-line note "oklch: yes/no" added here.

  ### **oklch: YES** — the gate is closed, M6 can use `oklch()` directly.

  *Done 2026-08-19, empirically, against the exact runtime Tauri will use:
  **WebKitGTK 2.52.3** (Ubuntu 26.04's `libwebkit2gtk-4.1-0`).
  `CSS.supports('color','oklch(0.76 0.13 255)')` → **true**; the 92° two-stop
  gradient form from SPEC §4.5 → **true**; palette values applied over an
  `rgb(1,2,3)` sentinel replaced it, so the declarations are genuinely honoured
  (computed style serialises them back as `oklch(…)`, per CSS Color 4).*

  *Bonus finding, worth more than the gate itself: all 16 keys × 2 themes were
  rendered by WebKit into a canvas and read back, then compared with
  `PaletteColor::hex()`. **They agree exactly — 0/255 per channel.** That
  validates the CI contrast property test, whose ≥4.5:1 ratios are computed from
  that same conversion; a wrong conversion would have meant the guard was
  guarding the wrong numbers. Noted in `palette.rs` beside the test.*

  *Method note (no sudo on this box): the runtime was staged with `apt download`
  + `dpkg -x` and mounted over `/usr/lib/x86_64-linux-gnu` via an unprivileged
  overlay inside `unshare -rm` — WebKitGTK hardcodes its helper-process path and
  honours no override. Nothing outside that namespace was touched, so **the apt
  install below is still un-run.***

  **Visual half also done** (Matt ran the apt line 2026-08-19; deps confirmed
  present at webkit2gtk-4.1 **2.52.3**, the same build the gate was tested
  against). `pnpm tauri dev` compiled the shell in 1m35s and the window opened;
  screenshot at [`docs/m0-shell-linux.png`](docs/m0-shell-linux.png). The frame
  matches SPEC §5: three hairline-separated panels, cool neutral grays, mono
  confined to metadata (panel labels, status bar) with sans body text, no
  avatars, no bubbles, no rounded panels, permanent status bar.

  *Gotcha worth remembering: there is no standalone `pnpm` on this box — it
  comes from Ubuntu's corepack. A shim now exists at `~/.local/bin/pnpm`
  (created with `corepack enable --install-directory ~/.local/bin`), which
  `~/.profile` puts on PATH for **new** shells. It only works **inside the
  repo**, where `package.json` pins pnpm 9.15.9; run bare `pnpm` from `$HOME`
  and corepack fetches pnpm 11, which Ubuntu's corepack cannot execute
  (`ERR_VM_DYNAMIC_IMPORT_CALLBACK_MISSING`). Not a project bug.*

- ✅ **T-003 · Shell opens on Windows and macOS** — effort: **low** *(needs hardware)*
  Same check as T-002 per OS. Can trail other work; must close before M7 starts.
  *Done 2026-08-19 as far as is possible without the hardware: the new
  `desktop.yml` workflow **builds the shell on `windows-latest` (6m54s) and
  `macos-latest` (1m10s)** and both are green. This is `cargo build`, not
  `check` — linking is where platform breakage actually shows, and the macOS run
  really did compile the platform-specific crates (`tao`, `muda`,
  `window-vibrancy`, `embed_plist`). So: **the shell compiles and links on all
  three OSes.***

  *⚠️ What CI cannot do is watch a window appear. The visual "it opens and the
  Console frame renders" sign-off on Windows/macOS still needs someone in front
  of those machines, and must happen before M7. Same for T-002's Linux
  screenshot.*

  *Cost note: `desktop.yml` is **manual-trigger only** (`gh workflow run
  desktop.yml`). This repo is private, so Actions minutes are metered and
  non-Linux runners bill at a multiplier (Windows 2×, macOS 10×) — that one run
  cost ~14 Windows-minutes and ~18 macOS-minutes of the monthly allowance.
  Running it per-push would drain it for no benefit; WebKitGTK in `ci.yml` is
  the strictest engine and catches ordinary breakage. Trigger this before a
  release or after touching `client/src-tauri`.*

- ✅ **T-004 · Windows activity spike** — effort: **low** *(needs hardware, throwaway)*
  Console binary: `GetForegroundWindow → GetWindowThreadProcessId →
  QueryFullProcessImageNameW`, print exe name 1/sec. One evening, per
  ARCHITECTURE §6. Keep learnings in a note under this task, then delete the code.

  ### **The Windows path is retired — it works, first try.**

  *Done 2026-08-19. Written against `windows` 0.62, compile-checked locally for
  `x86_64-pc-windows-msvc`, then **run on a real Windows runner**. Output:*

  ```
  self-check: pid=6928 → exe_name="spike-win32" ... [PASS]
  [ 0] exe_name="windowsterminal" pid=9020
       exe_path="C:\Program Files\WindowsApps\...\WindowsTerminal.exe"
  pid → exe resolution: WORKS
  foreground window seen: yes — full pipeline verified
  ```

  *The resolved name `"windowsterminal"` is **already an `exe` alias on the
  `terminal` entry in `registry/apps.json`**, so the pipeline works end to end —
  foreground window → pid → exe path → normalize → registry hit — with no
  Windows-specific special casing. ARCHITECTURE §6 called this platform "easy"
  and it was; **T-913 is a transcription job, not a research job.***

  *Two things worth keeping: `PROCESS_QUERY_LIMITED_INFORMATION` (not
  `PROCESS_QUERY_INFORMATION`) is what makes elevated processes resolve, and
  since windows 0.58 handles wrap a pointer so the empty-desktop check is
  `hwnd.0.is_null()`. The spike deliberately carried a "no foreground window"
  path because a CI session might have had no desktop — it turned out to have
  one, but that path is real on a locked screen and the backend needs it.*

  *Code deleted as the task requires; the full recipe lives in
  `crates/linger-activity/src/backend.rs` next to the KWin one.*

- ✅ **T-005 · Confirm the 12 reaction keys** — effort: **low** *(Matt decision)*
  `linger-core::REACTIONS` is provisionally `heart laugh wow cry fire skull up
  down eyes clap hundred sparkles`. Confirm or edit *before* M3 renders them;
  changing keys after messages exist means a migration.
  *Done 2026-08-19: Matt confirmed the set as-is. AGPL-3.0 license also confirmed.*

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

## M1 — server REST: auth, invites, rooms, messages

*Milestone check: integration test suite drives the full REST surface with real
HTTP. No UI. Every endpoint gets an integration test (AGENTS.md).*

- ✅ **T-101 · Auth foundation** — effort: **high**
  PROTOCOL §2, ARCHITECTURE §7. Add to `linger-server`: `argon2` (argon2id,
  `m=19456, t=2, p=1` minimum), `jsonwebtoken` (EdDSA — generate the Ed25519
  keypair at first boot, store under the data dir, 0600), `rand`, `sha2`.
  Endpoints: register (invite-gated), login, refresh, logout. Refresh tokens:
  opaque 256-bit, stored as sha256, 30-day, **rotating; reuse of a consumed token
  revokes the whole family** (track family via `refresh_tokens.id` chain or a
  family column — your call, test it). Bearer-auth extractor for protected routes.
  Password min 12 chars, **no composition rules**. Username `[a-z0-9_]{2,24}`,
  immutable. Login rate limit 5/min/IP.
  *Accept:* integration tests for happy paths + wrong password + expired access +
  refresh rotation + **reuse-revokes-family** + rate limit envelope
  (`RATE_LIMITED` + `retry_after_ms`).

- ✅ **T-102 · First-run host setup** — effort: **medium**
  ARCHITECTURE §9. On boot with zero users: generate a one-time setup token,
  print `https://<domain>/setup?token=…` to stdout (http only when there is no
  `LINGER_DOMAIN` and we are printing a bare bind address — the client keeps
  whatever scheme it is handed, so this decides whether the host's own session
  runs over TLS; corrected 2026-08-21), expose endpoints to
  create the host account + name the server (writes `server_config`). Token dies on
  use or restart. No env-var bootstrap credentials.
  *Accept:* integration test boots a fresh server, completes setup, second attempt
  fails, `GET /server` returns the name.

- ✅ **T-103 · Invites** — effort: **medium**
  PROTOCOL §7 + §2. CRUD per protocol; codes 12 chars base32 from a CSPRNG,
  single-use default; unauthenticated preview endpoint; register consumes a use
  atomically (single-writer pool makes this easy — one UPDATE … WHERE guards).
  Rate limit 10/day/user. Host-or-creator may revoke.
  *Accept:* tests for expiry, max_uses exhaustion, revocation, preview of each
  state, and the register-through-invite flow.

- ✅ **T-104 · Server + rooms endpoints** — effort: **medium**
  PROTOCOL §3. `GET/PATCH /server` (PATCH host-only; `accent_key` validated
  against `linger-core::PALETTE`). Rooms CRUD: create/patch/archive host-only,
  slug `[a-z0-9-]{1,32}` unique, `position` ordering, `last_message_id` filled
  from a join. Vocabulary check: it's `RoomId` and "room" in every string.
  *Accept:* tests incl. non-host 403s (`FORBIDDEN`), bad slug (`VALIDATION_FAILED`),
  bad palette key rejected.

- ✅ **T-105 · Messages, reactions, read markers** — effort: **high**
  PROTOCOL §4. Create/edit(author-only)/delete(author-or-host → tombstone: body
  `""`, `deleted_at` set, row kept)/pin/unpin. Pagination: `before`/`after` are
  message ids; UUIDv7 blob comparison gives the range scan; `limit` 1..100,
  newest-first. Reactions: PUT/DELETE, key must be in `linger-core::REACTIONS`
  else `VALIDATION_FAILED`. Read markers: `PUT /rooms/:id/read`, `GET /read`.
  **No count field anywhere — grep the diff for `unread` before finishing.**
  Message rate limit 10/10s/user. Attachments array wiring lands in M5; return
  `[]` for now.
  *Accept:* tests for pagination edges (empty room, exact-limit boundary,
  before+after), tombstone reply chains, reaction validation, marker idempotency,
  author/host permission matrix.

- ✅ **T-106 · Users, styling, statuses** — effort: **medium**
  PROTOCOL §5. `GET /users`, `GET /users/:id`, `GET /me`, `PATCH /me`
  (display_name 1–32; `style` — **server-side validation** of `font_key`/
  `msg_font_key` ∈ `FONTS`, fill color keys ∈ `PALETTE`, weight ∈ {400,500,700};
  `sign` field caps from `linger-core::limits`; `entrance_sound` bundled-key only
  until M4), `PATCH /me/password` (verifies current). Notify rules endpoints
  (person-centric, SPEC §4.2 — no keyword rules).
  *Accept:* tests proving every invalid key is rejected server-side with
  `VALIDATION_FAILED`; style round-trips into `user_style` columns and back.

- ✅ **T-107 · Rate-limit plumbing** — effort: **medium**
  One reusable keyed token-bucket (in-memory, `DashMap`) used by T-101/103/105
  constants in `linger-core::limits`. 429 envelope with `retry_after_ms` set.
  *Accept:* unit tests for bucket math + one integration test per limited surface.

## M2 — gateway

*Milestone check: a test client survives a forced mid-stream disconnect and
replays without gaps or duplicates. This is a flagged "you will be wrong" area:
test with real disconnects, not mocks.*

- ✅ **T-201 · WS gateway core** — effort: **high**
  PROTOCOL §8. Upgrade at `/api/v1/gateway`; `hello` (30000ms) → `identify`
  (JWT) → `ready` (full `ReadyData`). Heartbeat/ack; two missed acks server-side
  → drop. Presence lives in `DashMap<UserId, PresenceEntry>` — **never persisted**.
  Fan-out: one `tokio::sync::broadcast` bus of `ServerFrame`s; per-connection task
  filters per PROTOCOL §8 fan-out rules (only `room.enter` is filtered in V1).
  Sequence numbers are per-session, assigned at send.
  *Accept:* integration test connects two clients, sends a message over REST,
  both receive `message.create` with correct `s` ordering.

- ✅ **T-202 · Resume** — effort: **high**
  Per-session ring buffer (500 frames, 120s post-disconnect retention), `resume`
  → `resumed {replayed}` + replay, else `invalid_session` → client re-identifies.
  *Accept:* the AGENTS.md-mandated test: force-drop the socket mid-stream while
  messages keep flowing, resume, assert **no gaps and no duplicates** by sequence
  number; second test exceeds the window and asserts `invalid_session`.

- ✅ **T-203 · Presence ops + rooms occupancy** — effort: **medium**
  `presence.update` (client), `room.focus`, `room.occupancy`, `room.enter`
  (with entrance_sound key, only to those in the room), `room.leave`, `typing` (1/4s/room
  server-enforced), `user.update`/`room.*` fan-out from REST mutations.
  *Accept:* two-client test: A focuses a room, B in that same room receives
  `room.enter`; B in another room does not. Disconnect marks offline.

## M3 — client: message stream

*Milestone check: two clients on one machine exchange messages in real time.*

- ✅ **T-301 · API client + auth flow** — effort: **medium**
  Typed fetch wrapper over generated types (no `any`, no `as` across the wire);
  login/register screens (invite link paste); token refresh on 401; tokens in OS
  keyring via a Tauri command (`keyring` crate) — **test the headless/no-wallet
  fallback: clear re-login prompt, not a crash** (ARCHITECTURE §7.3).
  *Accept:* manual: login on a fresh profile, restart app, still authed.

  ### **Done 2026-08-19.** Both accept paths were run for real, on this box.

  *Signed in against a live `linger-server`, killed the app, started it again —
  it came back signed in with no typing. Screenshot at
  [`docs/t301-signed-in.png`](docs/t301-signed-in.png). Proof it was the real
  round trip and not a cached screen: the refresh token in the keyring changed
  from the one seeded before launch to a new one, which only happens if the app
  read it, spent it at `/auth/refresh`, and wrote the replacement back.*

  *The no-wallet case was run too, by starting the app with the D-Bus session
  address pointed at nothing — the same as a headless box or a locked wallet.
  The app opens normally, shows the sign-in box, and says "No usable keyring on
  this computer… You'll have to sign in again next time." No crash, no hang.*

  ### ⚠️ **The big one: the server had to learn CORS.**

  *A Tauri window is a web page, so every call it makes to the server is
  cross-origin, and a browser refuses to hand the page a response without
  permission headers. Without this the client cannot reach the server **at
  all** — the first live run failed with "Couldn't reach…" while `curl` against
  the same URL worked fine. `linger-server` now sends CORS headers for a fixed
  list of four origins (`tauri://localhost`, `http(s)://tauri.localhost` for
  Windows, and Vite's `http://localhost:1420` for `pnpm dev`), enforced by a
  test in `tests/health.rs`. It is a fixed list rather than a wildcard on
  purpose: with a wildcard, any website you visited could quietly check whether
  this server exists.*

  *Two smaller server-side changes came out of the same run. A wrong password
  answered "Sign in to do that.", which is nonsense under a login form — it now
  says "That username and password don't match." And the client's CSP now allows
  `http://localhost` and `http://127.0.0.1`, so a server running on your own
  machine works; plain-http LAN addresses are still refused, which is deliberate,
  since that would be a password over the wire in the clear.*

  ### What got built

  - `client/src/lib/api.ts` — every call, typed off `src/generated/`. There is
    no `as` cast anywhere in it: error codes are checked against a list that
    `satisfies Record<ErrorCode, ErrorCode>`, so if `linger-core` ever adds or
    renames a code, `pnpm check` fails. Verified by deleting a code and watching
    it fail. A refused call comes back as `ApiError` (the server talking) or
    `TransportError` (no answer at all) — the screens show `error.message`
    directly, which is what PROTOCOL §1 promises it is for.
  - `AuthedApi` handles the 15-minute access token expiring mid-use: it
    refreshes and retries once. Two calls failing at the same time share **one**
    refresh, because spending a refresh token twice revokes the whole family and
    would log you out.
  - `client/src-tauri/src/secrets.rs` — three Tauri commands over the `keyring`
    crate. **None of them can fail**: "there is no keyring here" is a value the
    UI renders, not an error that blows up the IPC call. Only the refresh token
    is stored; the access token is not, since it dies in 15 minutes anyway.
  - One paste box instead of three screens. It takes a setup link, an invite
    link, or a bare address, and picks the right form. The link shapes are now
    written down in PROTOCOL §2.2 and in the README, since nothing serves those
    paths — they only exist for a person to paste.

  ### Notes for whoever is next

  - *`keyring` is configured per platform: pure-Rust zbus on Linux
    (`async-secret-service` + `crypto-rust`), so **no new system dependency**.
    The kernel-keyutils backend is deliberately off — it would "work" on a
    headless box and then lose the secret at logout, hiding the exact case
    ARCHITECTURE §7.3 asks us to handle honestly.*
  - *Restoring the session runs **once per launch**, guarded by a
    module-level promise. React StrictMode double-mounts effects in development,
    which would spend the refresh token twice and revoke the family — a bug that
    would only ever appear on a dev machine.*
  - *`client/src-tauri` has real tests now (`cd client/src-tauri && cargo test`).
    `cargo test --workspace` does not run them. One needs a real desktop session
    and is marked `#[ignore]`; run it with `cargo test -- --ignored`.*
  - *There is still **no test runner on the frontend**. The API client and the
    link parser were checked by driving them from Node against a live server
    (26 assertions, all passing) — a throwaway harness, not committed. If the TS
    side grows much past this, it wants vitest; that is a decision, not a task
    yet.*
  - *The generated TypeScript did not drift, so nothing in
    `client/src/generated/` changed.*

- ✅ **T-302 · Gateway client in Rust core** — effort: **high**
  ARCHITECTURE §1: the WS client lives in the Tauri core, not the WebView.
  Connect/identify/heartbeat/resume/backoff(jittered, capped); emits Tauri events
  to the frontend; one Zustand-free store on the TS side (AGENTS: local state +
  one gateway store). Status bar shows protocol text states:
  `connecting… tls ok… identify… ready (28ms)`.
  *Accept:* kill the server, restart it, client resumes or re-identifies with no
  user action; status text follows the states.

  ### **Done 2026-08-19.** The accept path was run for real, twice over.

  *A live `linger-server` was SIGKILLed under a connected client. The status bar
  went `ready (0ms)` → `retry in 7s…` → `ready (4ms)` with nothing touched:
  screenshots at [`docs/t302-connected.png`](docs/t302-connected.png) and
  [`docs/t302-status-states.png`](docs/t302-status-states.png). Under the hood the
  client tried `resume` first, got `invalid_session` from the restarted process,
  and re-identified immediately.*

  ### **Resume was proved against the real server, not just a fake one.**

  *The harder half of the accept criterion is resume that actually resumes, which
  a killed server can never show — its sessions die with it. So the client was
  pointed at a `socat` proxy in front of a live server, and the proxy was killed
  mid-stream while messages kept being posted over REST. The client reconnected,
  sent `resume` with its last sequence number, and the server replayed. The
  frontend saw **s = 0,1,2,3,4,5,6,7,8,9 — each exactly once, in order**, with
  the three messages sent during the outage arriving in the replay. No gaps, no
  duplicates, against real `linger-server` code.*

  ### What got built

  - `client/src-tauri/src/gateway.rs` — the whole connection. It knows nothing
    about Tauri: status changes and frames go out through an `Events` trait that
    `lib.rs` implements with `AppHandle::emit` and the tests implement with a
    channel. That is what makes it testable over a real socket.
  - `client/src-tauri/tests/gateway_client.rs` — 11 tests, each standing up an
    actual TCP listener and speaking the protocol at the client. The disconnects
    are RSTs (`set_linger(0)`), not polite closes, per AGENTS' "test with real
    disconnects, not mocks".
  - `client/src/lib/gateway.ts` — the one store, ~80 lines of subscribe-and-notify
    behind `useSyncExternalStore`. No state library, per AGENTS.
  - The status bar is live, the rail lists real rooms, and the roster lists whoever
    is around. Both lists are deliberately plain — T-303 and T-401 replace them.

  ### The three rules this ended up resting on

  - ***Frames with a sequence number belong to the frontend; frames without one
    stay in Rust.*** `hello`, `heartbeat_ack`, `resumed`, `invalid_session` are
    connection plumbing and never reach the WebView. `ready` does, because it
    carries the roster and rooms.
  - ***The frontend owns tokens.*** The Rust side has no refresh token on purpose
    — two parties spending a rotating one revokes the family (PROTOCOL §2). When
    the server refuses a token, Rust emits `needs_token` and waits; the TS side
    answers through `AuthedApi.accessToken()`, which is single-flight. `AuthedApi`
    now tracks expiry, so `Tokens` gained an `expiresAt` field.
  - ***Believe the server about sequence numbers, but check.*** A replayed frame
    at or below the high-water mark is dropped, and a *gap* forces a full
    re-identify rather than being papered over — the same trade the server makes
    when its bus lags. Both have tests.

  ### Notes for whoever is next

  - *`connect_async` is one call for TCP + TLS + the WS handshake, so the status
    bar can't show a state between them. It shows `tls ok…` for `wss://` and
    `socket ok…` for a plain `ws://` server on your own machine — SPEC §5.6's
    example sequence, minus the lie about a TLS handshake that never happened.*
  - *`gateway_send(frame)` exists and is typed off the generated `ClientFrame`,
    but nothing calls it yet. T-402 (`room.focus`) and T-304 (`typing.start`) are
    its first users; a send while disconnected returns `false` rather than
    queueing, because stale presence is worse than none.*
  - *The store ignores `message.*` frames today — they arrive and are dropped on
    purpose rather than half-stored. T-303 adds them.*
  - *New deps in `client/src-tauri`: `tokio-tungstenite` + `rustls` (ring
    provider, OS trust store with the Mozilla bundle as backstop), `futures-util`,
    `rand`. rustls rather than native-tls so nobody needs libssl headers; ring
    rather than aws-lc-rs so nobody needs cmake or nasm. A WS client is required
    by ARCHITECTURE §1, so this is not a bundle-size judgment call — but it is
    ~1MB of binary, worth knowing before M7.*
  - *CI now **runs clippy and the tests** on the shell instead of just
    `cargo check`ing it (`tauri-shell` job). The root clippy job can't see this
    crate — it is outside the workspace on purpose — and clippy caught three real
    findings in this task's code, so the gap was worth closing. Costs a check
    pass plus a link step per run.*
  - *No frontend test runner still. The TS half here is small and was checked by
    watching the real app; if `client/src` grows much past this it wants vitest.*
  - *One local side effect of testing: this box's keyring now holds a session for
    a throwaway server at `http://127.0.0.1:8421` that no longer exists. The app
    will say it can't reach it on next launch — one "sign out" clears it.*

- ✅ **T-303 · The stream** — effort: **high**
  SPEC §4.7 + §5.6. Virtualized list **from day one**; author grouping (break
  10min); session breaks (3h) with natural-language dividers (`SATURDAY MORNING`
  mono small-caps); aging via one CSS custom property (body only, floor 78%);
  3px per-person gutter rule, no avatar column; density modes Comfortable/
  Compact/IRC (IRC: one line/message, mono, no grouping/aging/effects).
  *Accept:* milestone check runs here: two clients exchange live messages;
  scrollback of 10k messages stays smooth (virtualization proof).

  ### **Done 2026-08-20.** Both halves of the accept criterion were run for real.

  *Two `linger-client` windows on one machine, signed in as different people
  against a live `linger-server`, both looking at `#porch`. Matt typed, Callie's
  window showed it and followed to the bottom; Callie replied, Matt's window
  showed that. Neither window was touched in between — the messages arrived over
  the gateway socket, not from a refetch.
  [`docs/t303-two-clients.png`](docs/t303-two-clients.png) is the pair: Matt's
  window on the left with both messages, Callie's on the right at the moment
  Matt's arrived and pushed the view along.*

  *For the second half, `#garage` was seeded with 10,000 messages spread over a
  year (100 messages per HTTP round trip against a 10-per-10s send limit would
  have taken three hours, so the rows went straight into SQLite with UUIDv7 ids
  derived from their timestamps — the same shape the server writes). Scrolling
  to the start of history pulled all of it in. With **10,001 messages loaded and
  10,436 rows**, the list held **24–43 row elements in the DOM**, and the worst
  frame during hard continuous scrolling (20 page-downs a second for 18 seconds)
  was 28–38ms — the same as the worst frame sitting still in a 43-message room
  on this machine. Scrolling ten thousand messages costs nothing measurable over
  doing nothing. Screenshot at [`docs/t303-10k.png`](docs/t303-10k.png): the top
  of a year of history, with the counts on the line above the composer. **That
  readout was temporary instrumentation and is not in the code** — and the
  `worstframe` in that particular frame is 45ms because it was taken while the
  last page was still landing, which is the backfill cost noted below, not the
  scrolling cost.*

  ### What got built

  - `client/src/stream/time.ts` — session labels, clock times, aging steps. Pure,
    and takes `now` as an argument, because "yesterday afternoon" is only true
    relative to a moment and that moment has to be testable.
  - `client/src/stream/rows.ts` — messages in, rows out. Whether a message shows
    its author's name depends on the message before it, and a session divider is
    a row of its own; working that out in one place keeps the component down to
    "draw row N" and gives the virtualizer a stable key per row.
  - `client/src/stream/Stream.tsx` — the list, the header, the composer.
  - `client/src/lib/gateway.ts` — the store now holds each open room's history.
    History arrives two ways, as pages over REST and as frames over the socket,
    and the two have to be stitched in one place.
  - `client/src/lib/density.ts` — the preference. Local state, not a second
    store: every difference between the three modes is a custom property in
    `tokens.css`, except grouping, which CSS can't express.
  - **A test runner.** T-302's note said the TS side would want vitest once it
    grew; this is where it grew. 21 tests over the date arithmetic and the
    grouping boundaries — the parts that look right and are wrong at 2am, on a
    Sunday, or in March. CI runs `pnpm test`.

  ### The three rules this ended up resting on

  - ***Space between rows is padding, never margin.*** The virtualizer measures
    each row's own box and a margin sits outside it, so margin is spacing it
    cannot see. It shows up as a list that drifts while you scroll.
  - ***A row's height is a guess until it has been drawn once.*** Anything that
    scrolls to a position has to keep re-aiming as the real heights arrive.
    Landing on the newest message is a per-frame loop, not a jump: a frame is the
    beat because the browser reports a scroll asynchronously, so two jumps inside
    one frame means the second is aiming with the first one's stale numbers. The
    loop stops when the last row is actually drawn — *not* when the total size
    stops growing, which holds still for a frame all the time while measurements
    are still arriving. That wrong test cost an hour and landed the view a
    quarter of the way up the room.
  - ***A grid item will not shrink below its content unless told to.*** The
    stream sat in a `1fr` grid track with no `min-height: 0`, so a long room
    pushed the composer and the status bar off the bottom of the window. It only
    showed up once there was something long to render.

  ### Notes for whoever is next

  - *The composer sends plain text and nothing else. Markdown, the sanitizer,
    edit/delete/reply and reactions are **T-304** — but the accept criterion here
    is two clients exchanging messages, so there had to be a way to say
    something. It is an `<input>` and T-304 should replace it outright.*
  - *Reaction frames are stored but nothing renders them. Same reason the store
    holds a message's `reactions` at all: dropping `reaction.update` would let
    our copy drift from the server's. Rendering weight is T-304.*
  - *The per-person gutter rule and the author's name both point at
    `var(--name-<key>, <neutral>)`. **M6 generates those variables** from
    `linger-core::PALETTE` into `palette.generated.css`; until it does, both fall
    back to something neutral and the stream lights up the moment that file
    exists. Nothing in `client/` knows what "azure" looks like, and nothing
    should. Font keys and name effects are M6's too, so a styled name currently
    gets weight and italic only.*
  - *`ready` now clears loaded history. A fresh `ready` means the client had to
    re-identify, which means the resume window lapsed, which means there may be
    messages it never saw — and a hole in the middle of a room is invisible in a
    way an empty room is not. The open room refetches; it costs one request and a
    scroll position.*
  - *A backfill lands 100 messages at once, which costs one long frame (~45–65ms)
    while the row list and the measurement cache are rebuilt. It is a hitch when
    a page arrives, not while scrolling. If it ever needs fixing, the O(N) passes
    are `buildRows` and the virtualizer's `getMeasurements`.*
  - *Attachments render as nothing. There is no way to make one yet — uploads are
    M5 — so there is nothing to render, but the branch will need writing.*
  - *New dependency: `@tanstack/react-virtual` (~600KB of source, a few KB in the
    bundle; total build is 245KB / 77KB gzipped, well inside AGENTS' 2MB rule).
    Hand-rolling variable-height virtualization with anchored prepends is the
    kind of code AGENTS' "where you will be wrong" table is about, and this
    version has the chat case built in: `anchorTo: "end"` keeps your place when
    history loads above you, `followOnAppend` follows a new message only when you
    are already at the bottom.*
  - *`SPEC.md` §4.7 and §5.6 disagreed about aging: §4.7 said the steps were
    100/88/76/66, §5.6 said 100/88/78 with a floor at 78. The task text says
    floor 78%, so §5.6 won and §4.7 now points at it instead of restating
    different numbers. **If the four-step version was the intended one, this is
    the line to change back** — it is one constant in `stream/time.ts` and one
    test.*
  - *The scrollback region takes `tabIndex={0}` so it can be scrolled without a
    mouse. That adds a tab stop between the density picker and the composer.*
  - *Testing needed the GUI driven and this box has no xdotool (Wayland). A
    virtual keyboard over `/dev/uinput` worked; the scaffolding is throwaway and
    is not in the repo. Worth knowing: keyboard scrolling in WebKitGTK is
    animated, so synthetic key presses closer together than ~300ms coalesce and
    scroll far less than a page each.*

- ✅ **T-304 · Composer + message actions** — effort: **medium**
  Markdown (allowlist sanitizer, **no raw HTML passthrough**), send affordance in
  accent, edit/delete/reply, reactions by **weight** (denser/larger mark, count
  only in hover/aria — never rendered as a number).
  *Accept:* XSS attempt (`<img onerror>` etc.) renders inert; reactions show
  weight not numbers.

  ### **Done 2026-08-20.** Both accept criteria hold, and both were checked.

  *The XSS one is checked by test rather than by eye, on purpose. `Markdown` is
  rendered to the exact markup a browser would be handed
  (`react-dom/server`), and two assertions run over that string: every element
  in it has to come from a list of eleven, and every attribute from a list of
  five. A message body that could contribute one element or one attribute fails
  them, whatever the trick was — which is a stronger claim than any single
  screenshot of `<img onerror>` not firing. It was also checked by eye: a room
  seeded with `<img src=x onerror=…>`, `<script>fetch(…)</script>` and
  `[click me](javascript:alert(1))` renders all three as the characters that
  were typed. [`docs/t304-stream.png`](docs/t304-stream.png).*

  *The weight one was checked by eye, because it is a visual claim. One room,
  eight accounts, the same mark at 1, 2, 3 and 8 reactors: the marks step up in
  size and the fill gets more solid, and **no numeral is drawn anywhere**.
  [`docs/t304-reactions.png`](docs/t304-reactions.png) has the twelve-mark strip
  open above a message carrying marks of three different weights.*

  ### What got built

  - `client/src/stream/markdown.ts` — the parser. Source in, a tree of known
    node kinds out. **This is the sanitizer**, and it is worth being clear about
    why: there is no HTML string anywhere between a message body and the
    screen, so there is nothing to sanitize. `Markdown.tsx` switches over the
    node union and hands text to React as text. An allowlist you cannot get
    past beats a denylist you have to keep ahead of.
  - `client/src/stream/Markdown.tsx` — one element per node kind, and the only
    place a body reaches an attribute is a link's `href`, which only exists if
    `safeHref` built it.
  - `client/src/stream/reactions.ts` — the twelve keys, their glyphs, and the
    weight curve.
  - `client/src/lib/external.ts` + `tauri-plugin-opener` — links go to the
    system browser. See the note below; this is the one judgment call in the
    task.
  - `client/src/stream/Stream.tsx` — the composer is a textarea now, and each
    message grew a hover strip: react / reply / edit / delete.
  - `client/src/lib/gateway.ts` — `editMessage`, `deleteMessage`,
    `toggleReaction`, `reply_to` on send, and the typing map.
  - 48 more tests (69 total). `vitest.config.ts` now includes `.tsx`.

  ### Decisions worth knowing about

  - ***The markdown subset is small and closed.*** Bold, italic,
    strikethrough, inline code, fenced code, blockquotes, bullet and numbered
    lists, links, backslash escapes. **No headings, no tables, no images, no
    raw HTML.** Headings are shouting in a chat window and there is nothing to
    point an image at until uploads land in M5. It is not CommonMark and does
    not try to be — CommonMark's emphasis rules are a specification unto
    themselves. The rule followed instead is that anything ambiguous stays
    literal: a `*` with no partner is a `*`.
  - ***`snake_case_name` and `2 * 3 * 4` stay literal*** — word-boundary rules
    on underscore, and no span opens or closes on whitespace. `__init__` does
    *not*: with no word character on either end there is nothing to hold the
    delimiters down, and every markdown anyone has used bolds it. Backticks are
    the way to write it, and a dunder name belongs in backticks anyway.
  - ***Mono appears for code, and only for code.*** AGENTS 11 says mono in a
    message body is a defect; SPEC §5.2 lists code beside timestamps and
    numerals as one of mono's roles. Prose stays sans. There is a test that
    counts the mono classes in a rendered body so the exception cannot spread.
  - ***Reaction weight is logarithmic.*** One to two is the step that means
    something — somebody agreed. Nine to ten means almost nothing. A linear
    ramp would spend its whole range on the part nobody cares about. Eight
    reactors is full weight, which is more than everyone in the kind of room
    this is for.
  - ***Delete asks twice, in the strip itself.*** `delete` becomes `delete for
    good` / `keep`. No modal, no layer, and the row does not change height.
  - ***The twelve-mark palette takes over the action strip rather than opening
    a popover.*** A popover inside a virtualized row is either clipped by the
    scroll container or it changes the row's height, and both are worse than
    swapping what the strip contains.

  ### The one judgment call: `tauri-plugin-opener`

  *A link in a message has to go somewhere, and a WebView that navigates itself
  to a URL out of a chat message has replaced the app with a website, session
  and all. So the click is taken and handed to the system browser. That needs a
  plugin — every platform opens a URL differently and Windows is the one that
  is easy to get subtly wrong, which is AGENTS' "where you will be wrong" table
  exactly.*

  *It does widen the WebView's permission surface, which ARCHITECTURE §7 says
  to keep minimal, so it is scoped: `capabilities/default.json` allows
  `opener:allow-open-url` for `http://*` and `https://*` only, enforced in
  Rust. No `file://`, no launching a program. Two locks: `safeHref` decides
  what can become a link at all, and the capability decides what can be
  opened.*

  *SPEC §4.7's "restrained embeds" — a one-line card with favicon, title and
  domain — is **not** built and is not in any task. It would mean the client
  fetching a remote page, which is a privacy question worth asking on purpose
  rather than sliding into.*

  ### Two bugs this found in T-303's work

  - ***A new message did not follow the view down if it was tall.*** The
    virtualizer's `followOnAppend` aims at a bottom computed from estimated row
    heights, and a row is an estimate until it has been drawn once. Before
    markdown every message was a line or two and the error was a few pixels; a
    message with a code block in it is off by a screenful. Fixed with the same
    per-frame re-aim the room-entry landing uses, keyed on the *last* row's key
    so a page of older history loading in above never triggers it.
  - ***The in-place edit box collapsed to one line.*** It borrows
    `.composer-input`, which carries `flex: 1` so it fills the composer's
    *row* — inside the edit form's column that means `flex-basis: 0`, which
    beats the height the auto-grow measures. An eight-line message you can only
    edit through a slot is not editable.

  ### Notes for whoever is next

  - *`typing.start` is sent and rendered.* T-302's note nominated T-304 as its
    first user and the composer was being rewritten anyway, so both halves
    landed: throttled at 4s to match `RATE_TYPING_PER_ROOM`, and a line above
    the composer that holds its height whether or not anyone is typing. Nobody
    sends a "stopped" — the signal goes stale after 6s and `typistsIn` is what
    decides. Verified with a second gateway client signed in as another
    account. **T-402 still owns `room.focus`.**
  - *The action strip's buttons are in the tab order.* That is a real cost —
    tabbing from the scrollback to the composer now passes through two to four
    buttons per rendered row (~25–45 rows exist at a time). The alternative was
    keyboard users being unable to react or reply at all, which is worse. Up
    arrow in an empty composer edits your last message, which is the habit from
    every other client and the quick way to reach `edit`. If this needs
    revisiting, the fix is a roving tabindex driven by which row is active.
  - *`AuthedApi` gained `put()`* for the endpoints that answer 204 and say
    nothing, which is both reaction routes.
  - *A reaction is applied locally before the server answers*, because the mark
    should move under the cursor rather than a round trip later. The
    `reaction.update` frame is the truth and usually beats the HTTP response
    back; a refusal puts the old group back, which only matters when the socket
    is down.
  - ***The twelve reaction keys are confirmed*** (Matt, 2026-08-20): heart,
    laugh, wow, cry, fire, skull, up, down, eyes, clap, hundred, sparkles. The
    "provisional curation" note in `linger-core` is gone. They are written into
    `reactions` rows from here on, so **changing one is a migration, not an
    edit** — adding a thirteenth is the safe direction, because a client skips a
    key it does not know rather than guessing. A test reads the Rust constant
    and fails if `reactions.ts` drifts from it.
  - *`MAX_MESSAGE_CHARS` is written out in `Stream.tsx` as 8000.* ts-rs exports
    types, not constants, so there is no generated home for it. The server is
    still the authority and refuses anything longer; the copy exists so the
    composer can say so before the round trip.
  - *`@types/node` is now a declared devDependency.* It was already there as a
    hoisted phantom — the reaction drift test reads a file, which made the
    dependency real and pnpm noticed.
  - *Attachments still render as nothing*, and link embeds do not exist. Both
    are still waiting on M5.
  - *Testing drove a real desktop over `/dev/uinput`* again (Wayland, no
    xdotool), plus KWin scripting to place and confirm the window before every
    screenshot. That scaffolding is throwaway and is not in the repo. Two
    things worth writing down: **`pkill -f <pattern>` also matches the shell
    running it** — CLAUDE.md warns about this for `tauri dev` and it is just as
    true for a test script's own name, so kill by recorded PID; and **a uinput
    pointer device must stay open while you screenshot**, because tearing it
    down drops the hover you were trying to capture.

- ✅ **T-305 · "You left off here"** — effort: **medium**
  SPEC §4.2. Accent divider at last-read, persists for the session; room label
  weight change (60%→100% opacity, nothing else); "since you were gone" pulled
  from the room header; mention notifications (person-to-person only — there is
  no `@everyone` to implement); notify-rules settings UI.
  *Accept:* manual two-client script in the task notes; grep the diff for badge/
  count regressions.

  ### **Done 2026-08-21.** M3 is finished.

  ### The two-client script that was run

  *A clean server on `127.0.0.1:8421` from an empty data directory, two
  `linger-client` windows side by side on one machine, signed in as different
  people (Matt is the host, Callie joined on an invite), two rooms `#garage` and
  `#porch`, and a handful of messages already in `#garage`. Every step below was
  run for real and checked.*

  1. *Matt sits in `#garage` with the window focused, then clicks `#porch`.*
     `GET /read` on the server now answers with the id of the last message in
     `#garage` — a position, not a number, and the only shape that endpoint has.
  2. *Callie says three things in `#garage`, the last one `@matt come and look
     when you can`.* Matt is in `#porch`, so nothing on his screen changes except
     the weight of the `#garage` label in the rail.
  3. *A desktop notification appears on Matt's machine.* Checked on the bus
     rather than by eye, because a screenshot of a notification is a race:
     `busctl --user monitor org.freedesktop.Notifications` catches
     `Notify("linger-client", "Callie in #garage", "@matt the kettle is on")`.
     **An ordinary message in the same room produced no `Notify` call at all**,
     and neither did a mention that arrived while Matt was looking straight at
     `#garage` with the window focused.
  4. *The label weight was measured rather than eyeballed*, because 60% of a
     grey is not obviously different from 100% of it in a screenshot. The
     `#garage` label's brightest pixels read `(143,150,162)` with something
     unseen in the room and `(98,104,113)` once it had been read — the same
     colour at full and at 60%, which is `--text-secondary` `#8b929e` over the
     rail. No dot appeared, no number, no colour change.
  5. *Matt clicks back into `#garage`.* `YOU LEFT OFF HERE` is drawn in accent
     above the first message he had not seen, the two messages naming him carry a
     raised background, and `@matt` renders as a mark rather than as four
     characters. [`docs/t305-left-off.png`](docs/t305-left-off.png).
  6. *He pulls `since you were gone` from the room header.* It says who has
     spoken and when it started — `Callie · this afternoon` — and offers `go to
     where you left off`, which scrolls to the line. It never says how much.
  7. *`notify` in the roster, expand Callie, tick `#porch`.* The rule is on the
     server immediately (`GET /me/notify-rules` →
     `{target_user_id: callie, room_id: porch}`), and Callie's next message in
     `#porch` — **with nobody named in it** — raised
     `Notify(…, "Callie in #porch", "no name in this one at all")` while Matt was
     sitting in `#garage`. [`docs/t305-notify-rules.png`](docs/t305-notify-rules.png).

  *The badge/count grep is clean. Every hit for `badge`, `unread`, `count` or
  `tally` in the diff is a comment saying there isn't one; no line of code
  computes a difference between two message ids, and nothing new renders a
  numeral. The two numerals that do exist in the stream are older and sanctioned:
  a reaction's tally in its `aria-label` (SPEC §4.8) and "N people are typing".*

  ### What got built

  - `client/src/lib/gateway.ts` — the store now holds four more things: where you
    have read to in each room (`read`), the newest message that exists in each
    room (`newest`), where the line is pinned (`leftOff`), and your notify rules.
    Plus `markRead`, `loadUntil`, `enterRoom`, and the rule endpoints.
  - `client/src/stream/rows.ts` — the "you left off here" line is a row, the same
    way a session divider is.
  - `client/src/stream/markdown.ts` — a `mention` node, and `mentionHandles`.
  - `client/src/stream/Markdown.tsx` — draws a mention, or doesn't.
  - `client/src/notify/rules.ts` — the whole list of reasons anything is allowed
    to interrupt you. Pure, and tested, so the list can't quietly grow.
  - `client/src/notify/notify.ts` — the batching and the OS call.
  - `client/src/notify/NotifyRules.tsx` — the settings panel, in the roster
    column.
  - `client/src/App.tsx` — the label weight, and the roster column's second mode.
  - 34 more tests (103 total).

  ### Decisions worth knowing about

  - ***The line is pinned when you walk into a room, and never moves while you
    are standing in it.*** That is the reading of SPEC §4.2 that survives
    contact: a line that crept along as you read would be a line you could never
    use to find your way back, and a line pinned once at app start would be
    telling you about this morning by the evening. So it is per visit. Leaving
    the room and coming back pins it again, which is what makes it mean anything
    the second time.
  - ***The line is only drawn when the client holds the message on both sides of
    it.*** Otherwise it would sit at the top of a page of history and claim that
    is where you stopped, when really it is where the last fetch happened to end.
    Pulling "since you were gone" is what reaches further back — up to ten pages,
    and then it stops, because a line a thousand messages up is not somewhere
    anyone is scrolling to.
  - ***"Since you were gone" says who and when, never how much.*** A count there
    would be the badge in a coat, and it would also be the one dishonest number
    in the app, since the client only holds the pages it has fetched.
  - ***A mention is matched on the username, not the display name.*** The
    username is what is unique on a server and what somebody actually typed.
    `@matthews` is not a mention of `matt` — the pattern would happily stop short
    otherwise, and a notification landing on the wrong person's name is worse
    than no notification. `@Matt` is not one either: usernames are lowercase and
    the server rejects rather than normalises, so that is a word, not a name.
  - ***A mention is found by walking the parsed tree, not by scanning the
    text.*** So `` `@matt` `` in backticks is code, and does not ring anybody's
    phone. `@` also joined the list of characters a backslash makes literal.
  - ***A mention is weight, not colour.*** Names are painted from the palette
    everywhere else, but doing it inside a message body would mean a value out of
    somebody's profile reaching into the body's markup — and the body is the one
    place this client keeps free of attributes it did not choose itself. Accent
    was out for a different reason: it has four jobs (SPEC §5.3) and the "you
    left off here" line is the one it spends here.
  - ***The first message under the line shows its author's name*** even when the
    same person was already talking. A run of unattributed lines under the line
    is the one place grouping costs more than it saves.
  - ***You have read a room when its newest message is on screen* and *the window
    has your attention.*** A room left open on a second monitor while you type
    somewhere else has not been read, and marking it read would quietly eat the
    line. The write is debounced to the one-per-five-seconds PROTOCOL §4 allows,
    fails silently, and never retries — the next `GET /read` is the truth, and a
    red line in the UI because a bookmark did not save would be worse than the
    bookmark not saving.
  - ***Notifications are batched per room for 1.2 seconds.*** A resume replays
    everything that happened while the socket was down, and thirty notifications
    from one burst is the behaviour this app exists to not have.

  ### The judgment call: `tauri-plugin-notification`

  *SPEC §4.2 says a mention "produces a real notification", and a real
  notification is an OS one. That needs a plugin — the same shape of decision
  T-304 made for `tauri-plugin-opener`, and for the same reason: every platform
  raises a notification differently and none of it is worth hand-rolling. It is
  scoped in `capabilities/default.json` to `notification:default` and nothing
  else. On Linux it talks to the freedesktop notification service over zbus, so
  it adds no system package to anybody's build; the README says what happens when
  there is no notification daemon (nothing breaks, you just don't get
  interrupted). Permission is asked for once and a refusal is final and silent —
  somebody who turned notifications off has already said what they want.*

  ### Notes for whoever is next

  - *There is no `@` autocomplete in the composer.* You have to know the
    username. That is a real rough edge and it is deliberately not in this task —
    it touches the composer's keyboard handling, where Up-arrow already means
    "edit my last message". Worth its own small task.
  - *The roster column has two modes now* — who's around, and the notify rules —
    switched by a `notify` / `done` button on the panel label's line. **T-401
    rebuilds the roster's contents and should keep that switch**, or move it
    somewhere better; a settings screen for one setting would be a screen too
    many.
  - *`AuthedApi.delete()` takes an optional body*, because `DELETE
    /me/notify-rules` identifies the rule in JSON rather than in the path.
  - *`GET /read` answers a map, and `linger-core`'s `ReadMap` is a type alias, so
    `ts-rs` has nothing to export for it.* The client writes
    `Record<RoomId, MessageId>` out of two generated types, which is a
    composition of wire types rather than a hand-written one. If a third map-typed
    endpoint shows up it is probably worth making it a struct.
  - *`newest` exists because the server computes a room's `last_message_id` when
    it is asked for and never pushes a new one.* `ready` seeds it and
    `message.create` keeps it up to date. If a future frame ever changes what the
    newest message in a room is — a delete that tombstones the last one, say —
    that is the place to fix it. Today a tombstone leaves the label bright until
    you look, which is the harmless direction.
  - *The whole feature reads the clock from `document.hasFocus()`.* **T-402 owns
    `room.focus` upstream and the idle states**, and when it lands it will have a
    better answer for "is this person actually here". The read-marker rule should
    move onto it rather than keeping its own.
  - *Testing drove two real windows over `/dev/uinput`* again (Wayland, no
    xdotool), with KWin scripting to place them. Three things worth writing down.
    **KWin 6's window list is `workspace.windowList()`**, not `workspace.windows`
    — the latter throws a bare "Type error" into the journal and nothing else.
    **A loaded KWin script runs once**; to run it again, unload it by name and
    load it again. And **two clients on one machine share one keyring entry**, so
    the second one to refresh its token spends the first one's, the family gets
    revoked exactly as designed, and a window you were not touching drops to the
    sign-in screen. That is the reuse protection working, not a bug — but it
    makes a two-client script on one box fiddly, and it is worth knowing before
    you spend twenty minutes on it.
  - *One dead end, recorded so nobody re-walks it.* Partway through the manual
    run a client stopped applying `message.create` frames while its status stayed
    `ready`. It looked like a bug in this task and it is not: a raw WebSocket
    client proved the server was fanning out, fresh clients were fine, and
    signing out and back in did not reproduce it. The most likely cause is Vite's
    HMR replacing `lib/gateway.ts` under a running window, which resets that
    module's `state` and `connected` and orphans the Tauri listeners. **If a
    `pnpm dev` window goes quiet after you edit the store, reload it before you
    go looking for a bug.**

## M4 — presence, roster, statuses

*Milestone check: roster updates live across two clients, and a status set on one
shows up on the other.*

**Entrance sounds moved out on 2026-08-21** (Matt). They were T-403, T-404 and
T-408; they are **T-901, T-902 and T-903** now — see *Backburner* near the end of
this file. The sounds are still V1 (SPEC §6, item 4); they are just not what is
in the way. M4 therefore finishes on T-405, and its check no longer mentions
sound.

- ✅ **T-401 · The roster** — effort: **high** *(landed 2026-08-21)*
  SPEC §3. Card stack, not a name list: styled name, presence dot, room, activity
  line, status (expanded), last-seen + away message for offline. Narrow window →
  horizontal strip above composer (never hidden). This panel is the product
  thesis — spend the polish here.

  *Verified against a real server with a real client.* One `linger-server` on a
  temp database, four accounts, the desktop client signed in as one of them and
  the other three driven over the real gateway by a script. The roster moved
  live: somebody focusing a room jumped to the top with `in #porch` under their
  name; killing their connection hollowed the dot, dropped them to the bottom
  and started their last-seen clock — all without a reload.

  ### What got built

  - `client/src/roster/roster.ts` — the deciding, as pure functions: who is
    where, what order the cards come in, and how long ago "a while" is. 20 tests.
  - `client/src/roster/Roster.tsx` + `roster.css` — the cards. Name in the
    person's own color and weight, presence dot, the room, the activity, a short
    duration, and the status underneath when you open one.
  - `client/src/lib/names.ts` — `personStyle`/`nameStyle`, lifted out of
    `Stream.tsx` so the stream and the roster paint a person the same way. When
    T-601 generates the palette, both pick it up at once.
  - `client/src/lib/clock.ts` — `useNow`, also lifted out of the stream. Two
    components that each owned a timer would disagree by up to a minute.
  - `client/src/lib/layout.ts` — the one width in the app: under 880px the
    roster cannot be a column.
  - `client/src/lib/gateway.ts` — one new field, `offlineAt`.

  ### The three decisions worth knowing

  - ***The narrow layout moves the component, it does not hide one.*** SPEC §3
    puts the strip *above the composer*, and the composer lives inside
    `Stream.tsx`, so `Stream` takes the roster as a slot and `App` hands it over
    when the window is narrow. Rendering it in both places and hiding one would
    mean two open cards and two scroll positions. React owns the breakpoint
    because the question is *where the panel is rendered*, not how it is painted;
    the frame carries the answer down to CSS as `data-narrow`.
  - ***When somebody left is something this client has to remember.*** The server
    writes `last_seen_at` on disconnect and never pushes the new value, so for
    anyone who leaves while you are watching, the copy in `ready` is stale.
    `offlineAt` records the moment the offline frame arrived and beats it. A
    `ready` clears the map, because the users it carries are freshly read. This
    is why "1m" appears under somebody the moment they drop, rather than the
    hours-old value the database had.
  - ***An away message left on a status is not shown to somebody plainly here.***
    Presence carries the live one and wins. The one saved on a status only
    renders for `idle`, `away` and `offline` — "back after work" under a dot that
    says they are in the room is the app contradicting itself.

  ### Notes for whoever is next

  - *The activity line is the registry label, so it reads `♪ Spotify`, not
    `♪ Bill Evans`.* SPEC §3's sketch shows the track; the wire type has no field
    for one and never will (AGENTS rule 2). Only the `media` kind gets a mark —
    `activityMark` is one function if T-911 wants more.
  - *The status image is not rendered.* `image_key` needs the media store M5
    builds. **T-506 owns it now** — T-405 took the rest of the status and left
    the image behind, because there is nowhere to put a file until T-501 lands.
    Everything else on a status is there: the line in their own styling, and
    the three labeled fields.
  - **T-405 should not rebuild the card.** The status renders here already; what
    it owns is the editor and the away flow. *(It did not — it lifted the card
    into `status/StatusCard.tsx` so the popover could draw the same one.)*
  - *The `notify` switch survived* — it is still the second mode of this panel,
    now inside `Roster.tsx` rather than `App.tsx`, so `App` no longer holds any
    roster state.
  - *Nobody is "in a room" yet except by hand.* The client never sends
    `room.focus` — **that is T-402** — so in normal use everyone sits at
    `around`. The room line, the occupancy sort and the in-room dot are all
    written and were tested with scripted clients; they light up for real when
    T-402 lands.
  - *Testing needed a keyboard.* Wayland has no `xdotool`, so the sign-in form
    was driven through a virtual keyboard on `/dev/uinput`. Two things cost time
    and are worth writing down. `struct input_event` is **24 bytes** on x86-64
    and Python's `struct` gives you 16 for `"<llHHi"` — `l` is 4 bytes in
    standard size, so it has to be `"<qqHHi"`; every event write returns EINVAL
    otherwise, with nothing to say why. And KWin's scripting console is how you
    focus and resize the window from a script: `workspace.windowList()`,
    `workspace.activeWindow = w`, and assigning `w.frameGeometry`.
- ✅ **T-402 · In-room mechanics** — effort: **medium** *(landed 2026-08-21)*
  Focus = in the room (send `room.focus`), background/idle >90s = leave it
  (`room.focus` with `null`), input-idle >10min = `idle` state. Header occupancy
  `#garage · Matt, Callie`; sidebar mini-stacks.

  ### What got built

  - `client/src/lib/presence.ts` — the deciding, as pure functions over a clock:
    whether you are still here, which room you should be in, and the next frame
    to send. 26 tests, including the 90-second leave, a brief alt-tab that must
    not bounce occupancy, and idle that only fires once the room is already left.
  - `client/src/lib/watchPresence.ts` — watches the window, holds the last thing
    we sent, and talks to the gateway. A fresh `ready` is a new session: we are
    `around` with no room until this clock re-announces.
  - `client/src/lib/occupancy.ts` — who is in a room, as people and as
    `Matt, Callie`. Commas, not "and", matching SPEC §4.1. 9 tests.
  - `client/src/lib/looking.ts` — whether the window has your attention, shared
    by the read-marker and the notifier so they cannot disagree.
  - Header occupancy and a rail stack of dots in the person's own color. Five
    dots is as many as the column will hold; the rest live in the accessible
    label, never as a "+N".

  ### The three decisions worth knowing

  - ***Two clocks, not one.*** Sitting still for 90 seconds takes you out of the
    room even if the window is still focused. Putting the app in the background
    starts its own 90-second clock from the moment of the blur, so a brief
    alt-tab does not bounce occupancy. Ten minutes with no input is `idle`, and
    that clock is input only — backgrounding does not pretend to be a keystroke.
  - ***Leave the room, then go idle.*** `room.focus` is what the server uses to
    set `in_room` / `around`. Sending `presence.update idle` while still holding
    a room would show somebody as idle *in* the room. So at ten minutes the
    first frame is a leave, if one is still owed, and the second is idle.
  - ***`away` is left alone.*** T-405 owns that word. `room.focus` with `null`
    would set the server to `around` and wipe it, so this file will not send
    anything over the top of `away`.

  ### Notes for whoever is next

  - *T-901 plays the sound on `room.enter`.* The server already sends that
    frame only to people in the room (T-203). This task is what makes "in the
    room" true for a real client, so entrance sounds now have someone to play
    for. Playback, mutes, and quiet hours are still T-901.
  - *T-405 should not send `away` while still in a room without leaving first.*
    The server sets `around` on any `room.focus` with `null`, which would wipe
    the away state. Leave, then away — the same order idle uses.
  - *T-911's activity poller should keep sending the registry id on
    `presence.update`.* Idle (and the around that follows it) currently sends
    `activity: null`, because there is no activity to report yet. Once the
    poller is running, a `presence.update` here would clear the line until the
    next poll; sending the last known id along, or letting the poller own every
    `presence.update`, is the fix. Don't invent a window title to fill it.
  - *The read-marker and the notifier now ask `isLooking()`* instead of
    `document.hasFocus()` on their own. Sitting still for 90 seconds drops you
    from occupancy; it does not unread the room. Looking at a message is still
    looking at it.
  - *The occupancy line is names, never a count.* A long list ellipsizes in the
    header. The rail stack draws at most five dots and puts every name in the
    accessible label.
  - *A leftover `room.sit` in a server comment was renamed to `room.focus`.*
    Vocabulary only; behavior was already `room.focus`.
  - *The 90-second and ten-minute edges are tested as arithmetic, not slept
    through.* The live desktop window opened to the paste-link screen against a
    real server; driving sign-in from this session was not possible (no
    compositor scripting, no injected keystrokes). Occupancy on the wire was
    checked with a second gateway client sitting in `#garage`: `room.enter`,
    occupancy of one, `presence.update` `in_room`. The header and the rail
    stack light up from those same frames.
- ✅ **T-405 · Statuses + away UI** — effort: **medium** *(landed 2026-08-21)*
  SPEC §4.6. Status editor (line 240, three labeled fields), away message
  supersedes; roster + popover rendering. **The status image is not built** —
  it needs the media store, and moved to **T-506** in M5.

  *Verified against a real server with a real client.* One `linger-server` on a
  temp database, two accounts, the desktop client signed in as one of them and
  the other watched on a real gateway socket. Set a status in the editor and it
  appeared on the other client. Typed an away message and saved: the card went
  hollow, dropped down the stack, swapped the status line for the away message,
  and the watching client got the two frames in the right order — the room
  leave first, then `away`. "I'm back" put all of it back, including rejoining
  the room. Clicking a name in the stream opened that person's card; Escape
  closed it and put focus back on the name.

  ### What got built

  - `client/src/status/status.ts` — the deciding, as pure functions: what is in
    a status, what an empty one looks like, what the editor's boxes become on
    the wire, and whether anything actually changed. 22 tests.
  - `client/src/status/StatusEditor.tsx` — the form. A third mode of the roster
    panel, reached from `edit` on your own card, because in a narrow window the
    cards are chips in a strip and a chip is not a form.
  - `client/src/status/StatusCard.tsx` — the card, lifted out of `Roster.tsx` so
    the roster and the popover draw the same one.
  - `client/src/status/PersonName.tsx` — a name in the stream, and the card it
    opens.
  - `client/src/lib/presence.ts` — `away` is now a state this file can reach,
    driven by `wantAway`. 10 new tests.
  - `client/src/lib/watchPresence.ts` — `setAway`, and **a bug fix; see below**.
  - `client/src/lib/watchPresence.test.ts` — the driver, with the gateway mocked.
    9 tests. New file: nothing covered this before.

  ### ⚠️ **The big one: T-402's presence driver was wedging itself.**

  *Found by reading the code while adding `setAway`, then reproduced in six
  lines of Node. `tick()` guarded against two passes at once by holding the
  in-flight promise: `ticking = run()`. When `decide` has nothing to say, `run`
  has no `await` to reach, so **its whole body — including the `ticking = null`
  in its own `finally` — runs before the assignment happens**, and the
  assignment then puts the resolved promise back. From that moment every tick
  saw a non-null `ticking`, set `pending`, and returned. Presence stopped
  sending anything for the rest of the session.*

  *A tick with nothing to say is the common case: it happens on the first
  pointer move after everything is in sync. So this fired within seconds of
  launch, every launch. The 90-second leave, the ten-minute idle and room
  changes all stopped working, silently, and whatever was last sent stayed true
  on the server — which is why T-402's own testing did not catch it. Its tests
  covered `presence.ts`, which was correct; nothing tested the driver.*

  *The guard is a plain boolean now, set before the first statement, so it
  cannot be beaten by its own body. `watchPresence.test.ts` covers it: with the
  old guard, 7 of its 9 tests fail.*

  ### The three decisions worth knowing

  - ***An away message is a field on the status, not a mode.*** Type one and you
    are away; clear it and you are back. That is what AIM did and it is why
    SPEC §4.6 puts `away_message` on the status object. It also means going away
    is one save, and the "I'm back" button is that same save with the field
    emptied.
  - ***Away is deliberate, so it is sticky.*** Moving the mouse does not bring
    you back. Sitting still does not make you more away, and the ten-minute idle
    clock is not allowed to quietly downgrade you. Only the person coming back
    ends it. Auto-returning on input would mean glancing at the window silently
    dropped the message you left — the exact AIM annoyance.
  - ***Two writes, on purpose, and the order matters.*** `PATCH /me` saves the
    message and is the **only** thing that stamps `away_since`, which is what
    the roster counts "away 20m" from. The gateway frame is what everyone sees
    now. The frame goes second, and the room leave goes before it, because the
    server sets `around` on any `room.focus` with `null` and the other order
    wipes the away state a moment after setting it.

  ### Notes for whoever is next

  - ***The status image is not built, and could not be.*** SPEC §4.6 allows one
    image, ≤512 KB at 400×200. `image_key` names an object in the media store,
    and there is no media store until **M5 builds the upload pipeline (T-501)**;
    `/uploads` is not even mounted. Pulling it in would be reaching into a later
    milestone. Everything else on a status is done. `statusOf` carries
    `image_key` through every save untouched — `PATCH /me` replaces the whole
    status object, so dropping it would delete an image the moment T-501 lets
    somebody set one. The editor says so in a line of copy. **It is written up
    as T-506, in M5, right after the upload pipeline that unblocks it.**
  - *T-911's poller now has a third frame to keep the activity id on.* T-402
    already flagged `idle` and the `around` after it; `away` and the `around`
    that follows it send `activity: null` for the same reason. Same fix: let the
    poller own every `presence.update`, or pass the last known id along.
  - *`.person-name` and `.person-dot` moved from `roster.css` to
    `status/status.css`* when the popover became the second thing that draws
    them. The dot's state colors key off its own `data-state` now rather than an
    ancestor's, so one set of rules covers both surfaces.
  - *The popover renders through a portal into the body.* The stream is
    virtualized — every row is inside a scrolling box under a transform — so a
    card drawn inside a row gets clipped by the first edge it reaches. It is
    fixed to the viewport off the name's own rectangle, flips above the name
    when there is no room below, and closes on scroll, because the name moves
    and it does not.
  - *A fresh server still has no rooms, and the client still cannot make one.*
    Both test runs had to create `#garage` over REST before there was anywhere
    to stand. That is T-410.

## M4.5 — the parts of the shell nobody has built yet

*Added 2026-08-21, after Matt tried to add a room to a live server and found there
was no button. Nothing in M0–M4 ever built a way to run a server from inside the
client, and the server rail in SPEC §3 has never existed at all.*

*Milestone check: a host who has only ever seen the app can create a server, add a
second room, invite a friend, and rename the server — without curl and without
reading the docs.*

**Every REST endpoint T-410, T-411 and T-412 need already exists and already has
an integration test** (T-103, T-104, T-106). Those three are UI over a finished
server: no protocol changes, no new wire types, no schema work. If one of them
finds itself editing `linger-core`, something has gone wrong — stop and ask.

**T-413 and T-414 are the exceptions** and were added after the other three.
T-413 adds the one endpoint this milestone is missing, and T-414 is not UI at
all — it is a subcommand on the server binary. Neither gates the milestone check
above.

- ✅ **T-410 · Host controls: rooms, invites, the server itself** — effort: **high**
  The host's side of the sweep. Everything here is host-only and must be *absent*,
  not disabled-and-greyed, for a member — a greyed-out control is a role matrix
  drawn in CSS (AGENTS rule 10, SPEC §2 anti-goals).
  - **Rooms.** `+ room` on the rail: slug, name, optional topic. Rename and set the
    topic (`PATCH /rooms/{id}`), archive (`POST /rooms/{id}/archive`), and reorder
    by `position`. Slug rules and errors come from the server, not a second copy of
    the regex in TypeScript.
  - **Invites.** Create, list, revoke (`/invites`). The screen's whole job is to
    produce a link the host can paste into a text message, so: one obvious copy
    button, the expiry and use-count in plain words, and revoked ones visibly dead.
    The link shapes are written down in PROTOCOL §2.2 — use those, do not invent a
    format.
  - **The server.** Name and accent key (`PATCH /server`). Accent is a palette key
    picked from `linger-core::PALETTE`, never a hex field (AGENTS rules 8 and 12).
  - *Accept:* on a real server, from a fresh host account: make a room, post in it,
    rename it, archive it and watch it leave the rail; make an invite, register a
    second account through it, revoke a second invite and watch it stop working;
    rename the server and see the rail change without a reload.
  - *Note:* archiving is the only delete this product has. The rail already filters
    `archived_at`, so an archived room disappears; do not add a "deleted" state.
  - **Landed 2026-08-21.** One panel over the stream column (`client/src/host/`) with
    three sections — rooms, invites, server — reached from `+ room` and `manage` on
    the rail, both drawn only for a host. No modal, and the roster stays visible while
    you work, which matters because an invite is something you make *for* somebody.
    Every accept criterion was walked on a real server from a fresh host account:
    room made and posted in, renamed with a topic, reordered, archived and gone from
    the rail; two invites made, one used to register a second account and one revoked
    and confirmed dead (`valid: false` on preview, `INVITE_INVALID` on register); the
    server renamed and the rail changed with no reload. Then signed in as the second
    account and confirmed `manage` and `+ room` are simply not drawn.
  - *No wire types were touched*, as the milestone note said they would not need to be.
    `linger-core` and `client/src/generated/` are untouched; the new REST calls are
    typed wrappers on the existing generated types in `lib/api.ts`.
  - *The panel keeps no copy of the room list.* Saves go to the server, the server fans
    out `room.create`/`room.update`, and both the panel and the rail render off the same
    gateway store — so they cannot disagree, and the other clients get it too. Verified
    live in the app, not just in the response.
  - *Reordering renumbers, it does not swap.* `moveRoom` in `host/host.ts` sorts by
    position, moves one place, then renumbers `0..n-1` and sends only the rooms whose
    number actually changed. Nothing stops two rooms sharing a `position` in the schema,
    and a plain swap on a list that has drifted moves the wrong room. On a tidy list it
    is still the two PATCHes you would expect.
  - *The accent picker is wired but cannot show colour yet.* It saves a palette key and
    the frame sets `--accent: var(--name-<key>, var(--accent-default))`, so the moment
    **T-601** generates `palette.generated.css` from `linger-core::PALETTE` the accent
    starts painting with no further work. Until then every key falls back to the built-in
    accent and every swatch is the same grey — which is why the key's *name* is the
    label and the swatch is decoration. A line of copy in the panel says so, the same way
    the status editor is honest about images; delete it when T-601 lands. Two small
    additions came with it: `--accent-default` in `tokens.css` (so the frame can override
    `--accent` without a self-referential fallback), and `client/src/lib/palette.ts`,
    which mirrors the sixteen keys for pickers to iterate over. The server is still the
    only authority on which keys are real (AGENTS rule 8) and no hex value crosses into
    the frontend (rule 12). **T-601 should reuse `PALETTE_KEYS` for the styling picker.**
  - *Slug rules exist in exactly one place.* The new-room form has no regex and no
    hint that restates one; a bad slug comes back as the server's own sentence.
  - *Copying a link answers honestly.* The Clipboard API needs a secure context and a
    permission the WebView can refuse, so `copyText` returns a boolean and the screen
    says when it failed. The link also sits in a selectable read-only box, which is the
    fallback that always works. It did work in WebKitGTK on Plasma 6.
  - *Archive asks first, because it is one-way.* There is no unarchive endpoint and no
    plan for one, so the button turns into `yes, archive` / `keep it` and a line of copy
    says what does and does not survive.
  - **Two things this run turned up that are not T-410's to fix:**
    - *A member who joins after you connected is invisible to you until you reconnect.*
      Registering the second account did not put them in the first client's roster.
      Nothing pushes a new `User`, and `buildRoster` maps over the `users` list, so a
      presence frame for somebody the client has never heard of has no card to land on.
      **Written up as T-415**, below. First read of this said it needed a new wire type;
      that was wrong — `user.update` already carries a whole `User` and the client's
      fold already appends an unknown id, so it is very likely one `publish` call in
      `POST /auth/register` plus a line of PROTOCOL. T-413 needs the mirror (a removed
      member leaving the roster live) and that half *does* need something new, so the
      two want deciding together.
    - *One client instance, once, held a live socket that delivered no frames.* It was
      the instance launched **before** the server had been set up: it sat on the sign-in
      screen through the whole first-run, and after setup it showed `ready` and the
      roster but never applied a single `room.create`, including ones made by `curl`.
      The server was fine — a raw WebSocket probe on the same box got the fan-out
      immediately — and restarting the app fixed it permanently. **It did not reproduce**
      on a clean second run through the same first-run flow. The likely suspect is the
      StrictMode double `connect`/`disconnect` in `Console`'s effect racing against
      itself; `disconnect()` sets `connected = null` synchronously and the frame
      listener drops everything when `connected` is not the api it was attached for.
      Recorded rather than fixed: it is `lib/gateway.ts` and the sign-in path (T-301,
      T-302), not this task, and a fix chased on one unreproducible sighting is a guess.
      **Fixed in T-412**, where it reproduced on demand with two servers signed in.
      The guess above was right. See T-412's notes for what the fix was.

- ✅ **T-411 · The member's sweep: settings, empty states, first five minutes** — effort: **medium**
  The other half, for somebody who is not the host. There is no settings surface in
  the client at all right now — display name, password, and density all live either
  nowhere or in a corner of the status bar.
  - **A settings surface.** Display name and username (`PATCH /me`), change password
    (`PATCH /me/password`), density mode (already built, currently only reachable
    from the stream header), sign out. One panel, Console styling, no modal stack.
  - **Empty and error states, read end to end.** "This server has no rooms yet" is
    the only one that exists. Walk the whole first run as a new member — paste an
    invite, register, land in a server, find the rooms, find the roster, find
    yourself — and fix every dead end and every piece of copy that assumes you
    already know how the app works.
  - **Keyboard and focus.** Every control this milestone adds is reachable and
    operable without a mouse, and focus rings are the one shadow SPEC §5.1 allows.
  - *Accept:* a person who has not seen the app gets from a pasted invite link to a
    posted message with no verbal instructions, and can change their display name
    and see it update in the roster on the other client.
  - *Note for whoever runs this:* resist inventing new surfaces. The fix for most of
    these is a sentence of copy or a button in a place that already exists.
  - **Landed 2026-08-23.** One panel over the stream column (`client/src/settings/`),
    reached from `you` on the rail and from your name in the status bar. Sign-out
    moved into that panel so the status bar is just the connection and who you are.
    Density stays on the room header as well, because you change it while reading.
  - *No wire types were touched.* `PATCH /me` and `PATCH /me/password` already
    existed; the new REST calls are typed wrappers on the generated types in
    `lib/api.ts`.
  - *A display-name save folds the HTTP answer into the gateway store*, the same
    way a status save does, so your own roster card and the status bar move
    without waiting for the `user.update` fan-out. The other client still gets
    that frame, which is the accept path.
  - *Changing a password revokes every refresh family on the server* (it treats
    the change as "someone else may have had the old one"). The panel signs back
    in with the new password so this window is not kicked out fifteen minutes
    later. If that second login fails, the copy says so rather than pretending
    you are still signed in.
  - *Empty-state copy, not empty-state screens.* A member on a server with no
    rooms is told the host has to make the first one. The rail says `no rooms
    yet` instead of a dash. The roster says `finding who's around…` until
    `ready`, so connecting does not look like an empty house. A silent room
    points at the composer. The invite form says you will see the roster.
  - *Keyboard.* Every control is a real button or a labelled field. Opening
    settings focuses the display name. Escape closes the panel. Focus rings are
    the global `:focus-visible` rule from M0.
  - *Verified:* `pnpm check` and 223 client tests, including the copy and the
    request-body helpers. The two-client "rename yourself and watch the other
    window" walk is the remaining human check — this session could not drive
    two GUI windows.

- ✅ **T-412 · The server list (SPEC §3, V1 item 17)** — effort: **high**
  SPEC §3's layout has a `SERVERS` rail — `● home / ○ work / + add` — and SPEC §6
  lists "multi-server list in the client" as V1 item 17. **None of it exists.** The
  client holds exactly one server today: one `baseUrl` in `session.ts`, one keyring
  entry, one gateway connection in the Tauri core.
  - Sessions become a list: one keyring entry per server, each with its own tokens,
    its own account, its own `baseUrl`. Signing out of one must not touch the others.
  - The Tauri core holds a connection per server, not a global one. Presence, the
    read-marker map, notify rules and the notifier are all currently module-level
    singletons in the WebView — they have to become per-server, or the second server
    will quietly overwrite the first one's state.
  - The rail lists servers with a live dot; `+ add` is the paste box that already
    exists (T-301), just reachable from inside the app instead of only before sign-in.
  - *Accept:* signed into two servers at once, in one window: both dots live, switch
    between them and the rooms, stream, roster and presence all follow; a message on
    the background server marks its dot without a count anywhere (SPEC §4.2, AGENTS
    rule 3); kill one server and the other is unaffected.
  - **This is the expensive one on the list.** It is a refactor of session, gateway
    and presence ownership, not a screen. It is also the one V1 feature with no
    task at all until now, which is why it is written down here rather than left to
    be discovered again at M7. If the budget is tight, T-410 and T-411 are what make
    the app usable; this one makes it match the spec.
  - *Landed 2026-08-25.* Everything below is keyed by the server's base URL, top to
    bottom, and nothing is shared between two servers.
  - *The keyring holds one entry per server, plus a small index listing them.*
    Keyrings cannot be enumerated, which is the only reason the index exists.
    Signing out deletes that server's entry and drops it from the index; the
    others are untouched. A sign-in from a build before this one is migrated on
    first read and the old single entry is deleted, so upgrading does not sign
    anybody out.
  - *The Tauri core holds a `HashMap` of connections, not one slot.* Every event
    it sends up is wrapped in a small envelope naming the server it came from, so
    the WebView routes on that rather than on "the connection".
  - *The gateway store is one snapshot per server.* `useGateway(baseUrl)` is the
    hook; `useServers()` is the whole map, and the rail is its only caller.
  - *Presence splits in two.* Focus and the last keystroke are facts about the
    window and stay global; what was last *said* to a server is per server. You are
    in at most one room anywhere, so entering a room on one server leaves the room
    on every other one — that is what makes switching servers in the rail take your
    presence with it.
  - *`+ add` is the T-301 paste box, unchanged, in the stream column.* It is the
    same component, minus the wordmark.
  - **This run fixed the "live socket that delivered no frames" bug** recorded under
    T-410 as *"recorded rather than fixed"*. It reproduced immediately with two
    servers signed in: home said `ready`, drew its rooms and its roster, and then
    never applied another frame — including messages posted by `curl`. The cause was
    what T-410 guessed at. `connect` and `disconnect` are both async and both touch
    the same module state, so React's StrictMode double-invoke (create, destroy,
    create, none of them awaited) could interleave them, and a listener teardown
    that landed after the next connect left the WebView deaf for the rest of the run.
    Two changes: connects and disconnects for one server now run one at a time
    through a small queue, and the two frame listeners are attached once and never
    taken down — they cost nothing when nobody is connected, because an event for a
    server with no link is dropped. `Console` also stopped closing everybody's
    connections on unmount; each `ServerLink` owns exactly one and closes its own.
    `gateway.test.ts` covers it: the last call is what you end up with, which is the
    assertion that fails without the queue.
  - *Verified on 2026-08-25 against two real servers on one box.* Signed into both
    at once in one window, both dots live; switched between them and the rooms,
    stream, roster and presence followed; a message on the background server turned
    its name heavier with no number anywhere; killed one server and the other kept
    delivering, then the dead one came back on its own and its dot filled in again;
    signed out of one and the other never noticed. The three real-keyring tests
    (`cargo test --lib -- --ignored`) pass against KWallet, migration included.
  - **What is not covered.** A server that is down at launch is dropped from the
    rail for that run with a notice, rather than sitting there with a dead dot and
    coming back on its own — its keyring entry survives, so it returns on the next
    launch. Fixing it means holding a session we cannot name the user of yet, and
    nothing in the accept list needs it. Notifications also still say
    "Callie in #garage" with no server name, which is fine with one server and
    slightly ambiguous with two.

- ⬜ **T-413 · Removing a member** — effort: **medium**
  The host can remove somebody from the server, and let them back in. There is no
  separate ban — see *Decided — the host's side* below for why one would buy
  nothing here. The UI word is **"Remove from the server"**: not kick, not ban
  (SPEC §1 vocabulary).
  - **The endpoint.** Host-only, mirroring the shape rooms already use for
    archive: `POST /users/{id}/remove` sets `deactivated_at`,
    `POST /users/{id}/restore` clears it. Neither request has a body, so this
    likely needs no new wire type at all — check before adding one.
    The host cannot remove themselves (`FORBIDDEN`).
  - **Setting the column is about a quarter of the job.** Four doors, three of
    them currently unlocked — verified by reading the code on 2026-08-21:
    - `AuthedUser` in `crates/linger-server/src/auth.rs` only verifies the JWT
      signature. A removed member keeps every endpoint until their access token
      expires, up to 15 minutes.
    - `auth::rotate_refresh` does not look at `deactivated_at` either, so they can
      keep minting fresh access tokens for the full 30-day refresh window.
    - The gateway verifies the token once at identify (`gateway/socket.rs`) and
      never re-checks. An open socket keeps receiving fan-out indefinitely.
    - `GET /users` and the roster query already filter `deactivated_at IS NULL`,
      so that half is free — they leave the roster on their own.
    Removal must therefore **set the column, revoke every refresh family that
    user owns, and close their live gateway sessions.** Decide and write down
    whether the bearer extractor also gets a deactivation check — that is a
    database read on every authenticated request, and the gateway close is what
    actually gets somebody out of the room. Letting a 15-minute access token die
    on its own is a defensible answer; leaving it undecided is not.
  - **Their invites die with them.** Revoke invites they created, in the same
    transaction. Otherwise their own link is a way back in.
  - **Their messages stay.** SPEC principle 3 — removing a person is not deleting
    what they wrote. No tombstones, no scrubbing the author off old messages.
  - **The control.** On the member's card, host-only, and *absent* rather than
    greyed out for everybody else — same rule as T-410. Removed members need to
    be listed somewhere the host can reach, or restore is a feature nobody can
    find.
  - *Accept:* integration tests — a removed member's next refresh is rejected,
    their live socket closes, they are gone from `GET /users`, their messages are
    still in the room, and an invite they created stops previewing; the host
    cannot remove themselves; restore puts them back and they can sign in again.
    Then on a real server with two clients: remove the second account and watch it
    leave the first client's roster without a reload.
  - *Note:* this is the one task in M4.5 that changes the server. If it grows a
    wire type, that type lives in `linger-core` like every other one (AGENTS
    rule 7).

- ✅ **T-414 · `reset-password`, for the host who is locked out** — effort: **low**
  Today there is no way back in. There is no password reset, and the setup token
  only exists while the server has zero users. A host who forgets their password
  leaves a server their friends can still talk on and that nobody can ever add a
  room to again.
  - **A subcommand on the server binary:** `linger-server reset-password <username>`.
    Same env config, same SQLite file, and the argon2 that is already linked into
    the binary. Under the documented deployment that is
    `docker compose run --rm linger linger-server reset-password matt`.
  - **Running it on the box is the proof of ownership.** No token, no login, no
    env-var credentials — the same position ARCHITECTURE §9 takes on first-run.
  - Read the new password from stdin or generate one and print it. **Never from
    argv** — that lands in shell history and in `ps` output. Fail loudly and exit
    non-zero if the username does not exist. Revoke that user's refresh families
    at the same time: the reason to reset a password is usually that somebody
    else may have had it.
  - **Check for an arg parser before adding one.** If the workspace has no `clap`
    today, one `std::env::args()` match is enough for a single subcommand and
    keeps the dependency list where it is.
  - **Say what to do about the running server.** A second process writing to the
    same SQLite file steps outside the single-writer discipline AGENTS calls out.
    Simplest honest answer: tell people to stop the server first
    (`docker compose stop linger`), and set a busy timeout so a mistake waits
    instead of erroring.
  - **README.** A short "Locked out" section next to Backup, written for somebody
    who is already stressed: the exact command, and what it does not do.
  - *Accept:* on a real server, reset the host's password, sign in with the new
    one from a fresh client, and confirm the old refresh token is dead. A wrong
    username exits non-zero with a plain-English message.
  - *Note:* the first plan here was a `sqlite3` recipe in the README instead of a
    subcommand. **It does not work** — passwords are argon2id hashes and the
    `sqlite3` shell cannot produce one. The `sqlite3` lines in the README stay
    where they are, for backup, which they do well.
  - **Landed 2026-08-25.** `linger-server reset-password <username>` generates a
    password and prints it; `--stdin` reads one instead. No new dependency: the
    workspace still has no argument parser, and one subcommand is a `match` on
    `std::env::args()`. `main.rs` dispatches before it does anything else, so
    `serve()` is now a function rather than the body of `main`.
  - **The password is never an argument, and the command says why.** A third
    positional word is a hard error whose message explains that it would be left
    in shell history and readable in `ps`, rather than silently being treated as
    a username typo. Generated passwords are four groups of five characters from
    a 32-character alphabet with the look-alikes removed (100 bits, and typeable
    off a terminal onto a phone).
  - **Opening the database is its own function.** `db::open_writer` is the single
    WAL writer with `create_if_missing(false)` and a 30-second busy timeout, and
    it does not run migrations. A typo in `LINGER_DATA_DIR` now says there is no
    database there instead of making an empty one and reporting that nobody by
    that name lives on it; running against a live server waits instead of
    erroring, though the README still says to stop the server first.
  - **A removed member can still be reset.** The lookup deliberately ignores
    `deactivated_at`, because refusing would report a real account as
    nonexistent, which is a different and untrue thing to say. The command
    prints a note that they will not get in until the host lets them back in.
    (Reads a column T-413 also uses; needs none of T-413's code, so this branch
    is off `main`.)
  - **The refresh families go with it**, the same rule as changing your own
    password: the reason to reset one is usually that somebody else may have had
    it, and a surviving refresh token keeps minting access tokens for thirty
    days.
  - **The documented command is `docker compose run --rm linger reset-password
    matt`** — not the `linger linger-server reset-password matt` written above.
    The image's `ENTRYPOINT` is already `linger-server`, so naming it again would
    pass it to itself as a subcommand. `-T` is needed on the `--stdin` form.
  - **Tests drive the real binary** (`tests/reset_password.rs`, six of them),
    because the argument parsing, the printed password and the exit code *are*
    the feature: reset, then sign in over HTTP with the printed password, and
    watch the old password and the old refresh token both stop working. Also
    covered: a shouted username, a piped password never being echoed back, a
    wrong username exiting non-zero, a password as an argument being refused, and
    a missing database.
  - **Not verified on a real server.** Everything in the accept list is covered
    by the tests above, against a real SQLite file and real HTTP. The
    `docker compose` form of the command has not been run against a live
    deployment.

- ⬜ **T-415 · A new friend shows up without anybody restarting** — effort: **medium**
  Found during T-410 on 2026-08-21, on a real server. **Somebody who joins after you
  are already connected does not appear in your roster at all until you restart the
  app** — not when they register, and not when they come online and start talking.
  Four friends on a server, a fifth joins, and the four of them are looking at a
  roster that does not have them on it. The roster is the product (SPEC §3), so this
  is not a rough edge, it is the main surface being wrong.

  **And it is louder than just a missing card.** `Stream.tsx` resolves an author out of
  the same `users` list and falls back to `"someone"` when it misses (line ~563). So the
  fifth friend says hello and the other four see **"someone"** said it, in the default
  styling, with no card behind the name. Their first impression of the server is being
  an anonymous stranger to everybody already in it.

  **Why it happens, exactly.** Nothing tells a connected client that a person exists.
  `client/src/roster/roster.ts` builds the stack by mapping over `gateway.users`, and
  that list is only ever filled from the `users` array in `ready`. `POST /auth/register`
  publishes nothing. So the new person is not in `users`, and `buildRoster` never
  reaches them. Their presence frames *do* arrive and *are* folded into `presence` —
  there is simply no name, style or card to draw them with, so they are dropped on the
  floor by the one `.map()` that matters.

  **It is probably five lines, not a protocol change.** T-410's first write-up said
  this needed a new wire type; that was wrong and is corrected here. `ServerEvent::UserUpdate(User)`
  already exists, already carries a whole `User`, and the client's fold for it already
  does `upsert(current.users, …)` — which *appends* when the id is unknown. So the
  machinery is all there and only the announcement is missing. Publish `UserUpdate`
  from `POST /auth/register` after the row is written, and connected clients grow the
  card on their own. There is even accidental proof: a new member who edits their
  display name today pops into everybody's roster at that moment, because `PATCH /me`
  is the one route that does publish.

  **The one decision to make.** PROTOCOL §8 documents `user.update` as "display name,
  style, or status changed", and "update" for somebody who did not exist a second ago
  is a stretch. Two honest options, and this task picks one and writes it down:
  - Widen the `user.update` line in PROTOCOL §8 to mean "this is the current state of
    this person, whether or not you had them" — cheapest, no new op, and it matches
    what the client already does.
  - Add `user.create`, which reads better in the log and on the wire but is a new op
    for every client to learn and buys nothing the fold does not already handle.
  Either way it is one line in `linger-core::gateway`'s docs or enum, so **this is the
  rare M4.5 task that is allowed to touch `linger-core`** — unlike T-410/T-411/T-412.

  **Do this with T-413 in front of you, or right after it.** T-413 needs the exact
  mirror — a removed member has to *leave* the roster live — and that one genuinely
  cannot reuse `user.update`, because the wire `User` has no `deactivated_at` field and
  should not grow one just to carry a tombstone. So T-413 needs its own way to say
  "this person is gone". Decide the pair together so the answer to "somebody arrived"
  and the answer to "somebody left" have the same shape, rather than two different
  inventions a week apart.

  - *Accept:* a gateway integration test — client A identifies, client B registers over
    REST, and A receives a frame carrying B and nothing else changes. Then on a real
    server with two clients: register a third account through an invite while both are
    open, and watch the card appear on both without a reload and without either person
    touching anything. Then have the new person go online and confirm their dot and room
    follow, which is the half that already works and must keep working.
  - *Note:* do not "fix" this by having the client refetch `GET /users` on a timer, or
    on every presence frame for an unknown id. Both are polling with extra steps, and
    the second one is a request storm triggered by strangers. The server knows when
    somebody joined; it should say so once.

## M5 — uploads, media pipeline, the media grid

*Milestone check: a 400 MB video uploads, resumes after a killed connection, appears in the media grid.*

- ⬜ **T-501 · Upload pipeline (local backend)** — effort: **high**
  ARCHITECTURE §8 + PROTOCOL §6. Slot creation validates size/quota/MIME
  allowlist; token-authenticated direct-PUT URLs (bytes never traverse app
  routes — separate upload listener path); multipart >8MB with per-part URLs
  (this is the resumability); complete: re-verify size, sniff real MIME,
  re-encode images (kills EXIF + polyglots in one step — `image` crate),
  blurhash, video poster via ffmpeg. Reject oversize at slot *and* at complete.
  *Accept:* the milestone check, scripted: kill mid-upload, resume, complete;
  EXIF-GPS test image comes out clean; fake-MIME file is caught.
- ⬜ **T-502 · S3 storage adapter** — effort: **medium** — same trait, presigned
  URLs; test against MinIO in CI (service container).
- ⬜ **T-503 · Separate media origin** — effort: **medium**
  ARCHITECTURE §7: serve objects on the cdn host; `Content-Disposition:
  attachment` + `nosniff` off-allowlist; activate the Caddyfile block; strict CSP
  on the app origin.
- ⬜ **T-504 · The media UI + link cards** — effort: **medium**
  SPEC §4.4: grid, filter by person/type/date, stars (starred never expire),
  each item links to its message/moment. Restrained link embeds (favicon, title,
  domain — one line): server-side metadata fetch **with SSRF guard** (deny
  private ranges, cap size/time), cached.
- ⬜ **T-505 · Expiry + storage accounting** — effort: **medium**
  365-day expiry of non-starred/non-pinned (host-configurable/off), background
  task; storage-used figure for the status bar and `GET /server`.
- ⬜ **T-506 · The status image** — effort: **low** *(the rest of T-405, once
  there is somewhere to put a file)*
  SPEC §4.6's last bullet: one image on a status, **≤512 KB, displayed at
  400×200**. T-405 built every other part of the status and stopped here,
  because `image_key` names an object in the media store and there was no media
  store — `/uploads` was not even mounted. `linger-core::limits` already holds
  `MAX_STATUS_IMAGE_BYTES`.

  Do this **after T-501**, which is what makes an upload possible, and after
  **T-503** if it has landed, so the image is served from the media origin like
  everything else. It is small: most of the work is already done in both
  directions.

  - **Client.** A file picker in `client/src/status/StatusEditor.tsx`, and an
    `<img>` in `client/src/status/StatusCard.tsx` — one component, so the
    roster card and the name popover both get it at once. Refuse an oversize
    file before uploading it rather than after, the way the editor already
    counts characters down.
  - **Client, the trap.** `statusOf` in `client/src/status/status.ts` already
    carries `image_key` through untouched on every save, because `PATCH /me`
    replaces the whole status object. **Keep that.** The moment the editor can
    set a key, a save that dropped the field would delete somebody's image.
    `status.test.ts` has a test pinning this — do not delete it either.
  - **Server.** `validate::status` checks lengths and nothing else: it will
    accept any string as an `image_key` today. It has to become a real check —
    the key names an object that exists, belongs to this user, is an image, and
    is within `MAX_STATUS_IMAGE_BYTES`. Without that, `image_key` is a
    user-controlled string that ends up in a URL.
  - **Also.** A status image should not expire out from under the status
    (T-505 expires non-starred objects at 365 days), and replacing one should
    not leave the old object orphaned.
  - *Accept:* set an image, see it at 400×200 in both the roster card and the
    popover, on a second client; a 600 KB file is refused with a sentence a
    person can read; a key naming somebody else's object is refused by the
    server; the image survives a year-old status once T-505 is in.
  - **Take the line of copy out of the editor when this lands.** It currently
    says "Status images arrive with file uploads."

## M6 — styling: names, palette, themes, fonts

*Milestone check: a gradient name from two palette keys, contrast verifiably ≥4.5:1 in both themes (the CI property test already guards the values).*

- ⬜ **T-601 · Name rendering engine** — effort: **medium**
  Build step: emit `palette.generated.css` from `linger-core::palette::css_variables`
  (single source of truth; oklch or hex per T-002's verdict). Render styled names
  everywhere names appear; gradient fixed 92°; shimmer (4s linear)/glow honor
  `prefers-reduced-motion`, disabled in compact + IRC; "normalize everyone"
  toggle flattens names *and* message fonts.
- ⬜ **T-602 · Style picker + settings** — effort: **medium**
  Two-click named-color picker (mIRC energy, modern craft), font/weight/italic/
  effect, live preview, msg-font override. Server already validates keys.
- ⬜ **T-603 · Themes + time-of-day warmth** — effort: **low**
  Light theme tokens exist; add the ~200K post-sunset warmth shift (one variable
  swap, user-disableable) and theme switching.
- ⬜ **T-604 · Font pipeline** — effort: **low**
  Script: fetch the 12 faces (`assets/fonts/README.md` table), subset
  (latin/latin-ext, 400/500/700 + italics) to woff2, keep OFL texts,
  `@font-face` wiring. No CDN.

## M7 — packaging and updates

*Milestone check: a signed installer per OS; one auto-update ships end-to-end.
Budget the full estimate; notarization is a version-sensitive slog — follow
current vendor docs, not memory (AGENTS.md).*

- ⬜ **T-701 · Updater + signing keys** — effort: **high**
  Tauri updater; generate the signing key and **back it up offline before
  anything ships** (losing it = no more updates, ARCHITECTURE §7.7). Release
  workflow: tag → build 3-OS installers → publish manifest.
- ⬜ **T-702 · Windows signing + macOS notarization** — effort: **high**
  Needs certs/Apple developer account (Matt). Harden CSP for release while here
  (drop dev relaxations from `tauri.conf.json`).
- ⬜ **T-703 · Server image publish** — effort: **low**
  ghcr.io workflow for `deploy/Dockerfile` (+ ffmpeg once T-902/T-501 need it),
  version tags, compose points at it.

## M8 — export

*Milestone check: one archive contains every message and file, and it opens.*

- ⬜ **T-801 · Full export** — effort: **medium**
  SPEC §4.11, PROTOCOL §7: any member, 1/hour; background job → zip: per-room
  markdown (readable layout: dividers, names, timestamps), `media/` tree,
  `media.md` index. Job progress endpoint; download via the media origin.
  *Accept:* export a seeded server, unzip, spot-check messages/media; second
  request within the hour gets `RATE_LIMITED`.

---

## Backburner — later, not the next thing

Two V1 pieces live here. They are still in the spec. They are not on the path to a
usable product. Do not pull either "while you're in there."

### Entrance sounds

*Moved here 2026-08-21 by Matt. These three are still V1 (SPEC §6, item 4) and the
server already fans out `room.enter` to exactly the right people (T-203). They are
simply the lowest-value thing left of M4, so they go after M8 rather than in the
middle of it. Anything that lands before them must not break the frames they rely on.*

- ⬜ **T-901 · Entrance sound playback** — effort: **medium**
  SPEC §4.1. Play on `room.enter` for those in the room; per-user cooldown
  5min/listener;
  global + per-user mute; quiet hours 22:00–08:00 listener-local default-on;
  picker UI for bundled sounds.
- ⬜ **T-902 · Custom sound upload** — effort: **medium**
  Server: accept ≤2s/≤200KB, transcode to Opus + loudness-normalize (−16 LUFS),
  **reject long files, never truncate**. Needs ffmpeg in the Docker image — add it.
- ⬜ **T-903 · Curate the bundled sounds** — effort: **low** *(Matt-assisted, taste required)*
  12–16 sounds per `assets/sounds/README.md` rules; `ffmpeg -af loudnorm=I=-16`
  for normalization; fill the source/license table.

### Activity detection

*Moved here 2026-08-23 by Matt. These used to be M5 / T-501…T-507; they are
**T-911…T-917** now, so they do not sit in the M5 task block. Still V1 (SPEC §6,
item 8). The Linux and Windows spikes are already retired, `linger-activity`
already compiles, and the Null backend already reports nothing — which is the
correct product until this comes back. It is not needed for a usable chat app,
and it is large: four OS backends, a poller, a registry, and a sharing-controls
UI. **Do not start T-911.** M5 (uploads) starts when M4.5's check passes.*

*Milestone check, when this comes back: foreground app appears in the roster on
Plasma 6 Wayland and Windows.*

- ⬜ **T-911 · KWin backend + poller wiring** — effort: **high**
  The spike-verified recipe is in `crates/linger-activity/src/backend.rs` docs —
  follow it exactly (zbus; own D-Bus service; KWin script via
  `loadScript`/`run`/`unloadScript`; `resourceClass` + pid → `/proc/exe`).
  Event-driven cache behind the pull `ActivityBackend` API. Then the shared
  poller: 3s focused / 15s unfocused, 20s continuous-foreground debounce,
  hide-list, registry resolution, `presence.update` upstream. Client never sends
  raw process identity — resolution happens client-side in Rust, registry id only.
  *Accept:* on Plasma 6 Wayland: switch apps, roster follows within ~25s
  (debounce); unknown app shows nothing; hide-listed app shows nothing.
- ⬜ **T-912 · X11 backend** — effort: **medium** — `x11rb`: `_NET_ACTIVE_WINDOW`
  → `_NET_WM_PID` → `/proc`. Covers GNOME-on-X11 too.
- ⬜ **T-913 · Windows backend** — effort: **medium** — `windows` crate, per T-004
  spike learnings.
- ⬜ **T-914 · macOS backend** — effort: **medium** — `objc2` +
  `NSWorkspace.frontmostApplication.bundleIdentifier`. No special permission
  needed *because* we don't read titles — keep it that way.
- ⬜ **T-915 · Hyprland + sway backends** — effort: **low** — their IPC sockets;
  both are simple JSON/i3-IPC queries.
- ⬜ **T-916 · Registry to ~200 entries + local overrides** — effort: **medium**
  Top games (Steam appids), browsers, creative, editors, media. Local override
  file in the client config dir; **never synced to the server**.
- ⬜ **T-917 · Sharing controls UI** — effort: **medium**
  SPEC §4.3: global one-click off (roster), per-server off, per-app hide,
  idle-only mode, **persistent visible indicator** + status bar `sharing: <app>`.
  Default off overall.

---

## Decided — the password stays, the friction goes

**Matt, 2026-08-21.** The question was whether a locally-run server needs a
password at all, since Ventrilo asked for a name and nothing else. Answer: keep
it, but stop making people think about it.

The password is not protecting the messages — it is protecting *being you*. The
roster is the product, and the documented deployment is a box on the open
internet (ARCHITECTURE §7), so name-only would mean anyone who ever received an
invite link can connect as anybody. What was actually annoying was the
**12-character floor**, which is friction paid on every fresh install and buys
very little when the client already keeps the password in the OS keyring.

**Done in the same pass:** the floor is now **8**, which with no composition
rules is the NIST SP 800-63B position. One constant,
`linger-core::limits::MIN_PASSWORD_CHARS`; the server's error message counts off
it instead of spelling the number out, and the two client forms mirror it the
same way `Stream.tsx` mirrors `MAX_MESSAGE_CHARS`. PROTOCOL §2 says 8 and says
why.

The two bigger options were considered and **not** taken: making the invite link
itself the credential (a real change to PROTOCOL §2 and the refresh-token family
logic), and a host-set no-auth LAN mode (honest for a LAN party, dangerous the
day the box gets a public IP). If the friction comes back, those are the next
two rungs, in that order.

---

## Decided — the host's side

**Matt, 2026-08-21.** Four questions came out of reading T-410 before starting it.
The server-admin story is otherwise unchanged and already built: the host deploys
the image, reads the one-time setup URL out of the logs, pastes it into the
client, and from then on is a normal user with `is_host = 1`. Rooms and server
settings are database rows edited from inside the app — never a config file.

**No host transfer.** `is_host` stays a boolean that nobody can hand to anybody
else. If the host goes quiet, the friends stand up a new server. That is a real
answer for a group of eight, and it keeps a second root from growing on a product
whose anti-goals include a permission matrix (AGENTS rule 10).

**Removal, not banning** — T-413. A ban needs something durable to ban *by*, an
address or a device id, and Linger does not store either and should not. It would
not work anyway: housemates share one address and phone networks reshuffle them.
A removal here is already ban-shaped — usernames are unique and immutable, the
account row stays, and registration is invite-only, so the host is the only door
back in. One action, plus the reverse, so a removal made in a bad moment is
fixable.

**The M5 storage knobs are environment variables.** The 50 GB pool and the
365-day file expiry (SPEC §7) go in the compose file with everything else, per
the position `config.rs` already takes. No config-file format to document,
version, and migrate.

**The printed setup URL is https when a domain is set** — fixed 2026-08-21, not a
task. The client keeps whatever scheme it is handed, for the REST base URL and
the gateway socket both, so printing `http://` pinned the host's own session to
plaintext on the very first thing they ever do. It falls back to `http` only for
a bare bind address, which has no certificate and is honestly plaintext.

---

## Parking lot (decisions needed, not tasks yet)

- Bundle identifier is `com.linger.desktop` — fine? Changing after M7 is painful.
- `MediaItem` wire shape is minimal (attachment + message/room link) — revisit
  when T-504 starts if the grid needs more.
- Link-preview fetching is host-side (privacy: the host's IP fetches, not each
  member's). Confirm this trade-off is intended before T-504.
