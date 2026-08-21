# TASKS.md — the running work queue

This file is the handoff surface between the architect session (which maintains
structure and this file) and implementation sessions run on **Opus 5**. It is a live
document: check tasks off, add discoveries, and keep it truthful.

Tasks are marked **⬜ not started** or **✅ done** — emoji, not markdown `- [ ]`
checkboxes, so the state of the queue is visible at a glance while scrolling. Use
the same two characters when you add or close a task.

## How to run a task

**Model: Claude Opus 5** for every task below. One task per fresh session — a
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
| T-302, T-501, T-601, T-801+T-802 | **xhigh** | The genuinely treacherous ones: realtime client resume, Wayland/KWin, resumable uploads + media pipeline, signing/notarization — AGENTS.md §"Where you will be wrong" territory |

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
  must be closed before M8.*
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
- ⬜ **M4 — presence, roster, entrance sounds** — started. T-401 landed
  2026-08-21: the roster is the card stack the product is about, it moves live
  across clients, and on a narrow window it becomes a strip above the composer
  instead of hiding. **T-402 landed 2026-08-21: focusing a room puts you in it,
  90 seconds of background or sitting still takes you out, ten minutes with no
  input is idle, and occupancy is names in the header and a small stack on the
  rail.** Next up: T-403 (entrance sound playback).
- ⬜ **M5 … M9** — queued below.
- 🚫 **AI is off the roadmap** (Matt, 2026-08-19). The local-model features and the
  agent surface that used to sit behind V1 are cut — SPEC §8 records why, AGENTS
  rule 13 is the enforceable version. Do not build any of it back.

**The one thing that bit us in T-301:** a webview page is a cross-origin caller,
so the server had to start sending CORS headers before the client could read a
single response. The allowed origins are a fixed list in
`crates/linger-server/src/routes/mod.rs`. The gateway WebSocket is *not* subject
to CORS and T-302 confirmed that — but if a future browser-side call mysteriously
"can't reach the server", that list is the first place to look.

Two decisions M5/M7 no longer have to make: **use `oklch()` directly** (WebKitGTK
2.52.3 supports it, T-002), and **T-503's Win32 backend is a known quantity**
(T-004). Both recipes are recorded next to the code that will need them.

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
  not render, M7 ships the generated hex fallbacks only (they already exist:
  `linger-core::palette::PaletteColor::hex`).
  *Accept:* screenshot of the frame; a one-line note "oklch: yes/no" added here.

  ### **oklch: YES** — the gate is closed, M7 can use `oklch()` directly.

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
  Same check as T-002 per OS. Can trail other work; must close before M8 starts.
  *Done 2026-08-19 as far as is possible without the hardware: the new
  `desktop.yml` workflow **builds the shell on `windows-latest` (6m54s) and
  `macos-latest` (1m10s)** and both are green. This is `cargo build`, not
  `check` — linking is where platform breakage actually shows, and the macOS run
  really did compile the platform-specific crates (`tao`, `muda`,
  `window-vibrancy`, `embed_plist`). So: **the shell compiles and links on all
  three OSes.***

  *⚠️ What CI cannot do is watch a window appear. The visual "it opens and the
  Console frame renders" sign-off on Windows/macOS still needs someone in front
  of those machines, and must happen before M8. Same for T-002's Linux
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
  and it was; **T-503 is a transcription job, not a research job.***

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
  print `http://<domain-or-bind>/setup?token=…` to stdout, expose endpoints to
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
  Message rate limit 10/10s/user. Attachments array wiring lands in M6; return
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
    ~1MB of binary, worth knowing before M8.*
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
    `var(--name-<key>, <neutral>)`. **M7 generates those variables** from
    `linger-core::PALETTE` into `palette.generated.css`; until it does, both fall
    back to something neutral and the stream lights up the moment that file
    exists. Nothing in `client/` knows what "azure" looks like, and nothing
    should. Font keys and name effects are M7's too, so a styled name currently
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
    M6 — so there is nothing to render, but the branch will need writing.*
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
    point an image at until uploads land in M6. It is not CommonMark and does
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
    are still waiting on M6.
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

## M4 — presence, roster, entrance sounds

