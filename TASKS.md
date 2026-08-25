# TASKS.md — the running work queue

This file is the handoff surface between the architect session (which maintains
structure and this file) and implementation sessions — run by any contributor,
with any coding agent. It is a live document: claim tasks, check them off, add
discoveries, and keep it truthful.

Tasks are marked **⬜ not started**, **⏳ claimed** (name + date), or **✅ done**
— emoji, not markdown `- [ ]` checkboxes, so the state of the queue is visible
at a glance while scrolling. Use the same characters when you add or move a task.

Task numbers match the milestone: T-5xx is M5, T-6xx is M6, and so on. Work that
is still V1 but not on the critical path lives at the end as T-9xx (sounds) and
T-91x (activity detection). A 2026-08-23 renumber, after activity detection left
the main sequence: old M6–M9 are M5–M8; entrance sounds T-403/404/408 are
T-901/902/903; activity T-501…T-507 are T-911…T-917.

**This file stays small on purpose.** Every fresh session reads it, so finished
history does not live here: when a milestone passes its check, the architect
moves its section — landing notes and all — to `docs/tasks/`. Those notes are
the project's memory; read an archive when a task below points at it, not by
default.

## How to run a task

Any coding agent (or human) can run a task — the repo is tool-agnostic. Every
agent's contract is `AGENTS.md`; tools that insist on their own filename get a
pointer (`CLAUDE.md`, `QWEN.md`, `GEMINI.md`), and tools that read `AGENTS.md`
natively need nothing. **One task per fresh session** — a clean context follows
the task spec better and costs less than a long-running one. The prompt that
works in every tool:

> Read AGENTS.md and TASKS.md, then do task T-xxx. State the current milestone
> first.

1. **Claim the task first**: push a branch named `feat/t-xxx-short-slug`, and in
   your first commit flip the task here from ⬜ to ⏳ with your name and the
   date. That is how two people avoid building the same thing. If a task has
   been ⏳ for over a week with no branch activity, ask in the PR or ping Matt
   before taking it over.
2. Pick your model and effort from the task's label using the table below.
3. Read `AGENTS.md` in full, then the spec sections the task references. State
   the current milestone before writing code.
4. Do not start a milestone until the previous one passes its check
   (`ARCHITECTURE.md` §10). Do not pull work from a later milestone "while
   you're in there."
5. When done: all listed acceptance criteria pass, `scripts/check.sh` is green
   locally, CI is green after push, the task is flipped ⏳ → ✅ here with a
   dated landing note, and any surprises are recorded under the task. The
   landing note is not optional — it is what makes the next fresh session work.
6. **Never add AI attribution anywhere — no exceptions, ever.** Not in commits,
   not in comments, not in metadata, not in the PR. The author of a commit is
   the person who ran the session, under their own git identity. A model is a
   tool, and tools do not sign work. Whatever agent you use, turn off its
   attribution trailers and PR footers; CI rejects what slips through.

**Effort mapping (task label → what to run it with):**

| Task label | What it needs |
|---|---|
| **low** | Any capable model at its default setting. Mechanical and tightly specified — a frontier model at max effort just burns money re-deriving what the task text already decides. |
| **medium** | A frontier model at its normal setting. Real features with judgment in the details. |
| **high** | A frontier model at a high-reasoning setting. Cross-cutting, but the architecture docs carry a lot of the load. |
| **treacherous** | The strongest model and highest reasoning setting available to you, and coordinate with Matt before claiming. Currently T-701+T-702 (signing/notarization) and, on the backburner, T-911 (Wayland/KWin). AGENTS.md §"Where you will be wrong" territory. |

Running everything at maximum is not better — it is slower, pricier, and prone
to overbuilding simple tasks. Match the effort to the label and escalate only if
a task fails its acceptance criteria twice.

---

## Status

Closed milestones are archived in `docs/tasks/` with every landing note and
surprise intact. Tasks T-001…T-415 live there. Decisions that shaped the queue
(the password floor, no host transfer, removal not banning, storage knobs as
environment variables) are in [`docs/decisions.md`](docs/decisions.md).

