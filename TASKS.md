# TASKS.md — the running work queue

This file is the handoff surface between the architect session (which maintains
structure and this file) and implementation sessions run on **Opus 5**. It is a live
document: check tasks off, add discoveries, and keep it truthful.

## How to run a task

1. Open a fresh session on Opus 5 with the **effort level** the task states.
2. Paste/point at one task. Read `AGENTS.md` in full, then the spec sections the
   task references. State the current milestone before writing code.
3. Do not start a milestone until the previous one passes its check
   (`ARCHITECTURE.md` §10). Do not pull work from a later milestone "while you're
   in there."
4. When done: all listed acceptance criteria pass, `cargo test --workspace` and
   `cd client && pnpm check` are green, the task is checked off here, and any
   surprises are noted under the task.
5. **Never add AI attribution anywhere** — commits, comments, metadata. Author is
   Matt Guenther. No exceptions.

**Effort levels** (pick the stated one; when a task straddles, round up):
- **low** — mechanical or tightly specified; single area; the spec text is the design
- **medium** — a real feature across a few files with tests; design is settled, judgment needed in the details
- **high** — realtime/stateful/platform code, cross-cutting design, or anything AGENTS.md §"Where you will be wrong" flags

---

## Status

- ✅ **SPIKE (Linux)** — retired 2026-08-19 on this repo's primary target. KWin
  scripting over D-Bus works on Plasma 6.6 Wayland: got
  `resource_class="code", pid → /proc/<pid>/exe` with zero title exposure.
  The working recipe is documented in `crates/linger-activity/src/backend.rs`.
- ⬜ **SPIKE (Windows)** — T-004. Needs a Windows machine; documented-easy path.
- 🟨 **M0** — scaffold complete and green locally (frontend typechecks and
  builds; palette contrast property test in CI). Remaining: T-001–T-003.
- ✅ **M1 — server REST** (2026-08-19): auth (argon2id, EdDSA JWT, rotating
  refresh with family-reuse revocation), first-run setup, invites, stoop/rooms,
  messages/reactions/read markers, users/styling/signs, rate limiting. Every
  endpoint has an integration test driving real HTTP (T-101…T-107 below).
- ✅ **M2 — gateway** (2026-08-19): WS with hello/identify/ready, heartbeat,
  per-session sequence numbers, presence + sitting + occupancy + entrance-sound
  fan-out, typing limits, and resume via a 500-frame/120s session ring. The
  milestone check passes: a forced mid-stream disconnect resumes with **no gaps
  and no duplicates**, asserted by sequence accounting over real sockets
  (`tests/gateway.rs`).
- ⬜ **M3 … M9** — queued below. M3 is next; it wants T-002 (WebKitGTK install)
  done first so `pnpm tauri dev` runs.

What already exists (do not rebuild): workspace + CI; `linger-core` with typed
UUIDv7 ids, the full REST + gateway wire contract, palette/fonts/reactions/limits,
ts-rs export to `client/src/generated/` (committed, drift-checked in CI);
`linger-server` with config/env, WAL SQLite with **single-writer pool discipline**
(`db.write` is a 1-connection pool — keep it that way), migrations (full §5 schema),
error envelope, health route, integration-test harness pattern
(`crates/linger-server/tests/health.rs` — copy its `spawn_stoop` shape);
`linger-activity` with the resolution pipeline, registry loader (+41 seed entries),
backend classifier; Tauri 2 shell with the Console-token M0 frame; deploy files.

---

## M0 wrap-up

- [ ] **T-001 · First CI run goes green** — effort: **low**
  Push to GitHub, watch `ci.yml`. Fix anything `clippy --workspace --all-targets
  -- -D warnings` finds (none expected to be structural). Run `cargo fmt --all`
  from a rustfmt-equipped toolchain (dev box has none — CI's does; a devcontainer
  or `rustup component add rustfmt` on any machine works), commit the formatting,
  then delete the `continue-on-error: true` line from the fmt step so it gates.
  *Accept:* all three CI jobs green with fmt gating.