*Milestone check: roster updates live across two clients; sounds play and respect mutes.*

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
    T-701 generates the palette, both pick it up at once.
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
    `activityMark` is one function if T-501 wants more.
  - *The status image is not rendered.* `image_key` needs the media store M6
    builds, and T-405 owns the editor that would set one. Everything else on a
    status is there: the line in their own styling, and the three labeled fields.
  - **T-405 should not rebuild the card.** The status renders here already; what
    it owns is the editor, the away flow, and the image.
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

  - *T-403 plays the sound on `room.enter`.* The server already sends that
    frame only to people in the room (T-203). This task is what makes "in the
    room" true for a real client, so entrance sounds now have someone to play
    for. Playback, mutes, and quiet hours are still T-403.
  - *T-405 should not send `away` while still in a room without leaving first.*
    The server sets `around` on any `room.focus` with `null`, which would wipe
    the away state. Leave, then away — the same order idle uses.
  - *T-501's activity poller should keep sending the registry id on
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
- ⬜ **T-403 · Entrance sound playback** — effort: **medium**
  SPEC §4.1. Play on `room.enter` for those in the room; per-user cooldown
  5min/listener;
  global + per-user mute; quiet hours 22:00–08:00 listener-local default-on;
  picker UI for bundled sounds.
- ⬜ **T-404 · Custom sound upload** — effort: **medium**
  Server: accept ≤2s/≤200KB, transcode to Opus + loudness-normalize (−16 LUFS),
  **reject long files, never truncate**. Needs ffmpeg in the Docker image — add it.
- ⬜ **T-405 · Statuses + away UI** — effort: **medium**
  SPEC §4.6. Status editor (line 240, three labeled fields, image ≤512KB at
  400×200), away message supersedes; roster + popover rendering.
- ⬜ **T-408 · Curate the bundled sounds** — effort: **low** *(Matt-assisted, taste required)*
  12–16 sounds per `assets/sounds/README.md` rules; `ffmpeg -af loudnorm=I=-16`
  for normalization; fill the source/license table.

## M5 — activity detection

*Milestone check: foreground app appears in the roster on Plasma 6 Wayland and Windows.*

- ⬜ **T-501 · KWin backend + poller wiring** — effort: **high**
  The spike-verified recipe is in `crates/linger-activity/src/backend.rs` docs —
  follow it exactly (zbus; own D-Bus service; KWin script via
  `loadScript`/`run`/`unloadScript`; `resourceClass` + pid → `/proc/exe`).
  Event-driven cache behind the pull `ActivityBackend` API. Then the shared
  poller: 3s focused / 15s unfocused, 20s continuous-foreground debounce,
  hide-list, registry resolution, `presence.update` upstream. Client never sends
  raw process identity — resolution happens client-side in Rust, registry id only.
  *Accept:* on Plasma 6 Wayland: switch apps, roster follows within ~25s
  (debounce); unknown app shows nothing; hide-listed app shows nothing.
- ⬜ **T-502 · X11 backend** — effort: **medium** — `x11rb`: `_NET_ACTIVE_WINDOW`
  → `_NET_WM_PID` → `/proc`. Covers GNOME-on-X11 too.
- ⬜ **T-503 · Windows backend** — effort: **medium** — `windows` crate, per T-004
  spike learnings.
- ⬜ **T-504 · macOS backend** — effort: **medium** — `objc2` +
  `NSWorkspace.frontmostApplication.bundleIdentifier`. No special permission
  needed *because* we don't read titles — keep it that way.
- ⬜ **T-505 · Hyprland + sway backends** — effort: **low** — their IPC sockets;
  both are simple JSON/i3-IPC queries.
- ⬜ **T-506 · Registry to ~200 entries + local overrides** — effort: **medium**
  Top games (Steam appids), browsers, creative, editors, media. Local override
  file in the client config dir; **never synced to the server**.
- ⬜ **T-507 · Sharing controls UI** — effort: **medium**
  SPEC §4.3: global one-click off (roster), per-server off, per-app hide,
  idle-only mode, **persistent visible indicator** + status bar `sharing: <app>`.
  Default off overall.

## M6 — uploads, media pipeline, the media grid

*Milestone check: a 400 MB video uploads, resumes after a killed connection, appears in the media grid.*

- ⬜ **T-601 · Upload pipeline (local backend)** — effort: **high**
  ARCHITECTURE §8 + PROTOCOL §6. Slot creation validates size/quota/MIME
  allowlist; token-authenticated direct-PUT URLs (bytes never traverse app
  routes — separate upload listener path); multipart >8MB with per-part URLs
  (this is the resumability); complete: re-verify size, sniff real MIME,
  re-encode images (kills EXIF + polyglots in one step — `image` crate),
  blurhash, video poster via ffmpeg. Reject oversize at slot *and* at complete.
  *Accept:* the milestone check, scripted: kill mid-upload, resume, complete;
  EXIF-GPS test image comes out clean; fake-MIME file is caught.
