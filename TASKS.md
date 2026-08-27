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
| **treacherous** | The strongest model and highest reasoning setting available to you, and coordinate with Matt before claiming. Currently T-702 (signing/notarization) and, on the backburner, T-911 (Wayland/KWin). AGENTS.md §"Where you will be wrong" territory. |

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

**The check is met (2026-08-27).** A gradient name from `rose` and `amber` was
picked in the running client against a running server, saved, and came back on
the roster — screenshots and the engine's own computed styles are in the T-602
note. Contrast is guarded by the CI property test, which since T-603 reads all
16 keys against four backgrounds: both themes, cool and after dusk. Ready for
the architect to archive.

- ✅ **T-601 · Name rendering engine** — effort: **medium** — landed 2026-08-27
  Build step: emit `palette.generated.css` from `linger-core::palette::css_variables`
  (single source of truth; oklch or hex per T-002's verdict). Render styled names
  everywhere names appear; gradient fixed 92°; shimmer (4s linear)/glow honor
  `prefers-reduced-motion`, disabled in compact + IRC; "normalize everyone"
  toggle flattens names *and* message fonts.

  **Landing note.** `css_variables` now emits `oklch()` rather than hex, per
  T-002, and a new `palette::stylesheet()` wraps both themes into one file. A
  unit test writes it to `client/src/generated/palette.generated.css`, which is
  the same trick ts-rs plays next door — so `cargo test --workspace` regenerates
  it and CI's existing `git diff --exit-code client/src/generated` is the drift
  check. No new CI step. **Custom properties cannot carry a fallback
  declaration**: an unparsable value in `--name-azure` is not dropped at parse
  time the way a normal property's is, it fails later at substitution, so
  shipping hex *and* oklch was never an option and the file holds one value per
  key.

  A name is now drawn from custom properties, not from React. `lib/names.ts`
  turns a `Style` into `--person-*` values plus two data attributes;
  `styles/names.css` does all the painting. That split is what makes "normalize
  everyone" one attribute on `<html>` (`lib/normalize.ts`, toggle in settings
  beside density) instead of a flag threaded through every component that draws
  a person — a flag like that eventually misses one. Same reason the 92° angle
  and the density rules live there.

  **Verified in the real engine, not just in tests.** WebKitGTK 2.52.3 —
  what the shell renders in — reports: `--name-azure` = `oklch(0.76 0.13 255)`
  dark and `oklch(0.5 0.14 255)` light; a gradient name computes
  `linear-gradient(92deg, oklch(...350), oklch(...68))` clipped to the glyphs;
  shimmer computes `name-shimmer 4s linear`; compact and IRC report
  `animation-name: none` and `text-shadow: none` while the gradient *fill*
  survives (a fill is not an effect); normalize drops the background, the weight
  and the color back to the reader's default.

  Three judgment calls worth knowing about:
  - **The 3px gutter rule does not normalize.** It is SPEC §4.7's replacement
    for the avatar column, not name styling, and flattening it would leave the
    stream with nothing to identify a speaker by at a glance.
  - **IRC mode overrides the person's font on the nick column only.** That
    column is `width: 12ch`, so mixed faces would break the alignment the mode
    exists for. Color, weight and slant still come through. IRC also ignores a
    message-font override, because the mode defines its own body face.
  - **A name inside a line of mono metadata takes the color and stops** — the
    quoted author on a reply line, the uploader on a media tile. They use a
    lighter `.name-color` class that still answers to normalize. Mono is
    metadata-only (SPEC §5.2) and a shimmer inside a one-line quote is noise.

  **The font half is wired but unfed.** `--font-<key>` stacks for all twelve
  faces are in `tokens.css` and `lib/fonts.ts` guards the key, so a chosen face
  applies — but nothing is bundled until **T-604**, so every stack currently
  falls through to its generic. A name in Silkscreen looks like a name in the UI
  face today. That is the expected state, not a bug.

  **Not seen by a person in the running app** — *closed by T-602*, which drove
  the real client against a real server and watched a saved gradient land on the
  roster. What is still unseen is the **message stream**: the gateway is a Tauri
  IPC channel, so a browser gets no rooms and no messages, and the stream's names
  and the message-font override have only ever been checked as computed styles.
  Every one of them goes through the same `nameProps` and the same `.msg-body`
  rule as the surfaces that were verified live.