- [ ] **T-002 · Shell opens on Linux + `oklch()` gate** — effort: **low** *(Matt's machine)*
  `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
  librsvg2-dev`, then `cd client && pnpm tauri dev`. Verify the Console frame
  renders. Then the SPEC §5.4 M0 gate: temporarily set a token to an `oklch()`
  literal and confirm this WebKitGTK renders it; note the result here. If it does
  not render, M7 ships the generated hex fallbacks only (they already exist:
  `linger-core::palette::PaletteColor::hex`).
  *Accept:* screenshot of the frame; a one-line note "oklch: yes/no" added here.

- [ ] **T-003 · Shell opens on Windows and macOS** — effort: **low** *(needs hardware)*
  Same check as T-002 per OS. Can trail other work; must close before M8 starts.

- [ ] **T-004 · Windows activity spike** — effort: **low** *(needs hardware, throwaway)*
  Console binary: `GetForegroundWindow → GetWindowThreadProcessId →
  QueryFullProcessImageNameW`, print exe name 1/sec. One evening, per
  ARCHITECTURE §6. Keep learnings in a note under this task, then delete the code.

- [x] **T-005 · Confirm the 12 reaction keys** — effort: **low** *(Matt decision)*
  `linger-core::REACTIONS` is provisionally `heart laugh wow cry fire skull up
  down eyes clap hundred sparkles`. Confirm or edit *before* M3 renders them;
  changing keys after messages exist means a migration.
  *Done 2026-08-19: Matt confirmed the set as-is. AGPL-3.0 license also confirmed.*

## M1 — server REST: auth, invites, rooms, messages

*Milestone check: integration test suite drives the full REST surface with real
HTTP. No UI. Every endpoint gets an integration test (AGENTS.md).*

- [x] **T-101 · Auth foundation** — effort: **high**
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

- [x] **T-102 · First-run host setup** — effort: **medium**
  ARCHITECTURE §9. On boot with zero users: generate a one-time setup token,
  print `http://<domain-or-bind>/setup?token=…` to stdout, expose endpoints to
  create the host account + name the stoop (writes `stoop_config`). Token dies on
  use or restart. No env-var bootstrap credentials.
  *Accept:* integration test boots a fresh stoop, completes setup, second attempt
  fails, `GET /stoop` returns the name.

- [x] **T-103 · Invites** — effort: **medium**
  PROTOCOL §7 + §2. CRUD per protocol; codes 12 chars base32 from a CSPRNG,
  single-use default; unauthenticated preview endpoint; register consumes a use
  atomically (single-writer pool makes this easy — one UPDATE … WHERE guards).
  Rate limit 10/day/user. Host-or-creator may revoke.
  *Accept:* tests for expiry, max_uses exhaustion, revocation, preview of each
  state, and the register-through-invite flow.

- [x] **T-104 · Stoop + rooms endpoints** — effort: **medium**
  PROTOCOL §3. `GET/PATCH /stoop` (PATCH host-only; `accent_key` validated
  against `linger-core::PALETTE`). Rooms CRUD: create/patch/archive host-only,
  slug `[a-z0-9-]{1,32}` unique, `position` ordering, `last_message_id` filled
  from a join. Vocabulary check: it's `RoomId` and "room" in every string.
  *Accept:* tests incl. non-host 403s (`FORBIDDEN`), bad slug (`VALIDATION_FAILED`),
  bad palette key rejected.

- [x] **T-105 · Messages, reactions, read markers** — effort: **high**
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

- [x] **T-106 · Users, styling, signs** — effort: **medium**
  PROTOCOL §5. `GET /users`, `GET /users/:id`, `GET /me`, `PATCH /me`
  (display_name 1–32; `style` — **server-side validation** of `font_key`/
  `msg_font_key` ∈ `FONTS`, fill color keys ∈ `PALETTE`, weight ∈ {400,500,700};
  `sign` field caps from `linger-core::limits`; `entrance_sound` bundled-key only
  until M4), `PATCH /me/password` (verifies current). Notify rules endpoints
  (person-centric, SPEC §4.2 — no keyword rules).
  *Accept:* tests proving every invalid key is rejected server-side with
  `VALIDATION_FAILED`; style round-trips into `user_style` columns and back.

- [x] **T-107 · Rate-limit plumbing** — effort: **medium**
  One reusable keyed token-bucket (in-memory, `DashMap`) used by T-101/103/105
  constants in `linger-core::limits`. 429 envelope with `retry_after_ms` set.
  *Accept:* unit tests for bucket math + one integration test per limited surface.

## M2 — gateway

*Milestone check: a test client survives a forced mid-stream disconnect and
replays without gaps or duplicates. This is a flagged "you will be wrong" area:
test with real disconnects, not mocks.*

- [x] **T-201 · WS gateway core** — effort: **high**
  PROTOCOL §8. Upgrade at `/api/v1/gateway`; `hello` (30000ms) → `identify`
  (JWT) → `ready` (full `ReadyData`). Heartbeat/ack; two missed acks server-side
  → drop. Presence lives in `DashMap<UserId, PresenceEntry>` — **never persisted**.
  Fan-out: one `tokio::sync::broadcast` bus of `ServerFrame`s; per-connection task
  filters per PROTOCOL §8 fan-out rules (only `room.enter` is filtered in V1).
  Sequence numbers are per-session, assigned at send.
  *Accept:* integration test connects two clients, sends a message over REST,
  both receive `message.create` with correct `s` ordering.

- [x] **T-202 · Resume** — effort: **high**
  Per-session ring buffer (500 frames, 120s post-disconnect retention), `resume`
  → `resumed {replayed}` + replay, else `invalid_session` → client re-identifies.
  *Accept:* the AGENTS.md-mandated test: force-drop the socket mid-stream while
  messages keep flowing, resume, assert **no gaps and no duplicates** by sequence
  number; second test exceeds the window and asserts `invalid_session`.

- [x] **T-203 · Presence ops + rooms occupancy** — effort: **medium**
  `presence.update` (client), `room.sit`/stand, `room.occupancy`, `room.enter`
  (with entrance_sound key, only to sitters), `room.leave`, `typing` (1/4s/room
  server-enforced), `user.update`/`room.*` fan-out from REST mutations.
  *Accept:* two-client test: A sits, B sitting in same room receives `room.enter`;
  B in another room does not. Disconnect marks offline.

## M3 — client: message stream

*Milestone check: two clients on one machine exchange messages in real time.*

- [ ] **T-301 · API client + auth flow** — effort: **medium**
  Typed fetch wrapper over generated types (no `any`, no `as` across the wire);
  login/register screens (invite link paste); token refresh on 401; tokens in OS
  keyring via a Tauri command (`keyring` crate) — **test the headless/no-wallet
  fallback: clear re-login prompt, not a crash** (ARCHITECTURE §7.3).
  *Accept:* manual: login on a fresh profile, restart app, still authed.

- [ ] **T-302 · Gateway client in Rust core** — effort: **high**
  ARCHITECTURE §1: the WS client lives in the Tauri core, not the WebView.
  Connect/identify/heartbeat/resume/backoff(jittered, capped); emits Tauri events
  to the frontend; one Zustand-free store on the TS side (AGENTS: local state +
  one gateway store). Status bar shows protocol text states:
  `connecting… tls ok… identify… ready (28ms)`.
  *Accept:* kill the server, restart it, client resumes or re-identifies with no
  user action; status text follows the states.

- [ ] **T-303 · The stream** — effort: **high**
  SPEC §4.7 + §5.6. Virtualized list **from day one**; author grouping (break
  10min); session breaks (3h) with natural-language dividers (`SATURDAY MORNING`
  mono small-caps); aging via one CSS custom property (body only, floor 78%);
  3px per-person gutter rule, no avatar column; density modes Comfortable/
  Compact/IRC (IRC: one line/message, mono, no grouping/aging/effects).
  *Accept:* milestone check runs here: two clients exchange live messages;
  scrollback of 10k messages stays smooth (virtualization proof).

- [ ] **T-304 · Composer + message actions** — effort: **medium**
  Markdown (allowlist sanitizer, **no raw HTML passthrough**), send affordance in
  accent, edit/delete/reply, reactions by **weight** (denser/larger mark, count
  only in hover/aria — never rendered as a number).
  *Accept:* XSS attempt (`<img onerror>` etc.) renders inert; reactions show
  weight not numbers.

- [ ] **T-305 · "You left off here"** — effort: **medium**
  SPEC §4.2. Accent divider at last-read, persists for the session; room label
  weight change (60%→100% opacity, nothing else); "since you were gone" pulled
  from the room header; mention notifications (person-to-person only — there is
  no `@everyone` to implement); notify-rules settings UI.
  *Accept:* manual two-client script in the task notes; grep the diff for badge/
  count regressions.

## M4 — presence, roster, entrance sounds

*Milestone check: roster updates live across two clients; sounds play and respect mutes.*

- [ ] **T-401 · The roster** — effort: **high**
  SPEC §3. Card stack, not a name list: styled name, presence dot, room, activity
  line, sign (expanded), last-seen + away message for offline. Narrow window →
  horizontal strip above composer (never hidden). This panel is the product
  thesis — spend the polish here.
- [ ] **T-402 · Sitting-in mechanics** — effort: **medium**
  Focus = sit (send `room.sit`), background/idle >90s = stand, input-idle >10min
  = `idle` state. Header occupancy `#garage · Matt, Callie`; sidebar mini-stacks.
- [ ] **T-403 · Entrance sound playback** — effort: **medium**
  SPEC §4.1. Play on `room.enter` for sitters; per-user cooldown 5min/listener;
  global + per-user mute; quiet hours 22:00–08:00 listener-local default-on;
  picker UI for bundled sounds.
- [ ] **T-404 · Custom sound upload** — effort: **medium**
  Server: accept ≤2s/≤200KB, transcode to Opus + loudness-normalize (−16 LUFS),
  **reject long files, never truncate**. Needs ffmpeg in the Docker image — add it.
- [ ] **T-405 · Signs + away UI** — effort: **medium**
  SPEC §4.6. Sign editor (line 240, three labeled fields, image ≤512KB at
  400×200), away message supersedes; roster + popover rendering.
- [ ] **T-408 · Curate the bundled sounds** — effort: **low** *(Matt-assisted, taste required)*
  12–16 sounds per `assets/sounds/README.md` rules; `ffmpeg -af loudnorm=I=-16`
  for normalization; fill the source/license table.

## M5 — activity detection

*Milestone check: foreground app appears in the roster on Plasma 6 Wayland and Windows.*

- [ ] **T-501 · KWin backend + poller wiring** — effort: **high**
  The spike-verified recipe is in `crates/linger-activity/src/backend.rs` docs —
  follow it exactly (zbus; own D-Bus service; KWin script via
  `loadScript`/`run`/`unloadScript`; `resourceClass` + pid → `/proc/exe`).
  Event-driven cache behind the pull `ActivityBackend` API. Then the shared
  poller: 3s focused / 15s unfocused, 20s continuous-foreground debounce,
  hide-list, registry resolution, `presence.update` upstream. Client never sends
  raw process identity — resolution happens client-side in Rust, registry id only.
  *Accept:* on Plasma 6 Wayland: switch apps, roster follows within ~25s
  (debounce); unknown app shows nothing; hide-listed app shows nothing.
- [ ] **T-502 · X11 backend** — effort: **medium** — `x11rb`: `_NET_ACTIVE_WINDOW`
  → `_NET_WM_PID` → `/proc`. Covers GNOME-on-X11 too.
- [ ] **T-503 · Windows backend** — effort: **medium** — `windows` crate, per T-004
  spike learnings.
- [ ] **T-504 · macOS backend** — effort: **medium** — `objc2` +
  `NSWorkspace.frontmostApplication.bundleIdentifier`. No special permission
  needed *because* we don't read titles — keep it that way.
- [ ] **T-505 · Hyprland + sway backends** — effort: **low** — their IPC sockets;
  both are simple JSON/i3-IPC queries.
- [ ] **T-506 · Registry to ~200 entries + local overrides** — effort: **medium**
  Top games (Steam appids), browsers, creative, editors, media. Local override
  file in the client config dir; **never synced to the server**.
- [ ] **T-507 · Sharing controls UI** — effort: **medium**
  SPEC §4.3: global one-click off (roster), per-stoop off, per-app hide,
  idle-only mode, **persistent visible indicator** + status bar `sharing: <app>`.
  Default off overall.

## M6 — uploads, media pipeline, the shelf

*Milestone check: a 400 MB video uploads, resumes after a killed connection, appears in the shelf.*

- [ ] **T-601 · Upload pipeline (local backend)** — effort: **high**
  ARCHITECTURE §8 + PROTOCOL §6. Slot creation validates size/quota/MIME
  allowlist; token-authenticated direct-PUT URLs (bytes never traverse app
  routes — separate upload listener path); multipart >8MB with per-part URLs
  (this is the resumability); complete: re-verify size, sniff real MIME,
  re-encode images (kills EXIF + polyglots in one step — `image` crate),
  blurhash, video poster via ffmpeg. Reject oversize at slot *and* at complete.
  *Accept:* the milestone check, scripted: kill mid-upload, resume, complete;
  EXIF-GPS test image comes out clean; fake-MIME file is caught.
- [ ] **T-602 · S3 storage adapter** — effort: **medium** — same trait, presigned
  URLs; test against MinIO in CI (service container).
- [ ] **T-603 · Separate media origin** — effort: **medium**
  ARCHITECTURE §7: serve objects on the cdn host; `Content-Disposition:
  attachment` + `nosniff` off-allowlist; activate the Caddyfile block; strict CSP
  on the app origin.
- [ ] **T-604 · The shelf UI + link cards** — effort: **medium**
  SPEC §4.4: grid, filter by person/type/date, stars (starred never expire),
  each item links to its message/moment. Restrained link embeds (favicon, title,
  domain — one line): server-side metadata fetch **with SSRF guard** (deny
  private ranges, cap size/time), cached.
- [ ] **T-605 · Expiry + storage accounting** — effort: **medium**
  365-day expiry of non-starred/non-pinned (host-configurable/off), background
  task; storage-used figure for the status bar and `GET /stoop`.

## M7 — styling: names, palette, themes, fonts

*Milestone check: a gradient name from two palette keys, contrast verifiably ≥4.5:1 in both themes (the CI property test already guards the values).*

- [ ] **T-701 · Name rendering engine** — effort: **medium**
  Build step: emit `palette.generated.css` from `linger-core::palette::css_variables`
  (single source of truth; oklch or hex per T-002's verdict). Render styled names
  everywhere names appear; gradient fixed 92°; shimmer (4s linear)/glow honor
  `prefers-reduced-motion`, disabled in compact + IRC; "normalize everyone"
  toggle flattens names *and* message fonts.
- [ ] **T-702 · Style picker + settings** — effort: **medium**
  Two-click named-color picker (mIRC energy, modern craft), font/weight/italic/
  effect, live preview, msg-font override. Server already validates keys.
- [ ] **T-703 · Themes + time-of-day warmth** — effort: **low**
  Light theme tokens exist; add the ~200K post-sunset warmth shift (one variable
  swap, user-disableable) and theme switching.
- [ ] **T-704 · Font pipeline** — effort: **low**
  Script: fetch the 12 faces (`assets/fonts/README.md` table), subset
  (latin/latin-ext, 400/500/700 + italics) to woff2, keep OFL texts,
  `@font-face` wiring. No CDN.

## M8 — packaging and updates

*Milestone check: a signed installer per OS; one auto-update ships end-to-end.
Budget the full estimate; notarization is a version-sensitive slog — follow
current vendor docs, not memory (AGENTS.md).*

- [ ] **T-801 · Updater + signing keys** — effort: **high**
  Tauri updater; generate the signing key and **back it up offline before
  anything ships** (losing it = no more updates, ARCHITECTURE §7.7). Release
  workflow: tag → build 3-OS installers → publish manifest.
- [ ] **T-802 · Windows signing + macOS notarization** — effort: **high**
  Needs certs/Apple developer account (Matt). Harden CSP for release while here
  (drop dev relaxations from `tauri.conf.json`).
- [ ] **T-803 · Server image publish** — effort: **low**
  ghcr.io workflow for `deploy/Dockerfile` (+ ffmpeg once T-404/601 need it),
  version tags, compose points at it.

## M9 — export

*Milestone check: one archive contains every message and file, and it opens.*

- [ ] **T-901 · Full export** — effort: **medium**
  SPEC §4.11, PROTOCOL §7: any member, 1/hour; background job → zip: per-room
  markdown (readable layout: dividers, names, timestamps), `media/` tree,
  `shelf.md` index. Job progress endpoint; download via the media origin.
  *Accept:* export a seeded stoop, unzip, spot-check messages/media; second
  request within the hour gets `RATE_LIMITED`.

---

## Parking lot (decisions needed, not tasks yet)

- Bundle identifier is `com.linger.desktop` — fine? Changing after M8 is painful.
- `ShelfItem` wire shape is minimal (attachment + message/room link) — revisit
  when T-604 starts if the grid needs more.
- Link-preview fetching is host-side (privacy: the host's IP fetches, not each
  member's). Confirm this trade-off is intended before T-604.
