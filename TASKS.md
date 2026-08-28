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
| **treacherous** | The strongest model and highest reasoning setting available to you, and coordinate with Matt before claiming. Both are on the backburner: T-705 (signing/notarization, blocked on
certificates) and T-911 (Wayland/KWin). AGENTS.md §"Where you will be wrong" territory. |

Running everything at maximum is not better — it is slower, pricier, and prone
to overbuilding simple tasks. Match the effort to the label and escalate only if
a task fails its acceptance criteria twice.

---

## Status

Closed milestones are archived in `docs/tasks/` with every landing note and
surprise intact. Tasks T-001…T-604 live there. Decisions that shaped the queue
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
| M5 — uploads, media, the grid | 2026-08-26 | Resumable uploads on local or S3, files served from their own origin, the media grid with stars and link cards, expiry + a storage ceiling, the status image | [m5.md](docs/tasks/m5.md) |
| M6 — styling, themes, fonts | 2026-08-27 | Names drawn from custom properties, the two-click style picker, dark/light/system + evening warmth, twelve faces vendored — contrast ≥4.5:1 guarded in CI against four backgrounds | [m6.md](docs/tasks/m6.md) |
| M7 — packaging and updates | tasks 2026-08-27 | Signed updater behind two Rust commands, tag → draft release with Linux and Windows installers, the shipped CSP stops at the app's own server, the server image publishes to ghcr for x86-64 and ARM64. **Its check is still open** — see below | [m7.md](docs/tasks/m7.md) |

**Still open from closed milestones — four human errands, all outstanding, and
one tag knocks out most of them.** `release.yml` builds Linux and Windows
installers, `image.yml` publishes the server, and T-701's updater is already
proven end to end against a throwaway manifest:

- **M7's own check** (`docs/tasks/m7.md`): *one auto-update, installed, on a
  machine that did not build it.* Every M7 task is done and archived; this is
  the part no test can do. Cut a tag, publish the draft release, install the
  bundle somewhere else, then cut the next tag and watch that machine take the
  update. **The first tag is also the first time `image.yml` builds arm64 and
  the first time a `docker compose pull` has anything to pull** — worth a
  manual run of that workflow from the Actions tab first, and remember the ghcr
  package starts private and has to be made public by hand once.

- The visual "a window opens" sign-off on Linux, Windows and macOS
  (T-002/T-003 in `docs/tasks/m0.md`). Was meant to close before M7. Installing
  the bundles from a `v0.1.0` draft release closes **the Linux and Windows two
  thirds** of it and exercises the release path at the same time; the Windows one
  carries a SmartScreen warning, which is expected (see
  [`docs/decisions.md`](docs/decisions.md)) and not a reason to wait. The macOS
  third stays open, because v0.1.0 does not build a macOS bundle — it needs
  somebody with a Mac and a `cargo tauri build` from a checkout.
- **M5's milestone check itself** (`docs/tasks/m5.md`): a real 400 MB video,
  over a real network, from a server with real DNS for both names. Every piece
  of it is covered by tests, but nobody has clicked `+ file` in a running app —
  which also leaves the media grid, the storage figure in the status bar, and
  the status image at 400×200 on a second client unseen by a person. T-504,
  T-505 and T-506 each deferred it and each said so.