- ⬜ **T-602 · S3 storage adapter** — effort: **medium** — same trait, presigned
  URLs; test against MinIO in CI (service container).
- ⬜ **T-603 · Separate media origin** — effort: **medium**
  ARCHITECTURE §7: serve objects on the cdn host; `Content-Disposition:
  attachment` + `nosniff` off-allowlist; activate the Caddyfile block; strict CSP
  on the app origin.
- ⬜ **T-604 · The media UI + link cards** — effort: **medium**
  SPEC §4.4: grid, filter by person/type/date, stars (starred never expire),
  each item links to its message/moment. Restrained link embeds (favicon, title,
  domain — one line): server-side metadata fetch **with SSRF guard** (deny
  private ranges, cap size/time), cached.
- ⬜ **T-605 · Expiry + storage accounting** — effort: **medium**
  365-day expiry of non-starred/non-pinned (host-configurable/off), background
  task; storage-used figure for the status bar and `GET /server`.

## M7 — styling: names, palette, themes, fonts

*Milestone check: a gradient name from two palette keys, contrast verifiably ≥4.5:1 in both themes (the CI property test already guards the values).*

- ⬜ **T-701 · Name rendering engine** — effort: **medium**
  Build step: emit `palette.generated.css` from `linger-core::palette::css_variables`
  (single source of truth; oklch or hex per T-002's verdict). Render styled names
  everywhere names appear; gradient fixed 92°; shimmer (4s linear)/glow honor
  `prefers-reduced-motion`, disabled in compact + IRC; "normalize everyone"
  toggle flattens names *and* message fonts.
- ⬜ **T-702 · Style picker + settings** — effort: **medium**
  Two-click named-color picker (mIRC energy, modern craft), font/weight/italic/
  effect, live preview, msg-font override. Server already validates keys.
- ⬜ **T-703 · Themes + time-of-day warmth** — effort: **low**
  Light theme tokens exist; add the ~200K post-sunset warmth shift (one variable
  swap, user-disableable) and theme switching.
- ⬜ **T-704 · Font pipeline** — effort: **low**
  Script: fetch the 12 faces (`assets/fonts/README.md` table), subset
  (latin/latin-ext, 400/500/700 + italics) to woff2, keep OFL texts,
  `@font-face` wiring. No CDN.

## M8 — packaging and updates

*Milestone check: a signed installer per OS; one auto-update ships end-to-end.
Budget the full estimate; notarization is a version-sensitive slog — follow
current vendor docs, not memory (AGENTS.md).*

- ⬜ **T-801 · Updater + signing keys** — effort: **high**
  Tauri updater; generate the signing key and **back it up offline before
  anything ships** (losing it = no more updates, ARCHITECTURE §7.7). Release
  workflow: tag → build 3-OS installers → publish manifest.
- ⬜ **T-802 · Windows signing + macOS notarization** — effort: **high**
  Needs certs/Apple developer account (Matt). Harden CSP for release while here
  (drop dev relaxations from `tauri.conf.json`).
- ⬜ **T-803 · Server image publish** — effort: **low**
  ghcr.io workflow for `deploy/Dockerfile` (+ ffmpeg once T-404/601 need it),
  version tags, compose points at it.

## M9 — export

*Milestone check: one archive contains every message and file, and it opens.*

- ⬜ **T-901 · Full export** — effort: **medium**
  SPEC §4.11, PROTOCOL §7: any member, 1/hour; background job → zip: per-room
  markdown (readable layout: dividers, names, timestamps), `media/` tree,
  `media.md` index. Job progress endpoint; download via the media origin.
  *Accept:* export a seeded server, unzip, spot-check messages/media; second
  request within the hour gets `RATE_LIMITED`.

---

## Parking lot (decisions needed, not tasks yet)

- Bundle identifier is `com.linger.desktop` — fine? Changing after M8 is painful.
- `MediaItem` wire shape is minimal (attachment + message/room link) — revisit
  when T-604 starts if the grid needs more.
- Link-preview fetching is host-side (privacy: the host's IP fetches, not each
  member's). Confirm this trade-off is intended before T-604.
