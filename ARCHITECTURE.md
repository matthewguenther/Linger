# Linger — Architecture

Companion to `SPEC.md`. Wire protocol is in `PROTOCOL.md`. Agent working rules are in
`AGENTS.md`.

---

## 1. Shape

```
┌────────────────────────────────────────────┐
│  CLIENT (Tauri 2)                          │
│                                            │
│  Frontend — TypeScript + React + Vite      │
│    message list (virtualized)              │
│    roster, composer, settings              │
│                                            │
│  Core — Rust                               │
│    gateway client (WS, resume, backoff)    │
│    activity detector (per-OS backends)     │
│    token storage (OS keyring)              │
│    local cache (SQLite)                    │
└────────────────────────────────────────────┘
                    │ WSS + HTTPS
                    ▼
┌────────────────────────────────────────────┐
│  SERVER (single Rust binary)               │
│                                            │
│  axum   REST + WS gateway                  │
│  SQLite (WAL)  messages, users, rooms      │
│  Object store adapter                      │
│    local filesystem | S3-compatible        │
│  Background: media processing, expiry      │
└────────────────────────────────────────────┘
```

Deployment is one binary plus one data directory, or one Docker image.

---

## 2. Stack decisions and why

| Choice | Reason | When to revisit |
|---|---|---|
| **Rust server (axum + tokio)** | A single static binary is a dramatically better self-host story than "install Node, install pnpm, run migrations." Also lets client and server share types. | Never for V1. |
| **SQLite (WAL), not Postgres** | 20 friends, a few hundred messages a day. Postgres is ceremony at this scale. One file to back up. | Only if a server exceeds ~2k users or needs multi-node. A `Repository` trait keeps the swap cheap. |
| **Tauri 2, not Electron** | ~5–15 MB bundle vs ~150 MB; ~100 MB idle RAM vs ~400 MB; first-class Rust for native OS access, which the activity detector requires. | Only if WebKitGTK rendering on Linux becomes untenable. |
| **TypeScript + React frontend** | Largest ecosystem for virtualized lists and rich text; fastest to iterate. | — |
| **Object store adapter, not blobs in DB** | 500 MB files must never traverse the app server. | — |

### The Tauri caveat, stated up front

Tauri uses the OS WebView: WebView2 on Windows, WKWebView on macOS, **WebKitGTK on
Linux**. WebKitGTK is the weak link — it lags on newer CSS and has historically been
rough with WebRTC.

Consequences:
1. **Test on Linux first, every milestone.** If it works on WebKitGTK it works
   everywhere. The reverse is not true.
2. **Keep V2 audio in Rust, not in the WebView.** Use a Rust WebRTC stack (`webrtc-rs`)
   with `cpal` for device I/O. This is the better architecture anyway and it removes
   WebKitGTK from the critical path for voice.
3. Avoid CSS features newer than ~2023 without checking WebKitGTK support. `oklch()` is
   supported and is required by §4.5 of the spec; verify it in the target WebKitGTK
   version during M0.

---

## 3. Repository layout

```
linger/
├─ SPEC.md
├─ ARCHITECTURE.md
├─ PROTOCOL.md
├─ AGENTS.md
├─ Cargo.toml                 # workspace
├─ crates/
│  ├─ linger-core/             # shared types, ID generation, color palette
│  ├─ linger-server/           # axum, SQLite, object store, gateway
│  └─ linger-activity/         # activity detection, per-OS backends
├─ client/
│  ├─ src-tauri/              # Tauri shell, commands, keyring, gateway client
│  └─ src/                    # React frontend
├─ registry/
│  └─ apps.json               # bundled app registry (see §6)
├─ assets/
│  ├─ fonts/                  # subset, bundled
│  └─ sounds/                 # curated entrance sounds
└─ deploy/
   ├─ Dockerfile
   ├─ compose.yaml            # linger + caddy
   └─ Caddyfile
```

`linger-core` is the contract. Types defined there are exported to TypeScript via `ts-rs`
during build. **The frontend never hand-writes a type that crosses the wire.**

---

## 4. Identifiers

**UUIDv7**, stored in SQLite as `BLOB(16)`, rendered on the wire as lowercase hex.

- Time-sortable, so `ORDER BY id` is chronological and pagination is a simple range scan
- No coordination needed, unlike Snowflake
- 16 bytes indexed, not 36 bytes of text

Do not use auto-increment integers (leaks volume, breaks any future merge) and do not
use UUIDv4 (destroys index locality).

---

## 5. Schema

SQLite, WAL mode, `foreign_keys=ON`, `synchronous=NORMAL`.

```sql
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

CREATE TABLE server_config (
  key             TEXT PRIMARY KEY,
  value           TEXT NOT NULL
);
```

