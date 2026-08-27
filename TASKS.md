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
| **treacherous** | The strongest model and highest reasoning setting available to you, and coordinate with Matt before claiming. Currently T-705 (signing/notarization, blocked on certificates) and, on the
backburner, T-911 (Wayland/KWin). AGENTS.md §"Where you will be wrong" territory. |

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

**Still open from closed milestones — three human errands, all outstanding.
The first real tag chips at the oldest of them**, since `release.yml` builds
Linux and Windows installers and T-701's updater is already proven end to end
against a throwaway manifest:

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
M7 adds the release path (see T-701 below): the signed updater behind two narrow
Rust commands, `release.yml`, and the `version-check` / `signing-preflight`
scripts. **Updates are signed; installers are not** — that is a decision, not a
gap ([`docs/decisions.md`](docs/decisions.md)), and macOS is deliberately not
built at all until T-705.

---


## M7 — packaging and updates

*Milestone check, as written: a signed installer per OS; one auto-update ships
end-to-end.*

**Amended (Matt, 2026-08-27 — [`docs/decisions.md`](docs/decisions.md)): M7
closes on Linux and Windows, unsigned.** Code signing on Windows and
notarization on macOS are both blocked on paid certificates, not on work, and
neither is obviously worth buying for a server you hand to a friend group. macOS
does not ship at all rather than shipping unsigned — the move from an ad-hoc
signature to a real one is the transition that breaks an app the updater has
replaced in place, so an unsigned build now is not a step towards a signed one.
The signing work is parked as T-705. What still has to happen for real: **one
auto-update, installed, on a machine that did not build it.***

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

  **The key exists (Matt, 2026-08-27).** `scripts/updater-key.sh` generated it
  outside the repo at `~/.local/share/linger/updater.key`; it is password
  protected (scrypt), it is backed up in two places, and its public half —
  minisign key id `8CD4B2592EC1FDF8` — is committed in `tauri.conf.json`. That
  key is now permanent: replacing it orphans every copy installed under it, so
  the script refuses to overwrite one, and a future session must not "regenerate"
  it to fix a problem.

  **The secrets are checked, not assumed.** `release.yml`'s preflight runs
  `scripts/signing-preflight.sh`, which signs a throwaway file with
  `TAURI_SIGNING_PRIVATE_KEY` and then compares the key id in that signature to
  the public key in `tauri.conf.json`. *Present* is not *correct*: a key that
  signs fine but is not the mate of the committed one builds a green release,
  uploads cleanly, and installs on nothing — the failure only shows up when
  somebody tries to update. Running the workflow by hand from the Actions tab
  does the preflight and stops, so the secrets can be proven without cutting a
  release; only a tag builds bundles.

  **Releases are draft-first.** A tag builds Linux, Windows and both macOS
  architectures and opens a *draft* release carrying `latest.json`. Publishing
  it is what makes installed copies see the update, and that stays a human's
  click. The endpoint is this repo's `releases/latest/download/latest.json`,
  which resolves only to published releases.

  > **Corrected 2026-08-27:** the two macOS entries are commented out of
  > `release.yml`, so a tag builds Linux and Windows only. Everything else in
  > this note still holds. See [`docs/decisions.md`](docs/decisions.md) and
  > T-705.

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
  `unconfigured` while the key is absent, then report the new version, the notes
  and a refusal to install an unsigned bundle once pointed at a throwaway
  manifest.

  **Where downloads live is settled (Matt, 2026-08-27): the repo is public**, so
  `releases/latest/download/latest.json` is fetchable with no token and the
  committed endpoint is the final one. This mattered because a private repo's
  release assets need authentication, and the alternative — a credential in the
  client — is not one: shipping a token in a desktop app is shipping the token.
- ✅ **T-702 · Release CSP + the warning, said out loud** — effort: **low** — landed 2026-08-27

  **Landing note.** The policy is two policies now, and the one that ships is
  the strict one.

  **`csp` and `devCsp`, picked at compile time.** Tauri has this built in, so
  there is no second config file and no build-time swap: `devCsp` is used when
  `is_dev()` — which is the `tauri/custom-protocol` feature the CLI adds for a
  bundle and not for `pnpm tauri dev` — and `csp` otherwise. Dev keeps
  `http://localhost:*` and `http://127.0.0.1:*` (and the `ws://` pair) on
  `connect-src`, `img-src` and `media-src`. The shipped policy has none of them.
  It fails safe: forget to pass something and you get the strict one.

  **Said plainly, this means an installed copy only talks `https`.** That is
  less of a change than it sounds — `connect-src` never allowed a bare `http:`,
  so a plain-HTTP server on a LAN was already unreachable from a bundle, and
  localhost was the last exception. The README now says so next to the compose
  instructions, because "run the server on this machine and point the installed
  app at `http://localhost:8080`" is a reasonable thing to try and it will not
  work.

  **The surprise: neither policy allowed Tauri's own IPC.** `invoke()` is a
  `fetch` at `ipc://localhost`, or `http://ipc.localhost` on Windows, and
  neither was in `connect-src`. It never showed up because **the CSP has never
  actually been enforced anywhere yet**: with a `devUrl` the page is served by
  Vite, which sends no policy and into which Tauri injects nothing, so the
  configured CSP has only ever applied to a bundle, and nobody has built one.
  A blocked IPC call does not fail loudly either — it drops onto a slower
  `postMessage` fallback with a console warning. Both sources are allowed in
  both policies now, which is what Tauri's own docs say to do.

  **`style-src 'unsafe-inline'` stays**, and ARCHITECTURE §7 now says why rather
  than claiming otherwise: the message list is virtualized, so a row's position
  is a style attribute, and a name is painted from `--person-*` properties set
  the same way. `script-src` is `'self'` and nothing else.

  **`client/src-tauri/tests/csp.rs`** parses the config with Tauri's own `Config`
  type — so a misspelled `devCsp` fails there instead of silently shipping no dev
  policy — and asserts: no local address in the release policy except
  `http://ipc.localhost`, `https:` still allowed on all three fetching
  directives, IPC allowed in both, the local relaxations still present in dev,
  and the two policies identical apart from local addresses.

  **The SmartScreen half was already done** — it landed in the README with the
  packaging decision earlier the same day, next to the download link, with what
  the warning means and a pointer to `docs/decisions.md`. Left as it was.

  **What is not proven.** "Still reaches its own server" is asserted about a JSON
  file, not watched in an installed build, because building one needs the signing
  key and a bundle nobody has installed yet. That is M7's milestone check, and it
  is a person's job.
- ⬜ **T-703 · Server image publish** — effort: **low**
  ghcr.io workflow for `deploy/Dockerfile` (ffmpeg is already in it, T-501; the
  CI *runner* still needs it, or the video-poster test keeps skipping),
  version tags, compose points at it.
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
- Link-preview fetching is host-side (privacy: the host's IP fetches, not each
  member's). **Built that way in T-504** — the favicon is inlined as a `data:`
  URI so a reader's machine never touches the linked site either. Matt has not
  confirmed the trade-off; the cost is that the server's IP appears in the logs
  of every site anybody links, and turning it off means either no cards or
  every reader fetching for themselves.