- ✅ **T-602 · Style picker + settings** — effort: **medium** — landed 2026-08-27
  Two-click named-color picker (mIRC energy, modern craft), font/weight/italic/
  effect, live preview, msg-font override. Server already validates keys.

  **Landing note.** `settings/StylePicker.tsx` over `settings/style.ts`, in the
  `you` panel under *how your name looks*. Everything is drawn as itself: each
  face is labelled in its own face, each weight shown at that weight, each
  swatch is the color, and the preview is your display name through the same
  `nameProps` the stream uses — so there is one rendering path, not two that can
  disagree. A gradient is the same sixteen swatches a second time, labelled
  `from` and `to`; no hidden "which one am I editing" mode, because the promise
  is two clicks and a mode you have to discover is a third.

  `style.ts` holds the part that can be wrong without looking wrong — draft ⇄
  wire, and `isDirty`, which compares through the wire shape so changing the
  second color of a *solid* fill correctly leaves the save button down.

  One deliberate exception in the CSS: the preview carries `name-raw`, which
  exempts it from the reader's own normalize and density settings. A preview
  that obeys "normalize everyone" shows you a flat name while you are picking a
  gradient. The panel says so in a line under the preview rather than silently
  lying.

  **Driven live, in the real engine.** `pnpm dev` in a plain browser is a
  supported way to work on the UI (`lib/ipc.ts`), so this was checked by loading
  the actual client off the actual Vite server against a real `linger-server` in
  WebKitGTK 2.52.3 — first-run setup, sign in, pick `rose`→`amber` + Newsreader
  + 700 + shimmer, save, and watch the name come back styled on the roster.
  Theme, warmth and normalize were flipped the same way: light reports
  `#F7F8F9`, warm-light `#FBF7F2` and warm-dark `#1A1714`, matching the Rust
  constants exactly; normalize flattens the roster and leaves the preview alone.

  **What that could not reach:** the gateway is a Tauri IPC channel, so a browser
  has no rooms and no messages. The stream's names and the message-font override
  are still computed-style checks only. They go through the same `nameProps` and
  the same `.msg-body` rule as everything that was verified live, but nobody has
  watched a styled name go past in a room.

- ✅ **T-603 · Themes + time-of-day warmth** — effort: **low** — landed 2026-08-27
  Light theme tokens exist; add the ~200K post-sunset warmth shift (one variable
  swap, user-disableable) and theme switching.

  **Landing note.** `lib/theme.ts` plus one block of tokens. Theme is dark /
  light / system, and `system` resolves through `prefers-color-scheme` and
  re-resolves when the OS changes its mind, so `data-theme` is always explicit
  and the CSS never had to learn a third state.

  **The warmth shift keeps lightness and moves only hue.** Every warm token is
  its cool twin's OKLab lightness with the hue rotated to the amber side. That
  is what makes it a temperature change instead of a dimming — and it is the
  reason the 16-color palette is still safe after dark. `linger-core::palette`
  now carries `DARK_BG_WARM` / `LIGHT_BG_WARM` and the CI property test reads
  all 16 keys against **four** backgrounds instead of two. Measured: the worst
  case moves from 7.79 to 7.83 (dark) and 4.89 to 4.87 (light). A contrast
  guarantee that lapsed at dusk would not be one.

  **Sunset is approximated by the clock, and that is a decision.** Knowing real
  sunset means knowing latitude and longitude. A chat client that asks for your
  coordinates to tint its background has made a bad trade, so warmth is on from
  19:00 to 07:00 local. At high latitudes in June that is wrong by a couple of
  hours; the whole effect is a 200K tint nobody is meant to consciously notice,
  which is why that is acceptable and a location permission is not. If Matt
  wants real sunset later, it needs a location and that is his call.

- ✅ **T-604 · Font pipeline** — effort: **low** — landed 2026-08-27
  Script: fetch the 12 faces (`assets/fonts/README.md` table), subset
  (latin/latin-ext, 400/500/700 + italics) to woff2, keep OFL texts,
  `@font-face` wiring. No CDN.

  **Landing note.** `scripts/fetch-fonts.sh` → `scripts/fetch_fonts.py`. Output
  is committed: 30 faces, **about 800 KB total** in `client/src/fonts/`, plus
  generated `@font-face` rules and the OFL texts in `assets/fonts/`. Nobody
  needs to run the script to build — run it when a face is added to
  `linger-core::FONTS` or to pull an upstream fix. The script refuses to run if
  its manifest and `linger-core::FONTS` disagree.

  Things worth knowing:
  - **Ten of the twelve come from `google/fonts`.** It is where those projects
    publish, its paths are stable, and it carries the OFL text next to the
    binary. Commit Mono and Departure Mono come from their own repos. The Source
    column in `assets/fonts/README.md` is each face's *home*, not the download
    URL — those live in the manifest.
  - **Variable fonts where they exist.** Seven faces publish one, so one file
    covers 400–700 and the rule declares `font-weight: 400 700`. Two-axis fonts
    (Plex Sans, Inter, Newsreader) get the second axis pinned before subsetting,
    which is most of the size saving.
  - **Nothing is faked.** Instrument Serif has one weight, Departure Mono has
    one and no italic, Silkscreen has no italic. Those ship as they are drawn
    and the browser synthesises if somebody asks for what does not exist.
  - **Debian and Ubuntu ship python3 without `ensurepip`**, so the plain
    `python3 -m venv` fails on exactly the machines this project is developed
    on. The script falls back to `--without-pip` and bootstraps pip. The venv
    lives under `target/` and nothing is installed system-wide.