**Presence is never persisted.** It lives in an in-memory `DashMap<UserId, Presence>` in
the gateway. On restart everyone is offline until they reconnect. This is correct: stale
persisted presence is worse than none.

---

## 6. Activity detection — the hard part

Lives in `crates/linger-activity`. Single public API:

```rust
pub enum Activity {
    None,
    App { registry_id: String, since: SystemTime },
}

pub trait ActivityBackend: Send + Sync {
    fn foreground_process(&self) -> Result<Option<ProcessIdent>>;
}

pub struct ProcessIdent {
    pub exe_name: String,      // "firefox", "steam_app_730"
    pub exe_path: Option<PathBuf>,
    pub bundle_id: Option<String>,   // macOS only
}
```

**No backend ever returns a window title.** The type system enforces the privacy rule —
there is no field for it. Do not add one.

### Per-platform reality

| Platform | Approach | Difficulty |
|---|---|---|
| Windows | `GetForegroundWindow` → `GetWindowThreadProcessId` → `QueryFullProcessImageNameW` | Easy |
| macOS | `NSWorkspace.frontmostApplication` → `bundleIdentifier` | Easy, **and needs no special permission** — this is precisely because we do not read titles |
| Linux / X11 | `_NET_ACTIVE_WINDOW` → `_NET_WM_PID` | Easy |
| **Linux / Wayland** | **Per-compositor. No unified API.** | **Hard** |

Wayland deliberately has no `_NET_ACTIVE_WINDOW` equivalent, for security reasons. Every
activity tracker implements a different solution per compositor:

- **KDE / KWin** (the primary dev target): KWin scripting API over D-Bus
  (`org.kde.KWin.Scripting`), or the `kde-plasma-window-management` Wayland protocol.
  Works identically under X11 and Wayland sessions.
- **Hyprland**: IPC socket at `$XDG_RUNTIME_DIR/hypr/$HIS/.socket.sock`
- **sway**: i3 IPC
- **GNOME**: no official API. Requires a shell extension exposing D-Bus. **Ship the X11
  backend for GNOME and document that Wayland+GNOME reports presence only, not
  activity.** Do not build a GNOME extension in V1.

Select the backend by reading `$XDG_SESSION_TYPE` and `$XDG_CURRENT_DESKTOP` with
substring matching. Fall back to `None` cleanly — never crash, never block startup.

### Resolution pipeline

```
foreground process
  → normalize (strip .exe, lowercase, resolve Steam appid from path)
  → look up in registry/apps.json
  → if MISS: report Activity::None        ← default deny
  → if HIT:  check user's per-app hide list
  → if hidden: report Activity::None
  → else: report Activity::App { registry_id }
```

`registry/apps.json`:

```json
{
  "version": 1,
  "apps": [
    { "id": "cs2",     "label": "Counter-Strike 2", "kind": "game",
      "match": { "steam_appid": 730 } },
    { "id": "firefox", "label": "Firefox", "kind": "browser",
      "match": { "exe": ["firefox", "firefox-bin"], "bundle": ["org.mozilla.firefox"] } },
    { "id": "blender", "label": "Blender", "kind": "creative",
      "match": { "exe": ["blender"], "bundle": ["org.blenderfoundation.blender"] } }
  ]
}
```

Ship ~200 entries covering the top games, browsers, creative tools, editors, and media
players. `kind` drives roster iconography. Users add unknown apps to a **local** override
file; those entries never sync to the server.

### Polling

Poll at 3s while focused, 15s while unfocused. Debounce: an app must be foreground for
20 continuous seconds before it is reported, which suppresses alt-tab noise.

---

## 7. Security

### Threat model, stated plainly — publish this verbatim in the README

> Messages and files are encrypted in transit (TLS) and at rest on the host's disk.
> **The person running the server can read everything on it.** There is no
> end-to-end encryption. Run your own server, or trust the person who runs the one
> you're on. If you need cryptographic guarantees against your host, use Signal.

**Do not implement partial E2EE.** Real E2EE with multi-device, searchable history, and
file sharing means MLS (RFC 9420) plus device verification, key transparency, and backup
escrow — a larger project than everything else in this repo combined. Half-implemented
E2EE launders a false promise, which is worse than an honest limitation.

### Baseline requirements

1. **Passwords:** argon2id, `m=19456, t=2, p=1` minimum. Never SHA/bcrypt.
2. **Tokens:** access JWT, 15 min TTL, `EdDSA`. Refresh token, 30 days, rotating, stored
   hashed. Reuse of a rotated refresh token revokes the whole family.
3. **Client token storage:** OS keyring via `tauri-plugin-stronghold` or the `keyring`
   crate. **Test the headless / no-wallet fallback path explicitly** — a Linux box with
   no KWallet or gnome-keyring unlocked must degrade to a clear prompt, not a crash.
