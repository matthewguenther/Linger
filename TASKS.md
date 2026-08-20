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
- ⬜ **M3 — client: message stream** — started, and **the milestone check
  passes** (2026-08-20). T-301 landed 2026-08-19 (sign in, and stay signed in).
  T-302 landed 2026-08-19: the client holds a live gateway connection from the
  Tauri core, survives the server being killed and restarted with no user
  action, and resumes without gaps or duplicates against a real server. T-303
  landed 2026-08-20: the stream is real, two clients on one machine exchanged
  messages in real time, and 10,000 messages of scrollback held 24–43 rows in
  the DOM. T-304 landed 2026-08-20: markdown that can never become HTML, the
  composer, edit/delete/reply, and reactions by weight. **Next up: T-305 ("you
  left off here"), which closes M3.**
- ⬜ **M4 … M9** — queued below.
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

  ### **Done 2026-08-20.** Both halves of the accept criterion were run for real.

  *A throwaway server was seeded with seven extra accounts and a room full of
  hostile input, then the real client was pointed at it.
  [`docs/t304-inert.png`](docs/t304-inert.png) is the answer to the first half:
  `<img src=x onerror=…>`, `<script>alert(document.cookie)</script>`,
  `<a href="javascript:…">`, `<svg/onload=…>` and `[click me](javascript:alert(1))`
  all render as the characters somebody typed. No dialog, no broken-image icon, no
  link. The last one shows its own source, because a refused destination is worth
  seeing rather than quietly swallowing.*

  *[`docs/t304-weight.png`](docs/t304-weight.png) is the second half: one message
  carrying the same five reactions held by one, two, four, six and eight people.
  The mark gets larger and more solid across the five and **there is not a number
  anywhere** — the count is in the hover title and the accessibility label only,
  which is where PROTOCOL §4 says it may live.*
  [`docs/t304-actions.png`](docs/t304-actions.png) *is the action strip with the
  twelve reactions open in it.*

  *Everything else was driven in the real app too, entirely from the keyboard:
  replying (banner up, focus into the composer, quote line on the sent message),
  editing in place (the editor opens on the **markdown source**, enter saves,
  "edited" lands at the end of the last line), deleting (confirm, tombstone, and
  the reply pointing at it survives), the twelve-reaction picker, IRC and
  Comfortable density, and `Callie is typing…` driven by a second account on the
  gateway.*

  ### What got built

  - `client/src/stream/markdown.ts` — the parser. Bold, italic, strike, code spans
    and fences, quotes, both kinds of list, `[label](url)` and bare links. It emits
    **typed nodes, never an HTML string**.
  - `client/src/stream/Markdown.tsx` — draws those nodes as React elements.
  - `client/src/stream/MessageRow.tsx` — one message: the body, what it replies to,
    its reactions, and the action strip.
  - `client/src/stream/Composer.tsx` — a textarea that grows, enter to send,
    shift-enter for a new line, the reply banner, and the line that says who else
    is typing.
  - `client/src/stream/reactions.ts` — key → glyph → weight step.
  - `client/src/lib/limits.ts`, `client/src/lib/open.ts`, `client/src/lib/autogrow.ts`.
  - `client/src/lib/gateway.ts` — `editMessage`, `deleteMessage`, `setReaction`,
    `startTyping`, replies on `sendMessage`, and the `typing` frame.
  - 37 more tests (58 total): the parser's grammar and every hostile input above,
    the weight steps, and two drift tests that **read the Rust source** and fail if
    `linger-core`'s reaction keys or limits stop matching the TypeScript copies.

  ### The three rules this ended up resting on

  - ***There is no sanitizer, because there is nothing to sanitize.*** A sanitizer
    is what you need when you have built an HTML string and want to make it safe
    again. Parsing to typed nodes and rendering those as elements means the unsafe
    string is never built. The rule that keeps it true is one line long: no
    `dangerouslySetInnerHTML` anywhere in `client/`. Grep for it — there are none,
    on purpose, so there is nothing to copy from.
  - ***A link destination is the one string left that can still do something.***
    Everything else in a body is inert by construction; an `href` is not. So it
    goes through `new URL()` — which settles case, encoding and control characters
    before anything is compared — and then an allowlist of http, https and mailto.
    The check is repeated in `lib/open.ts` and again in the Tauri capability,
    because a comparison costs nothing and a miss costs everything.
  - ***A strip that swaps in place beats a popover.*** React, reply, edit, delete,
    the twelve reactions, and "delete this?" are all the same strip in different
    states. A popover next to a virtualized row would have to be positioned against
    a scroller, clipped at its edges and layered over the rows after it — three
    problems, in exchange for nothing.

  ### Notes for whoever is next

  - *Reaction glyphs are a **client** decision (PROTOCOL §4) and the twelve keys are
    still marked provisional in `linger-core::lib.rs` — "confirm the set with Matt
    before M3 ships reactions". They ship here as the obvious reading of those keys
    (`up` → 👍, `hundred` → 💯). **Changing a glyph is one line in
    `stream/reactions.ts`; changing a key is a wire change**, so this is the cheap
    moment to say the set is wrong. The provisional note is still in the Rust.*
  - *`typing.start` is wired, which T-302's note flagged as this task's job. It is
    not in the SPEC §6 V1 list — PROTOCOL §8 defines both directions and the server
    already rate-limits it, so leaving it unwired would have left a dead path. The
    line above the composer holds its height whether or not it has anything to say,
    so nobody's typing ever moves the stream. **If typing indicators are not wanted,
    the whole feature is `startTyping` plus the `typing` case in the store.***
  - *A message body is hostile input, and a parser is a place that costs time.
    Three shapes of body used to scan to the end of the string once per
    character — 8,000 unclosed `*`, `` ` `` or `[` — which is 143ms of one frame
    per render for a body somebody could send on purpose. Emphasis and code spans
    now remember that a delimiter of a given shape found nothing to close
    against (true from any later position too, since closing depends only on what
    surrounds the closer), brackets are bounded by a label length instead
    (**not** the same argument: `[[a]` has no link at the first bracket and one
    at the second), and `MessageBody` parses once per body rather than once per
    render. 143ms → 3.2ms, and `markdown.test.ts` holds a 50ms budget over five
    such bodies so it cannot quietly come back.*
  - *No headings in the markdown subset. Type sizes are fixed by the density mode
    (SPEC §5.2), so a line that silently became 24px would be a hole in the design
    system. No images or tables either — an image is an upload (M6).*
  - *Editing is lost if the message scrolls far enough out of view for the
    virtualizer to unmount its row. The draft lives in the row, and the row is not
    guaranteed to exist. It takes a real effort to hit — the overscan is twelve rows
    either side of a message you are looking at — and the fix is lifting the draft
    into `Stream.tsx` beside `editingId` if it ever bites.*
  - *In IRC mode a message with reactions is no longer one line. Neither is one with
    a code block, so "one line per message" was already an intention rather than a
    rule; reactions ride at the end of the text and wrap.*
  - *New dependencies: `tauri-plugin-opener` in the shell and
    `@tauri-apps/plugin-opener` in the client, so a link can open in the system
    browser instead of navigating the WebView away from the app forever. The
    capability in `client/src-tauri/capabilities/default.json` narrows it to http,
    https and mailto. Also `@types/node`, so the two drift tests can read the Rust
    source. The bundle went 245KB → 261KB (77KB → 82KB gzipped).*
  - *`.density-option` in `app.css` sets `font: inherit`, which defeats the
    `meta` class it also carries — so the density picker draws in the 13.5px body
    face rather than 11px mono. It is T-303's and it may well be deliberate, so it
    was left alone; the new buttons in `stream.css` avoid the shorthand for exactly
    this reason. Worth one decision from Matt either way.*
  - *Testing needed the GUI driven again and this box still has no xdotool. This
    time it was a keyboard only, over `/dev/uinput`, which turned out to be a real
    test of its own: **every action in this task is reachable and operable from the
    keyboard**, including the strip, the picker and the confirm. The scaffolding is
    throwaway and is not in the repo.*

- ⬜ **T-305 · "You left off here"** — effort: **medium**
  SPEC §4.2. Accent divider at last-read, persists for the session; room label
  weight change (60%→100% opacity, nothing else); "since you were gone" pulled
  from the room header; mention notifications (person-to-person only — there is
  no `@everyone` to implement); notify-rules settings UI.
  *Accept:* manual two-client script in the task notes; grep the diff for badge/
  count regressions.

## M4 — presence, roster, entrance sounds

*Milestone check: roster updates live across two clients; sounds play and respect mutes.*

- ⬜ **T-401 · The roster** — effort: **high**
  SPEC §3. Card stack, not a name list: styled name, presence dot, room, activity
  line, status (expanded), last-seen + away message for offline. Narrow window →
  horizontal strip above composer (never hidden). This panel is the product
  thesis — spend the polish here.
- ⬜ **T-402 · In-room mechanics** — effort: **medium**
  Focus = in the room (send `room.focus`), background/idle >90s = leave it
  (`room.focus` with `null`), input-idle >10min = `idle` state. Header occupancy
  `#garage · Matt, Callie`; sidebar mini-stacks.
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