- **A styled name in the message stream** (`docs/tasks/m6.md`). The roster, the
  style picker, themes and warmth were all driven live, but the gateway is a
  Tauri IPC channel, so the browser sessions that verified them had no rooms and
  no messages. The stream's names and the per-person message font have only ever
  been checked as computed styles. Same code path as what was verified; still
  nobody has watched a styled name go past in a room.

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
M5 adds the whole of uploads (see [m5.md](docs/tasks/m5.md)): the `ObjectStore`
trait with a local and an S3 backend, resumable part uploads, the complete step
that re-encodes and sniffs, `GET /media` with keyset paging, link cards behind
an SSRF guard, the expiry sweeper, and the status image.
M6 adds the whole of styling (see [m6.md](docs/tasks/m6.md)): `palette.generated.css`
written out of `linger-core` by a unit test and drift-checked like the bindings,
names painted from `--person-*` custom properties in `styles/names.css`, the
style picker, theme + evening warmth as attributes on `<html>`, and the twelve
faces vendored under `client/src/fonts/` — **never write a hex or `oklch()`
literal into the frontend, and never add a remote font URL.**
M8 adds the export (T-801 below): `POST /export` and `GET /export/:job_id`,
the archive written on a blocking thread and served from the media origin like
any other object, and `repo::messages::batch_ascending` for walking a room
forwards. **Do not rebuild it** — what is missing is only the button, T-802.
M7 adds the release path (see [m7.md](docs/tasks/m7.md)): the signed updater
behind two narrow Rust commands, `release.yml`, `image.yml` publishing the
server to ghcr on a tag, the four-file version check and `signing-preflight`,
and a shipped CSP that reaches the app's own server and nothing else — which is
why **a server needs a name; a bare `IP:port` is unreachable from anything
anybody installed**, and says so at startup. **Updates are signed; installers
are not** — that is a decision, not a gap
([`docs/decisions.md`](docs/decisions.md)), and macOS is deliberately not built
at all until T-705.

---


## M8 — export

*Milestone check: one archive contains every message and file, and it opens.*

- ✅ **T-801 · Full export** — effort: **medium** — landed 2026-08-27

  **Landing note.** `POST /export` writes a row, spawns a task and hands back an
  id; `GET /export/:job_id` says how far along and, when there is one, where to
  download it. The archive is served from `/objects/...` on the **media
  origin** — the same host uploads come from — because the whole server in one
  file has no more business being same-origin with the app than a photo does.
  The origin split is tested, not assumed: on a named server the archive
  answers on `cdn.` and 404s on the app's own name.

  **What is in the zip**, under one top-level folder so unzipping it does not
  scatter files across somebody's downloads: `rooms/<slug>.md` per room in the
  order things were said, with a divider and a heading per day and every
  message as `**HH:MM** — Display Name (@username)`; a reply quoting what it
  answered, because a transcript where replies point at nothing is the least
  readable kind there is; `media/` with every file
  under its own name; `media.md` indexing who shared what, when and in which
  room; and a `README.md` that says what the archive is and that it needs
  nothing from this project to read. Times are UTC and the archive says so —
  the server does not know what timezone a reader is in, and quietly writing
  its own would be worse than naming the one it used.

  **Three decisions worth knowing.**
  *Deleted messages stay deleted.* A tombstone is skipped; an archive that
  resurrects what somebody deleted would be a worse product.
  *One archive per member.* Asking again deletes the previous one's bytes
  first, so a member with a button cannot fill a host's disk with copies of
  their own server. An old `url` stops working, and there is a test for that.
  *Archives are not attachments.* They are not `attachments` rows, do not count
  against `LINGER_POOL_BYTES`, and never appear in the media grid. Their keys
  (`exports/<id>.zip`) cannot be mistaken for an attachment key by
  `storage::key_owner`.

  **The zip is written on a blocking thread.** `zip` is synchronous and an
  archive is hundreds of megabytes; doing it on the reactor would stall every
  other connection. Media entries are `Stored` rather than deflated — a JPEG is
  already compressed, and running deflate over one spends the CPU of the whole
  export to save a fraction of a percent.

  **Uploaded filenames are somebody else's text**, so `archive_filename` is
  also the zip-slip guard: no directories, no `..`, nothing that can climb out
  of `media/` when an unzipper puts it back on a disk. Two people who both
  shared `IMG_0001.jpg` both keep their name (` (2)`).

  **Dates are hand-written rather than a dependency.** One format in one file
  did not justify a date crate; `civil_date` is Hinnant's algorithm with tests
  for the leap day, the 1900/2000 century rule and a pre-epoch date.

  **What is not proven here.** The S3 path — an export on a bucket has to pull
  every file back out through a presigned URL, which the local backend never
  does — has a test in `tests/s3.rs`, and that file skips itself without a
  bucket. It has only run in CI's `s3` job, never on this machine. And nobody
  has exported a server with a year of real video in it; the tests use small
  files.

  **Also added:** `repo::messages::batch_ascending`. `page` is newest-first
  because the stream reads that way, and its `after` still orders `DESC` — the
  newest messages after a point rather than the next ones. An archive reads
  forwards, so it gets its own query rather than a flag on that one.
