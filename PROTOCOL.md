# Linger — Wire Protocol v1

All REST under `/api/v1`. Gateway at `/api/v1/gateway` (WSS).
Content type `application/json` unless noted. Timestamps are Unix milliseconds (i64).
IDs are lowercase hex UUIDv7 strings.

**Rule: every type crossing this boundary is defined in `crates/linger-core` and exported
to TypeScript via `ts-rs`. The frontend never hand-writes a wire type.**

---

## 1. Errors

Every non-2xx response:

```json
{ "error": { "code": "INVITE_EXPIRED", "message": "That invite has expired.",
             "retry_after_ms": null } }
```

`code` is a stable SCREAMING_SNAKE identifier the client switches on. `message` is
human-readable and safe to display. Never leak internals in `message`.

Standard codes: `UNAUTHENTICATED`, `FORBIDDEN`, `NOT_FOUND`, `RATE_LIMITED`,
`VALIDATION_FAILED`, `INVITE_INVALID`, `INVITE_EXPIRED`, `QUOTA_EXCEEDED`,
`FILE_TOO_LARGE`, `UNSUPPORTED_MEDIA`, `CONFLICT`, `INTERNAL`.

---

## 2. Auth

Access token: JWT, EdDSA, 15 min TTL, sent as `Authorization: Bearer <jwt>`.
Refresh token: opaque, 30 days, **rotating**. Reuse of a consumed refresh token revokes
the entire token family and forces re-login.

```
POST /auth/register     { invite_code, username, display_name, password }
                     →  { access_token, refresh_token, expires_in, user }

POST /auth/login        { username, password }
                     →  { access_token, refresh_token, expires_in, user }

POST /auth/refresh      { refresh_token }
                     →  { access_token, refresh_token, expires_in }

POST /auth/logout       { refresh_token }              → 204

GET  /auth/invite/:code → { valid, stoop_name, expires_at }   # unauthenticated preview
```

`username`: `[a-z0-9_]{2,24}`, unique, immutable after creation.
`display_name`: 1–32 chars, mutable.
`password`: minimum 12 characters. Do not impose composition rules.

---

## 3. Linger and rooms

```
GET  /stoop              → { name, accent_key, icon_key, member_count, created_at }
PATCH /stoop             (host only) { name?, accent_key?, icon_key? }   # accent_key from PALETTE

GET  /rooms              → Room[]
POST /rooms              (host only) { slug, name, topic? }         → Room
PATCH /rooms/:id         (host only) { name?, topic?, position? }   → Room
POST /rooms/:id/archive  (host only)                                → Room
```

```ts
type Room = {
  id: string; slug: string; name: string; topic: string | null;
  position: number; archived_at: number | null;
  last_message_id: string | null;   // client compares to read marker
}
```

---

## 4. Messages

```
GET  /rooms/:id/messages?before=<id>&after=<id>&limit=<1..100>   → Message[]
POST /rooms/:id/messages   { body, reply_to?, attachment_ids? }  → Message
PATCH  /messages/:id       { body }                              → Message
DELETE /messages/:id                                             → 204
POST   /messages/:id/pin                                         → Message
DELETE /messages/:id/pin                                         → Message
PUT    /messages/:id/reactions/:key                              → 204
DELETE /messages/:id/reactions/:key                              → 204
```

`before`/`after` are message IDs, not timestamps. UUIDv7 sorts chronologically, so
pagination is a range scan. Results are always newest-first.

```ts
type Message = {
  id: string; room_id: string; author_id: string;
  body: string;                                // markdown source, unrendered
  reply_to: string | null;
  attachments: Attachment[];
  reactions: { key: string; count: number; user_ids: string[] }[];
  pinned_at: number | null;
  edited_at: number | null;
  deleted_at: number | null;                   // tombstone; body is "" when set
  created_at: number;
}
```