4. **No open registration.** Invite code required, always. Codes are 12 chars from a
   CSPRNG, single-use by default.
5. **Rate limits:** login 5/min/IP, message send 10/10s/user, upload slot 20/hour/user,
   invite creation 10/day/user, knock 3/hour/target.
6. **CORS is an allowlist, not a wildcard.** The client is a webview page, so it is
   a cross-origin caller and the server must grant it permission explicitly. The
   allowed origins are the Tauri app's (`tauri://localhost`, and
   `http(s)://tauri.localhost` on Windows) plus Vite's dev server. Reflecting any
   origin would let a website you happened to visit probe whether this server
   exists, which is worth avoiding for a product that is otherwise this private.
7. **Tauri capabilities:** the WebView gets the minimum permission set. Activity
   detection is exposed as exactly one narrow command returning a resolved
   `Activity`. The WebView can never enumerate processes.
8. **Signed auto-updates.** Tauri's updater with a signing key generated at M0 and
   **backed up offline**. Losing it means you can never ship an update.
9. **No telemetry.** Not opt-in, not anonymous, not crash reporting. None.

### User content is hostile

- **Serve uploads from a separate origin** (`cdn.linger.example`, not
  `linger.example/files`). Uploaded SVG and HTML are XSS vectors, and same-origin serving
  makes them session-stealing vectors.
- Force `Content-Disposition: attachment` and `X-Content-Type-Options: nosniff` on
  anything not on the image/video/audio allowlist.
- Re-encode images server-side. This strips EXIF (spec §4.10) and neutralizes polyglot
  files in one step.
- Strict CSP on the app origin: no `unsafe-inline`, no remote script, no remote fonts.
- Markdown rendering: allowlist-based sanitizer, no raw HTML passthrough, ever.

---

## 8. File pipeline

Never proxy bytes through the app server.

```
1. Client  → POST /uploads          { filename, size, mime }
2. Server  → validates quota, creates attachment(state=pending),
             returns { upload_id, url, method, headers, part_size }
3. Client  → PUT direct to object store (multipart if > 8 MB, resumable)
4. Client  → POST /uploads/{id}/complete
5. Server  → verifies size, sniffs real MIME, re-encodes image (strips EXIF),
             generates blurhash + poster frame, sets state=complete
6. Server  → attaches to message, fans out over gateway
```

**Storage backends:**
- `local` — filesystem under the data dir. Default. Correct for a home server.
- `s3` — any S3-compatible endpoint.

Both sit behind one `ObjectStore` trait (`crates/linger-server/src/storage/`). It has
four jobs: hand out a slot, gather an upload's parts into one local file the server can
inspect, store/read/delete a finished object, and throw away the parts of an upload that
died. Everything that decides *whether* bytes are acceptable is in `media`, not storage.

**The S3 backend does not use S3's own multipart upload.** Each part is presigned to its
own key, `uploads/{upload_id}/{part}`, and `assemble` streams those down into one local
file. S3 multipart would assemble the object inside the bucket, and the server's next
move is to download it anyway — sniffing the real type and re-encoding an image both need
the bytes on local disk — so the file would cross the wire three times. It would also
introduce an upload id of S3's own, handed out by a network call, which is exactly the
per-upload state this design does not want to store. The finished object goes up as one
PUT; files are capped at 500 MB, well under S3's 5 GB single-PUT limit.

Serving from S3 is a redirect to a presigned GET, so bytes never cross the app process on
the way out either. The `Content-Type` and `Content-Disposition` that force a download for
anything off the inline allowlist are signed into that URL as `response-*` overrides,
because the app server is not the thing sending the response any more.

For cloud hosting, recommend **Cloudflare R2**: zero egress fees, which matters
enormously for a file-sharing app. Backblaze B2 is the runner-up. Plain AWS S3 will bite
you on egress.

**The local backend's listener.** Step 3 is trivial with S3 — the client PUTs at Amazon.
With a filesystem there is no second machine, so the local backend hands out URLs under
`PUT /upload/{upload_id}/{part}`, and objects come back from `GET /objects/{key}`.
Neither path is under `/api/v1`, neither reads an `Authorization` header, and neither
touches a session: the part URL is signed with an HMAC over the upload id, the part
number and an expiry, which is what an S3 presigned URL is. The signing key lives in the
data dir beside the JWT key, because it has to survive a restart or a resumed upload
would find every URL it was holding invalid — the exact case resumability exists for.

An upload id **is** an attachment id. The part layout is a pure function of the declared
size, so nothing about an in-flight upload needs its own table.

**Failing at step 5 is two different things.** Parts missing is the ordinary shape of a
dropped connection: the slot stays pending and the client sends what is missing. Anything
else — wrong size, a file that is not the type it claimed, an image that will not decode
— is final, and the parts go.