## M7 — packaging and updates

*Milestone check: a signed installer per OS; one auto-update ships end-to-end.
Budget the full estimate; notarization is a version-sensitive slog — follow
current vendor docs, not memory (AGENTS.md).*

- ✅ **T-701 · Updater + signing keys** — effort: **high** — landed 2026-08-27
  Tauri updater; generate the signing key and **back it up offline before
  anything ships** (losing it = no more updates, ARCHITECTURE §7). Release
  workflow: tag → build 3-OS installers → publish manifest.

  **Landing note.** The updater is wired end to end in code, and one step is
  left that only Matt can do — see *the key* below.

  **The WebView is granted nothing.** `capabilities/default.json` does not list
  `updater:default`, so the page cannot call `plugin:updater|check` or
  `|install`. It calls two of the app's own commands in
  `client/src-tauri/src/updates.rs`, which is the same shape the gateway and the
  keyring already use. A page that can start an installer is not a page with a
  minimum permission set. The plugin still has to be *registered*, because that
  is what parses `[plugins.updater]` into somewhere `app.updater()` can read it.

  **A build with no key refuses every update, rather than taking an unsigned
  one.** `pubkey` ships as `""` in `tauri.conf.json` until the key exists, and
  `updates.rs` treats "no key or no endpoint" as its own answer —
  `unconfigured`, which the settings panel renders as "this copy was not built
  to update itself". Underneath that, the plugin's own `verify_signature` is
  unconditional and has no bypass, so even a mistake here fails closed.

  **The key is Matt's errand and is not done.** `scripts/updater-key.sh`
  generates it outside the repo (`~/.local/share/linger/updater.key` by
  default), refuses to write inside the working tree, refuses to overwrite an
  existing key, and stamps the public half into `tauri.conf.json` for
  committing. Then: back the private half and its password up offline, and add
  `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as
  repository secrets. Until that is done `release.yml` fails in its first job,
  on purpose — a release that quietly built unsigned bundles would be worse.

  **Releases are draft-first.** A tag builds Linux, Windows and both macOS
  architectures and opens a *draft* release carrying `latest.json`. Publishing
  it is what makes installed copies see the update, and that stays a human's
  click. The endpoint is this repo's `releases/latest/download/latest.json`,
  which resolves only to published releases.

  **One version number, three files** — `client/package.json`,
  `client/src-tauri/Cargo.toml`, `client/src-tauri/tauri.conf.json`.
  `scripts/version-check.sh` gates it, from `check.sh` and from CI's `rules`
  job, and `release.yml` checks the tag against it before building anything.
  This is the failure that looks like success: ship 0.2.0 under the old number
  and every installed copy decides it is already current.

  **Two things worth knowing.** Linux builds on `ubuntu-22.04`, not `latest` —
  a bundle linked against a newer glibc will not start on an older
  distribution, and the reverse is fine. And `createUpdaterArtifacts` is on in
  the committed config, so a local `pnpm tauri build` now needs
  `TAURI_SIGNING_PRIVATE_KEY` in the environment or it stops. `pnpm tauri dev`
  is untouched, and nobody's normal loop runs `tauri build`.

  **What has not been proven.** Nobody has installed a signed bundle and watched
  it update itself — that needs the key, a published release, and three
  operating systems, and it is M7's milestone check rather than this task's.
  What was checked here: the shell compiles and its tests pass against the real
  crate, the settings panel and status-bar line render, and both halves report
  `unconfigured` today because the key is genuinely absent. **The private repo
  matters for this**: GitHub release assets need no authentication only on a
  public repo, so before the first real release either the repo goes public or
  the endpoint moves to a host Matt controls. It is one line in
  `tauri.conf.json` either way — see the parking lot.
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
- **Where release downloads live.** The updater endpoint is this repo's
  `releases/latest/download/latest.json`, and the repo is private — GitHub
  release assets are only fetchable without a token on a public repo. So before
  the first release either the repo (or its releases) goes public, or the
  endpoint moves to a static host Matt runs. Anything that needs a credential in
  the client is not an option; shipping a token in a desktop app is shipping the
  token. One line in `client/src-tauri/tauri.conf.json` either way, but it has
  to be decided before a tag is pushed.
- Link-preview fetching is host-side (privacy: the host's IP fetches, not each
  member's). **Built that way in T-504** — the favicon is inlined as a `data:`
  URI so a reader's machine never touches the linked site either. Matt has not
  confirmed the trade-off; the cost is that the server's IP appears in the logs
  of every site anybody links, and turning it off means either no cards or
  every reader fetching for themselves.