| Milestone | Closed | What stands | Archive |
|---|---|---|---|
| SPIKE + M0 — scaffold | 2026-08-19 | Workspace + CI; shell opens on all three OSes; both activity spikes retired with zero title exposure; `oklch()` gate answered yes | [m0.md](docs/tasks/m0.md) |
| T-006 — vocabulary | 2026-08-19 | The coined words are gone from code, UI, and docs | [t-006-vocabulary.md](docs/tasks/t-006-vocabulary.md) |
| M1 — server REST | 2026-08-19 | Auth, setup, invites, rooms, messages, styling, statuses; every endpoint integration-tested over real HTTP | [m1.md](docs/tasks/m1.md) |
| M2 — gateway | 2026-08-19 | WS with heartbeat, presence, typing; forced-disconnect resume with no gaps and no duplicates | [m2.md](docs/tasks/m2.md) |
| M3 — client: message stream | 2026-08-21 | Sign-in that sticks, live stream across two clients, 10k-message scrollback virtualized, composer, catch-up | [m3.md](docs/tasks/m3.md) |
| M4 — presence, roster, statuses | 2026-08-21 | The roster moves live; in-room/away/idle mechanics; the status card | [m4.md](docs/tasks/m4.md) |
| M4.5 — the shell's missing surfaces | 2026-08-25 | Host controls, invites, member settings, server list, remove + re-admit, password reset, live member announce | [m4-5.md](docs/tasks/m4-5.md) |

**Still open from closed milestones:** the visual "a window opens" sign-off on
Linux, Windows and macOS (T-002/T-003 in `docs/tasks/m0.md`) is a human errand,
outstanding, and must be closed before M7.

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


## M5 — uploads, media pipeline, the media grid

*Milestone check: a 400 MB video uploads, resumes after a killed connection, appears in the media grid.*