**ffmpeg is optional.** `ffprobe` supplies video and audio duration and video dimensions;
`ffmpeg` grabs the poster frame. A server without them stores media perfectly well and
simply has no poster. The published image installs them.

---

## 9. Deployment

Target: a non-expert friend gets a working server in under 15 minutes.

```yaml
# deploy/compose.yaml
services:
  linger:
    image: ghcr.io/OWNER/linger:latest
    restart: unless-stopped
    volumes: [ "./data:/data" ]
    environment:
      LINGER_DOMAIN: linger.example.com
      LINGER_DATA_DIR: /data
      LINGER_STORAGE: local
  caddy:
    image: caddy:2
    restart: unless-stopped
    ports: [ "80:80", "443:443" ]
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
volumes: { caddy_data: {} }
```

Caddy is bundled specifically so TLS certificates are automatic. A self-hoster should
never have to think about certbot.

**First-run flow:** binary starts, finds no config, prints a one-time host-setup URL with
a token to stdout. Host opens it, creates their account, names the server. No env-var
bootstrap credentials.

**Backup:** the entire server is `data/linger.db` plus `data/objects/`. Document
`sqlite3 linger.db ".backup"` and a cron one-liner in the README. Do not build a backup
feature.

**Locked out:** no reset email, no reset link — the setup token only exists while the
server has no users, so the only remaining proof of ownership is access to the box.
`linger-server reset-password <username>` sets a new password and revokes that user's
refresh families. The password is generated and printed, or read from stdin; never
taken from argv, which leaks into shell history and `ps`. Stop the server first — one
SQLite file has one writer.

---

## 10. Build milestones

Each milestone is independently verifiable. Do not start the next until the current one
passes its check.

| # | Milestone | Done when | Estimate |
|---|---|---|---|
| **M0** | Workspace scaffold, CI, `ts-rs` type export, Tauri shell opens on all 3 OSes | `cargo test` and `pnpm build` green in CI; blank window opens on Linux/Win/macOS | 1 day |
| **M1** | Server: auth, invites, rooms, messages | Integration test suite drives the full REST surface with `reqwest`. No UI yet. | 2–3 days |
| **M2** | Gateway: WS, heartbeat, sequence numbers, resume | Test client survives a forced disconnect mid-stream and replays without gaps or duplicates | 2 days |
| **M3** | Client: message list, composer, session grouping, aging, density modes | Two clients on one machine exchange messages in real time | 4–5 days |
| **M4** | Presence + roster + in-room state + statuses | Roster updates live across two clients; a status set on one shows up on the other | 2 days |
| **M4.5** | Host controls (rooms, invites, server settings), member settings, the server list | A host who has only seen the app can create a server, add a room, invite a friend, and rename the server — no curl, no docs | 2–3 days |
| **M5** | Uploads, media pipeline, the media collection, status images | 400 MB video uploads, resumes after a killed connection, appears in the media grid | 3 days |
| **M6** | Styling: names, statuses, 16-color palette, themes, fonts | A user sets a gradient name from two palette keys; contrast is verifiably ≥4.5:1 in both themes | 2–3 days |
| **M7** | Packaging: installers, signing, notarization, auto-update | A signed installer for each OS, and an update ships end-to-end | 3–5 days |
| **M8** | Export | One archive contains every message and file, and it opens | 1 day |

**Do the activity-detection spike first, before M0.** Spend one evening writing a
throwaway Rust binary that prints the foreground app every second on
Kubuntu/Plasma 6 Wayland, then on Windows. If that is pleasant, the project is
real. If Wayland eats the whole evening, you have learned the most important
thing about the timeline for the cost of one night. That spike is retired
(2026-08-19). The real backends are on the backburner.

**Entrance sounds moved to the end of the queue** (Matt, 2026-08-21). They are still
V1 (SPEC §6, item 4) — they are simply the last thing built, after M8, and M4's check
no longer waits on them. `TASKS.md` holds them under *Backburner* as T-901…T-903
(they were T-403, T-404, T-408).

**Activity detection is off the critical path** (Matt, 2026-08-23). It is still
V1 (SPEC §6, item 8). The spikes are retired and the crate is in the tree, but the
real backends, poller, registry and sharing UI are not needed for a usable product
and they are large. It used to occupy M5 / T-501…T-507; those tasks are
**T-911…T-917** now. **M5 (uploads) starts when M4.5's check passes.** Do not
start T-911 until Matt takes this off the backburner.

**M4.5 was added on 2026-08-21**, after the client turned out to have no way to create
a room, invite anybody, or edit the server — every endpoint for all three has existed
since M1 with nobody calling it. It also carries the server list from §3 of the spec
(V1 item 17), which had never been given a task at all.

M7 is not interesting and cannot be skipped. macOS notarization in particular is a
tedious, version-sensitive slog. Budget the full estimate.