Reaction `key` is one of 12 fixed values defined in `linger-core::REACTIONS`. The server
rejects anything else. `count` is sent but the client renders weight, not the number
(SPEC §4.8) — it is present for accessibility labels and hover.

Edits are only permitted by the author. Deletes are permitted by the author or the host.
Deleted messages become tombstones; they are not removed, so reply chains survive.

**Read markers**

```
PUT /rooms/:id/read      { last_read_id }     → 204
GET /read                → { [room_id]: last_read_id }
```

The client sends this at most once per 5 seconds per room, debounced. **No count is ever
returned by the server.** There is no unread-count endpoint and one must not be added.

---

## 5. Users, styling, signs

```
GET   /users              → User[]              # all members of this stoop
GET   /users/:id          → User
GET   /me                 → User
PATCH /me                 { display_name?, style?, sign?, entrance_sound? }  → User
PATCH /me/password        { current_password, new_password }                 → 204

GET  /me/notify-rules     → NotifyRule[]
PUT  /me/notify-rules     { target_user_id, room_id | null }   → 204
DELETE /me/notify-rules   { target_user_id, room_id | null }   → 204
```

```ts
type User = {
  id: string; username: string; display_name: string;
  is_host: boolean;
  style: Style;
  sign: Sign | null;
  entrance_sound: string | null;      // bundled key or object key
  last_seen_at: number | null;
}

// One of the 16 named palette keys defined in linger-core::PALETTE. See SPEC §5.4.
// ember rust amber brass lime fern mint teal
// cyan  sky  azure indigo violet orchid rose slate
type ColorKey = string;

type Style = {
  font_key: string;                   // must be in linger-core::FONTS
  weight: 400 | 500 | 700;
  italic: boolean;
  fill: { kind: "solid"; color: ColorKey }
      | { kind: "gradient"; from: ColorKey; to: ColorKey };   // angle is fixed at 92°
  effect: "none" | "shimmer" | "glow";
  msg_font_key: string | null;
}

type Sign = {
  line: string | null;                // <= 240 chars
  reading: string | null;             // <= 80
  listening: string | null;           // <= 80
  working_on: string | null;          // <= 80
  image_key: string | null;
  away_message: string | null;        // supersedes `line` when set
  away_since: number | null;
}
```

### Palette validation (server-side, mandatory)

There is no runtime color clamping, because there are no arbitrary colors. The server
validates that every `ColorKey` and `font_key` is a member of `linger-core::PALETTE` and
`linger-core::FONTS` respectively, and rejects anything else with `VALIDATION_FAILED`.

Contrast safety is structural. The palette is defined once with theme-mirrored lightness:

```
dark theme:   oklch(0.76 0.13 <hue>)     // slate: chroma 0.02
light theme:  oklch(0.50 0.14 <hue>)
```

Every entry holds ≥4.5:1 against both theme backgrounds by construction. A property test
asserts this across all 16 keys × 2 themes and must run in CI — it is the guard against
someone "improving" a palette value later.

---

## 6. Uploads

```
POST /uploads              { filename, size_bytes, mime }
                        →  { upload_id, attachment_id, method, url,
                             headers, part_size_bytes, parts? }

POST /uploads/:id/complete { parts?: [{ number, etag }] }   → Attachment
DELETE /uploads/:id                                         → 204
```

Client PUTs bytes **directly to the returned URL**, never through the app server.
Files over 8 MB use multipart with per-part URLs, which is what makes uploads resumable.

Server rejects at slot creation: `size_bytes > 500 MB`, stoop pool over quota, or a mime
not on the allowlist. Server re-validates real size and sniffs actual MIME at complete —
never trust the declared values.

```ts
type Attachment = {
  id: string; filename: string; mime: string; size_bytes: number;
  url: string;                       // separate CDN origin, see ARCHITECTURE §7
  width: number | null; height: number | null; duration_ms: number | null;
  blurhash: string | null; poster_url: string | null;
  starred_at: number | null;
  uploader_id: string; created_at: number;
}
```