- ✅ **T-802 · The export button** — effort: **low** — landed 2026-08-28

  **Landing note.** Settings, under *take everything with you*: one button that
  starts an export, a line that says what is happening, and a second button
  that hands the finished archive to the system browser once there is one.

  **The download goes out of the app, not into it.** `openExternal`, the same
  path a link in a message takes — a WebView that navigates itself to a zip has
  left the application, taking the signed-in session with it.

  **A refusal is a sentence, not an error.** A second export inside the hour
  comes back `RATE_LIMITED` with `retry_after_ms`, which the panel says as "you
  can ask again in about 50 minutes". Rounded up and vague on purpose. A server
  that refuses without saying when falls back to the documented hour.

  **Polling, not a gateway frame.** A job belongs to the one person who asked
  for it; putting progress on the socket every member shares would tell the
  whole server whenever anybody takes a copy. Closing the panel stops the
  asking and leaves the job running — it is being built for a person, not for a
  window.

  Twelve tests in `client/src/settings/export.test.ts` cover the wording, the
  refusal, the poll-to-completion, and that nothing is said after the panel
  closes. The logic is in `settings/export.ts` so it is testable without
  rendering; the component is the thin part.

  **Not done here:** nobody has clicked it in a running app. It is on the
  human-checks list.

---

## Backburner — later, not the next thing

Three things live here: two V1 features that are still in the spec, and one
release errand that is blocked on money. None of them is on the path to a
usable product. Do not pull any of them "while you're in there."

### Signing and notarization

*Moved here 2026-08-27 by Matt, from M7. Nothing in the milestone waits on it:
M7 closes on Linux and Windows, unsigned, by decision
([`docs/decisions.md`](docs/decisions.md)). It comes off the backburner the day
somebody buys the certificates, not before.*

- ⬜ **T-705 · Windows signing + macOS notarization** — effort: **treacherous**
  **Blocked on money, not on effort** — a Windows OV certificate (a few hundred
  a year) and an Apple Developer Program membership ($99/year). Do not start
  without both; there is nothing to test against.
  Windows: certificate into a repository secret, signing wired into the bundler,
  the workflow refusing to publish an unsigned installer. macOS: Developer ID
  signing plus `notarytool` and a stapled ticket, then uncomment the two macOS
  entries in `release.yml` — **in the same pass, never before**. Follow current
  vendor docs, not memory; this is the version-sensitive slog AGENTS.md warns
  about.
  *Accept:* a downloaded installer raises no warning on either OS, and a macOS
  copy installed from it takes an auto-update without being killed for a
  signature mismatch.

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
**T-911…T-917** now, so they never collided with M5's own tasks (archived in
[`docs/tasks/m5.md`](docs/tasks/m5.md)). Still V1 (SPEC §6,
item 8). The Linux and Windows spikes are already retired, `linger-activity`
already compiles, and the Null backend already reports nothing — which is the
correct product until this comes back. It is not needed for a usable chat app,
and it is large: four OS backends, a poller, a registry, and a sharing-controls
UI. **Do not start T-911.** M5 and M6 are closed; M7 (packaging) is the current
milestone.*

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
- **Nothing in the client can pin a message.** The server has
  `POST /messages/{id}/pin`, the media grid has a `pinned` filter, and file
  expiry spares "starred or pinned" files — but there is no pin control
  anywhere in the app and no `pinMessage` in `client/src/lib/api.ts`. So the
  filter is always empty and half of the expiry promise is unreachable. Add a
  control, or drop the filter and the half-promise. Found while writing the
  guides, 2026-08-27.
- Link-preview fetching is host-side (privacy: the host's IP fetches, not each
  member's). **Built that way in T-504** — the favicon is inlined as a `data:`
  URI so a reader's machine never touches the linked site either. Matt has not
  confirmed the trade-off; the cost is that the server's IP appears in the logs
  of every site anybody links, and turning it off means either no cards or
  every reader fetching for themselves.