- ✅ **T-501 · Upload pipeline (local backend)** — effort: **high** — landed 2026-08-25
  ARCHITECTURE §8 + PROTOCOL §6. Slot creation validates size/quota/MIME
  allowlist; token-authenticated direct-PUT URLs (bytes never traverse app
  routes — separate upload listener path); multipart >8MB with per-part URLs
  (this is the resumability); complete: re-verify size, sniff real MIME,
  re-encode images (kills EXIF + polyglots in one step — `image` crate),
  blurhash, video poster via ffmpeg. Reject oversize at slot *and* at complete.
  *Accept:* the milestone check, scripted: kill mid-upload, resume, complete;
  EXIF-GPS test image comes out clean; fake-MIME file is caught.

  All three accept criteria are `crates/linger-server/tests/uploads.rs`
  (15 tests): `a_killed_upload_resumes_and_completes` sends a body that dies
  mid-part, proves complete refuses, resends and completes;
  `exif_never_survives_an_upload` builds a real JPEG with a hand-written EXIF
  APP1 + GPS IFD and asserts nothing of it is in the stored bytes;
  `a_file_that_lies_about_its_type_is_refused` sends zip bytes declared as PNG.

  **Decisions and surprises, for whoever picks up T-502…T-506:**

  - **An upload id is an attachment id.** Same UUID, two newtypes, no `uploads`
    table and no migration. Nothing about an in-flight upload needs storing that
    the attachment row does not already hold, and the part layout is a pure
    function of the declared size (`storage::part_plan`), so a resumed upload
    recomputes the plan rather than looking it up.
  - **Failing at complete is two different things**, and getting this wrong is
    what the first version did. Parts missing = the ordinary dropped connection:
    the slot stays pending, the client sends what is missing, complete again.
    Anything else (wrong size, a file that isn't the type it claimed) = final,
    parts discarded. Without that split, one flaky part burned the whole upload.
  - **A message with a file on it may have an empty body.** PROTOCOL §4 said
    1–8000 chars, which would have made "share a photo without a caption"
    impossible. PROTOCOL §4 is updated in the same commit; `validate::caption`
    is the version that allows empty and `validate::message_body` still doesn't.
  - **`ObjectStore` exists with one implementation** (`storage::LocalStore`), so
    T-502 has a target: slot, assemble-into-a-local-file, put/read/delete,
    discard. `assemble` returning a local path is deliberate — S3 will download
    the completed object there, because sniffing and re-encoding need the bytes.
  - **The listener is `PUT /upload/{id}/{part}` and serving is
    `GET /objects/{key}`**, both outside `/api/v1`, neither authenticated. Part
    URLs carry an HMAC over (upload id, part, expiry); the key is
    `data/upload_hmac.key` and must survive restarts or resume breaks. Serving
    is unauthenticated on purpose — the key holds a UUIDv7, the URL is the
    secret, and that is the only arrangement an `<img>` tag can use. What makes
    it safe is the headers, and **T-503 is what finishes it** by moving these
    responses off the app origin.
  - **WebP comes back as PNG.** The `image` crate reads WebP and cannot write
    it. Rather than skip re-encoding (and keep EXIF), a WebP is re-encoded to
    PNG and its filename extension corrected. GIFs are re-encoded frame by frame
    so animation survives. AVIF is not on the allowlist: decoding it needs a
    native dav1d build, which is not worth a system dependency yet.
  - **`server_config['pool_bytes']` already works** — `repo::attachments::
    pool_limit` reads it and falls back to the 50 GB default. T-505 needs the
    endpoint that sets it and the used figure on `GET /server`;
    `repo::attachments::pool_used` is the used figure, and it counts pending
    uploads so a full server cannot hand out fifty more slots.
  - **Abandoned uploads are swept on the way in**, not by a background task:
    slot creation deletes pending rows older than 48h and their part files. It
    is the only moment the answer matters (pending bytes count against the
    pool) and slot creation is 20/hour/person, so it is nowhere near hot.
  - **ffmpeg/ffprobe are optional and shelled out to.** No poster and no
    duration without them; nothing fails. `deploy/Dockerfile` installs ffmpeg
    now, so T-701's note about adding it is done. The video test skips when
    ffmpeg is absent, so **CI is currently not exercising the poster path** —
    add ffmpeg to the CI runner in T-701 and it starts running.
  - The scripted resume test moves 16 MB over three parts, not the milestone's
    400 MB over fifty. Same loop, fifty times; `storage::tests` pins the part
    arithmetic for 400 MB and 500 MB. **A real 400 MB video over a real network
    is still a human check** and belongs at the end of M5, with T-504's grid.
  - `LINGER_STORAGE=s3` now refuses to start rather than booting a server whose
    uploads cannot work. T-502 removes that.

- ✅ **T-502 · S3 storage adapter** — effort: **medium** — landed 2026-08-25
  Same `ObjectStore` trait, presigned URLs, `LINGER_STORAGE=s3` now boots.
  `crates/linger-server/tests/s3.rs` (7 tests) drives the public endpoints
  against a real MinIO; CI has an `s3` job that starts one, and
  `scripts/minio-test.sh` does the same locally.

  **Decisions and surprises, for whoever picks up T-503…T-506:**

  - **No S3 multipart upload, on purpose.** Each part is a presigned PUT to its
    own key, `uploads/{upload_id}/{part:05}`, and `assemble` streams them down
    into `data/staging/`. S3's own multipart would assemble the object inside
    the bucket, and the next thing the server does is download it anyway —
    sniffing and EXIF-stripping need the bytes locally — so the file would
    cross the wire three times. It would also hand out an upload id of S3's
    own, which would have to be stored: exactly the per-upload row T-501 got
    rid of. ARCHITECTURE §8 is updated to say this.
  - **A server on S3 still needs a data directory.** `linger.db` and the JWT
    key live there, and every upload passes through `data/staging/` on its way
    to the bucket. It is deleted immediately afterwards, whether the upload
    succeeded or was thrown away, but the disk has to be able to hold one file.
  - **`read_object` now takes a `ServeAs`.** The download-forcing headers
    (ARCHITECTURE §7) used to be set by the route, which works while the route
    is the thing sending bytes. With S3 it is a redirect, so the route works
    the two headers out and the store signs them into the presigned URL as
    `response-content-type` / `response-content-disposition`. The local backend
    ignores the argument. **`X-Content-Type-Options: nosniff` cannot be signed
    into an S3 URL** — S3 has no `response-` override for it — so on S3 that one
    header is missing until **T-503** puts these responses behind the CDN host,
    where the proxy can add it. Worth knowing before T-503 is called done.
  - **Serving is still `GET /objects/{key}` and then a 307.** The redirect keeps
    one URL shape in `Attachment.url` for both backends and keeps the row lookup
    that knows the filename and mime. It costs a round trip per image; if that
    ever matters, the fix is a public bucket domain, not a change to the client.
  - **`rusty-s3` + `reqwest`, not the AWS SDK.** `rusty-s3` only builds and signs
    URLs — no sockets, no runtime of its own — which is a much smaller thing to
    carry into a binary that already has an HTTP stack. Presigned URLs are the
    whole S3 API surface this needs.
  - **`reqwest` is a real dependency now**, not just a dev one, with
    `rustls-tls`. Integration tests can still use it: cargo gives test targets
    the regular dependencies as well as the dev ones.
  - **MinIO in CI is `docker run`, not a `services:` block.** A service container
    cannot pass a command, and the MinIO image needs `server /data` to start.
    The image tag is pinned.
  - **The S3 tests skip when `LINGER_TEST_S3_ENDPOINT` is unset**, printing a
    line saying so. That is what keeps `cargo test --workspace` green on a
    laptop — and it means a green workspace run proves nothing about this
    backend. Run `scripts/minio-test.sh` before claiming it works.
  - The test variables are `LINGER_TEST_S3_*`, deliberately not `LINGER_S3_*`.
    A test must not be able to write into a real bucket by inheriting the
    environment of the machine it runs on.
  - Still a human check, as with T-501: **a real 400 MB video into a real
    bucket over a real network.** The tests move 16 MB over three parts.
- ⏳ **T-503 · Separate media origin** — effort: **medium** — Matt, 2026-08-25
  ARCHITECTURE §7: serve objects on the cdn host; `Content-Disposition:
  attachment` + `nosniff` off-allowlist; activate the Caddyfile block; strict CSP
  on the app origin.
  **Carries a known gap from T-502:** on `LINGER_STORAGE=s3` the response comes
  from the bucket, and S3 has no `response-` override for
  `X-Content-Type-Options`, so `nosniff` is currently absent on that backend.
  The content type and disposition are signed into the presigned URL and do
  arrive. Closing that gap is part of this task, not an extra.
- ⬜ **T-504 · The media UI + link cards** — effort: **medium**
  SPEC §4.4: grid, filter by person/type/date, stars (starred never expire),
  each item links to its message/moment. Restrained link embeds (favicon, title,
  domain — one line): server-side metadata fetch **with SSRF guard** (deny
  private ranges, cap size/time), cached.
- ⬜ **T-505 · Expiry + storage accounting** — effort: **medium**
  365-day expiry of non-starred/non-pinned (host-configurable/off), background
  task; storage-used figure for the status bar and `GET /server`.
  The pool and expiry knobs are environment variables, not a config file —
  decided in [`docs/decisions.md`](docs/decisions.md).
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
  ghcr.io workflow for `deploy/Dockerfile` (ffmpeg is already in it, T-501; the
  CI *runner* still needs it, or the video-poster test keeps skipping),
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


## Parking lot (decisions needed, not tasks yet)

- Bundle identifier is `com.linger.desktop` — fine? Changing after M7 is painful.
- `MediaItem` wire shape is minimal (attachment + message/room link) — revisit
  when T-504 starts if the grid needs more.
- Link-preview fetching is host-side (privacy: the host's IP fetches, not each
  member's). Confirm this trade-off is intended before T-504.
