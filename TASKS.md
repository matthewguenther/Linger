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
surprise intact. Tasks T-001…T-506 live there. Decisions that shaped the queue
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

**Still open from closed milestones — two human errands, both outstanding:**

- The visual "a window opens" sign-off on Linux, Windows and macOS
  (T-002/T-003 in `docs/tasks/m0.md`). Must be closed before M7.
- **M5's milestone check itself** (`docs/tasks/m5.md`): a real 400 MB video,
  over a real network, from a server with real DNS for both names. Every piece
  of it is covered by tests, but nobody has clicked `+ file` in a running app —
  which also leaves the media grid, the storage figure in the status bar, and
  the status image at 400×200 on a second client unseen by a person. T-504,
  T-505 and T-506 each deferred it and each said so.

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

---


## M6 — styling: names, palette, themes, fonts

*Milestone check: a gradient name from two palette keys, contrast verifiably ≥4.5:1 in both themes (the CI property test already guards the values).*

- ⏳ **T-601 · Name rendering engine** — effort: **medium** — Matt, 2026-08-27
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
**T-911…T-917** now, so they never collided with M5's own tasks (archived in
[`docs/tasks/m5.md`](docs/tasks/m5.md)). Still V1 (SPEC §6,
item 8). The Linux and Windows spikes are already retired, `linger-activity`
already compiles, and the Null backend already reports nothing — which is the
correct product until this comes back. It is not needed for a usable chat app,
and it is large: four OS backends, a poller, a registry, and a sharing-controls
UI. **Do not start T-911.** M5 (uploads) is closed; M6 (styling) is the next
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
- Link-preview fetching is host-side (privacy: the host's IP fetches, not each
  member's). **Built that way in T-504** — the favicon is inlined as a `data:`
  URI so a reader's machine never touches the linked site either. Matt has not
  confirmed the trade-off; the cost is that the server's IP appears in the logs
  of every site anybody links, and turning it off means either no cards or
  every reader fetching for themselves.