**The shelf**

```
GET /shelf?kind=image|video|audio|file|link|pin
          &author=<user_id>&before=<id>&limit=<1..100>   → ShelfItem[]
PUT    /shelf/:attachment_id/star                        → 204
DELETE /shelf/:attachment_id/star                        → 204
```

---

## 7. Invites, export, knock

```
GET    /invites          → Invite[]
POST   /invites          { expires_in_hours?, max_uses? }   → Invite
DELETE /invites/:code                                       → 204

POST /export             → { job_id }
GET  /export/:job_id     → { state, progress, url? }        # any member, 1/hour
POST /knock              { target_user_id }                 → 204   # V2
```

---

## 8. Gateway

`wss://<host>/api/v1/gateway`. JSON frames.

```ts
type Frame = { op: string; d: unknown; s?: number }
```

`s` is a monotonically increasing sequence number, present on server→client frames only.

### Handshake

```
S→C  { "op": "hello",  "d": { "heartbeat_interval_ms": 30000 } }
C→S  { "op": "identify", "d": { "token": "<access_jwt>", "client": "linger-desktop/0.1.0" } }
S→C  { "op": "ready",  "d": { "session_id", "user", "users": User[],
                              "rooms": Room[], "presence": PresenceEntry[] },
                              "s": 0 }
```

### Heartbeat

Client sends `{ "op": "heartbeat", "d": { "s": <last_seq> } }` every
`heartbeat_interval_ms` ± jitter. Server replies `{ "op": "heartbeat_ack" }`. Two missed
acks → client reconnects.

### Resume

```
C→S  { "op": "resume", "d": { "session_id", "token", "s": <last_seq> } }
S→C  { "op": "resumed", "d": { "replayed": <n> } }    # then replays missed frames
S→C  { "op": "invalid_session", "d": { "reason": "expired" } }   # → re-identify
```

The server holds a 500-frame ring buffer per session for 120 seconds after disconnect.
Beyond that, the client must re-identify and refetch.

### Client → server

| op | payload | notes |
|---|---|---|
| `presence.update` | `{ state, activity, away_message? }` | `activity` is the resolved registry id or `null`. **Never a window title.** |
| `room.sit` | `{ room_id \| null }` | `null` = stood up |
| `typing.start` | `{ room_id }` | server rate-limits to 1 per 4s per room |

### Server → client

| op | payload |
|---|---|
| `message.create` | `Message` |
| `message.update` | `Message` |
| `message.delete` | `{ id, room_id }` |
| `reaction.update` | `{ message_id, key, count, user_ids }` |
| `presence.update` | `PresenceEntry` |
| `room.occupancy` | `{ room_id, user_ids }` |
| `room.enter` | `{ room_id, user_id, entrance_sound }` — triggers the sound |
| `room.leave` | `{ room_id, user_id }` |
| `user.update` | `User` — display name, style, or sign changed |
| `room.create` / `room.update` | `Room` |
| `typing` | `{ room_id, user_id }` |
| `knock` | `{ from_user_id }` (V2) |

```ts
type PresenceEntry = {
  user_id: string;
  state: "sitting" | "around" | "idle" | "away" | "offline";
  room_id: string | null;
  activity: { registry_id: string; label: string; kind: string; since: number } | null;
  away_message: string | null;
}
```

### Fan-out rules

- Presence and occupancy go to every connected client. At this scale, no filtering.
- `room.enter` is sent only to clients currently sitting in that room. The receiving
  client applies its own mute rules and quiet hours before playing anything (SPEC §4.1).
- Message events go to all clients; the client decides what to render. There are no
  per-room permissions in V1, so there is nothing to filter on.

---

## 9. Versioning

The path carries the major version. Within v1, additive changes only: new optional
fields, new `op` values, new error codes. Clients must ignore unknown fields and unknown
`op` values rather than erroring.

Any breaking change means `/api/v2` and a client that speaks both during a transition
window.
