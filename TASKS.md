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
T-90x (entrance sounds). A 2026-08-23 renumber moved old M6–M9 to M5–M8 and
entrance sounds T-403/404/408 to T-901/902/903. The T-911…T-917 band was
activity detection; it was cut on 2026-08-28 and those numbers are retired
rather than reused.

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
| **treacherous** | The strongest model and highest reasoning setting available to you, and coordinate with Matt before claiming. AGENTS.md §"Where you will be wrong" territory: T-705 (signing,
blocked on certificates), T-1402 (real-time audio) and T-1604 (app stores). |

**If you are running Claude specifically**, this is the mapping. The label on
the task is still the source of truth — it is vendor-neutral so that any agent
can run this queue — and this table is just how to read it with today's Claude
models:

| Task label | Claude model | Thinking effort |
|---|---|---|
| **low** | Haiku 4.5 | default |
| **medium** | Sonnet 5 | normal |
| **high** | Opus 5 | high |
| **treacherous** | Opus 5 | maximum |

Two ways to waste money here, and they cost about the same. Running a **low**
task on Opus at maximum effort burns tokens re-deriving decisions the task text
already made. Running a **high** task on Haiku produces something that compiles,
passes the tests it was told to write, and is wrong in the way that only shows
up with four clients on three networks. Match the label.

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
| M7 — packaging and updates | tasks 2026-08-27 | Signed updater behind two Rust commands, tag → draft release with Linux and Windows installers, the shipped CSP stops at the app's own server, the server image publishes to ghcr for x86-64 and ARM64. **Its check is still open** — see *Human checks* | [m7.md](docs/tasks/m7.md) |
| M8 — export | tasks 2026-08-28 | Any member can take a zip of the whole server — a file per room, every upload, an index — built in the background and downloaded from the media origin. **One human check is still open** — see *Human checks* | [m8.md](docs/tasks/m8.md) |
| M9 — knock | tasks 2026-08-29 | `POST /knock` addressed to one person's sessions, a card that fades on its own, the sound player the entrance sounds will extend. **Its check is half-open** — see *Human checks*, HC-6 | [m9.md](docs/tasks/m9.md) |
| M10 — search | 2026-08-31 | An FTS5 index kept by triggers, `GET /search` with no query language to trip over, and a destination in the rail that lands you on a hit six months back — `around=` on the messages endpoint, and a room that knows when it is behind its own newest message | [m10.md](docs/tasks/m10.md) |
| M11 — DMs and group DMs | 2026-08-31 | A DM is a room with members: create-or-find on a canonical member key, a gateway fan-out that filters every frame naming a room, and a membership condition folded into every query that lists messages, files or search results. **Its two-machine half is a human check** — see *Human checks*, HC-7 | [m11.md](docs/tasks/m11.md) |

**Everything a person still has to do by hand lives in one place now:
[Human checks](#human-checks--things-only-you-can-do), at the bottom of this
file.** Seven of them: five left over from closed V1 milestones, one from M9
(hearing a knock on a second computer) and one from M11 (holding a DM across
two, with a third person looking for it). They are not optional and they are
not tasks an agent can take — each one needs somebody sitting in front of a
real computer.

**The one thing that bit us in T-301:** a webview page is a cross-origin caller,
so the server had to start sending CORS headers before the client could read a
single response. The allowed origins are a fixed list in
`crates/linger-server/src/routes/mod.rs`. The gateway WebSocket is *not* subject
to CORS and T-302 confirmed that — but if a future browser-side call mysteriously
"can't reach the server", that list is the first place to look.

One decision styling no longer has to make: **use `oklch()` directly** —
WebKitGTK 2.52.3 supports it (T-002).

What already exists (do not rebuild): workspace + CI; `linger-core` with typed
UUIDv7 ids, the full REST + gateway wire contract, palette/fonts/reactions/limits,
ts-rs export to `client/src/generated/` (committed, drift-checked in CI);
`linger-server` with config/env, WAL SQLite with **single-writer pool discipline**
(`db.write` is a 1-connection pool — keep it that way), migrations (full §5 schema),
error envelope, health route, integration-test harness pattern
(`crates/linger-server/tests/health.rs` — copy its `spawn_server` shape);
Tauri 2 shell with the Console-token M0 frame; deploy files.
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
M9 adds knock (see [m9.md](docs/tasks/m9.md)): `POST /knock` addressed to one
person's sessions, the fading cards, and `lib/sound.ts` — the player entrance
sounds will extend.
M10 adds the whole of search (see [m10.md](docs/tasks/m10.md)): the `message_fts`
index kept by triggers in `0004_search.sql` (**nothing in Rust writes to it**),
`GET /search` over `repo::search` with the query rebuilt rather than forwarded,
and `client/src/search/` — the rail destination, `Ctrl`/`Cmd`+`K`, and landing on
a hit. Two pieces of it belong to the stream rather than to search and are worth
knowing before touching either: **`around=` on `GET /rooms/:id/messages`**
(PROTOCOL §4), which fetches a window centred on one message, and
**`RoomStream.atEnd`**, which is false while a room is showing one of those
windows — live message frames for such a room are *dropped*, because folding
today's message into a February window makes a gap nothing can see. If you are
changing how history loads, read `openAround`, `loadNewer` and `leaveWindow` in
`lib/gateway.ts` first.
M11 adds DMs (see [m11.md](docs/tasks/m11.md)), and it is the one that changed
rules the rest of the server had been relying on. **Every frame naming a room is
filtered by that room's membership**, and `room_of` in `gateway/mod.rs` has no
wildcard arm so a new frame type will not compile until somebody says whether it
names one — do not add a `_ => None` to make it build. **Every query that lists
messages, files or search results folds in
`repo::rooms::visible_rooms(alias)`**, in the `WHERE` rather than as a check
afterwards; `media::Query` and `search::Query` both require a `viewer` for the
same reason. A DM is a room with members — same tables, same endpoints, same
everything downstream of `room_id` — so **do not build a parallel structure for
it**, and do not add a way to change who is in one: a different set of people is
a different DM.

---


## V1 polish — small changes after the milestones closed

Not a milestone and not a backburner: one-off changes to V1 surfaces that came
out of using the app. Each one is small enough that it lands in a single session
with its note written here rather than in an archive.

- ✅ **T-904 · Density belongs in settings, not over every conversation** —
  effort: **low** — Matt, 2026-08-31
  `comfortable` / `compact` / `irc` sat in the room header *and* in settings.
  The header copy is gone; settings is the only home. **Nothing about the
  feature changed** — the same component, the same three modes, still one
  attribute on `<html>` and still remembered in `localStorage`.

  The reasoning is Matt's and worth keeping: a density is chosen once and then
  kept. A control for a decision somebody makes in their first week does not
  earn a permanent place above every conversation — it is chrome paid for on
  every screen, forever, for a choice nobody is making today. The room header is
  now the room's name, who is in it, the topic, and the two things that appear
  only when they apply (`back to the newest`, `since you were gone`).

  Two leftovers went with it: `.density`'s `margin-left: auto`, which existed to
  pin the control to the right of the room name, and the settings rule that
  undid it. `Stream` no longer takes `onDensityChange` at all — it reads
  `density` to decide grouping and nothing else.

  **SPEC did not have to change.** §5.6 lists the three modes; it never said
  where the control lives.

---

## Backburner — later, not the next thing

Three things live here: one V1 feature that is still in the spec, one release
errand blocked on money, and the mobile client. None of them is on the path to a
usable desktop product. Do not pull any of them "while you're in there".

**Activity detection used to be here and is gone** (Matt, 2026-08-28 —
[`docs/decisions.md`](docs/decisions.md)). T-911…T-917 are deleted, along with
the `linger-activity` crate, the bundled app registry, and the `activity` field
on presence. A status is where somebody says what they are doing, because they
typed it. Do not build it back.

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
  **The player already exists — extend it, do not write a second one.**
  `client/src/lib/sound.ts` landed with T-1102 and already owns the global mute
  and the quiet-hours rule, with both switches in settings under `sound`. What
  is missing there is bundled `.opus` playback (the knock is synthesized), the
  5-minute-per-listener cooldown, and per-user mute.
- ⬜ **T-902 · Custom sound upload** — effort: **medium**
  Server: accept ≤2s/≤200KB, transcode to Opus + loudness-normalize (−16 LUFS),
  **reject long files, never truncate**. Needs ffmpeg in the Docker image — add it.
- ⬜ **T-903 · Curate the bundled sounds** — effort: **low** *(Matt-assisted, taste required)*
  12–16 sounds per `assets/sounds/README.md` rules; `ffmpeg -af loudnorm=I=-16`
  for normalization; fill the source/license table.



### Mobile

*Moved here 2026-08-28 by Matt, out of V2.* **Desktop first.** The app has to be
finished, installed by real people, and lived with for a while before a second
platform is worth starting. A mobile client doubles the surface of every bug
still in the desktop one, and V1 has not been through a single human check yet.
This is not a "next quarter" item, it is a "when the desktop app is boring"
item.

*Check, when it comes back: sign in, read a room, send a message and a photo,
from a phone.*

**Start with the decision, not the code.** Mobile has one question in it that is
Matt's and is not a technical one:

> **Push notifications go through Apple and Google.** There is no other way to
> wake a phone app. That means a message's *existence* — and whatever the
> notification says — passes through a third party, which is a different promise
> from the one the README makes today.

Three honest answers, and one has to be picked before T-1602: ship without push
and let the app only notify while open; ship with push and **change the README
to say exactly what leaves the server**; or run a self-hosted push relay, which
is a second piece of infrastructure for every host and probably kills it.
Recorded in the *Parking lot* too.

- ⬜ **T-1601 · Decide what mobile means** — *not a task; a decision.* Matt.
  The push question above, plus: does mobile get uploads, does it get voice, and
  is it iOS-and-Android or one of them. Write the answers into SPEC before
  anything else starts.

- ⬜ **T-1602 · The mobile shell** — effort: **high**
  Tauri 2 builds for iOS and Android from the same crate. What does not carry
  over: the OS keyring (mobile has its own secure storage) and the tray. Expect
  the gateway to
  need reconnect behaviour for a network that changes every time somebody walks
  out of a building.
  *Accept:* the app opens on a real phone, signs in, and stays connected across
  a wifi-to-mobile-data switch.

- ⬜ **T-1603 · The layout at phone width** — effort: **medium**
  The roster already collapses under 880px (`client/src/lib/layout.ts`), which
  is a head start. What is missing is one-hand reach, a composer above a
  software keyboard, and the rail as something other than a fixed column.
  *Accept:* usable one-handed on a phone somebody actually owns.

- ⬜ **T-1604 · Getting it onto a phone that is not yours** — effort: **treacherous**
  Apple Developer Program, Google Play, review, and store listings. This is the
  same money-and-paperwork wall as T-705, doubled. **Do not start without both
  accounts.** Follow current vendor docs, not memory — this changes every year.
  *Accept:* somebody who has never met you installs it from a store.

---

## V2 — M9, M10 and M11 built; voice is started

**Three of the five are built and archived** — knock on 2026-08-29
([`m9.md`](docs/tasks/m9.md)), search and DMs on 2026-08-31
([`m10.md`](docs/tasks/m10.md), [`m11.md`](docs/tasks/m11.md)). Everything
below is still planned and not started. **Nothing below M11 is next.** V1 is
done except the things in *Human checks*, and those come first — a release
nobody has installed is not a finished V1, and three of the seven checks now
belong to V2 features that have only ever run on one computer.

**Read `m11.md` before touching the gateway fan-out, or anything that lists
messages, files or search results.** DMs put a membership check in every one of
those, and three of the mechanisms are load-bearing in ways the code does not
say out loud.

**Read this before touching any of it:** SPEC §6 lists what V2 is; anything not
on that list is not V2, it is scope creep. Voice is the one left that adds a
whole category of thing this product does not have, and **T-1402 says to
coordinate with Matt before starting** — that is where audio, real networks and
`webrtc-rs` all arrive at once.

**Numbering.** V1 used `T-1xx`–`T-8xx` for milestones M1–M8, plus `T-9xx` for
the V1 work that came off the critical path — and for small V1 changes that land
after the milestones closed (`T-904`). V2 starts a new band at **`T-1xxx`, where
the hundreds digit is the area** — `T-11xx` knock, `T-12xx` search, `T-13xx`
DMs, `T-14xx` voice, `T-15xx` ambient voice — and `T-16xx` mobile, which is on
the backburner rather than in V2, and `T-17xx` themes, which is V3. It does not
continue the milestone-matches-number rule, because `T-9xx` is already spoken
for.

**The order is not SPEC §6's order, on purpose.** SPEC lists voice first because
it is the headline. It is sequenced last here because it is the largest and
riskiest thing in the project, and because knock and search are small,
self-contained, and make the app better next week rather than next quarter.
Build order, cheapest and safest first:

```
M9 knock (built) → M10 search (built) → M11 DMs (built)
  → M12 voice (signalling + transport built; no microphone) → M13 ambient voice
```

**Mobile is not in this sequence** (Matt, 2026-08-28). It was going to be M14;
it is on the *Backburner* instead. Desktop has to be finished and used by real
people first.

---


---

### M12 — voice rooms

*Milestone check: four people talk to each other, on four different networks,
for an hour, without anybody dropping.*

**The riskiest thing in the project.** AGENTS.md §"Where you will be wrong"
names two of these areas explicitly: WebRTC generated from memory works on
localhost and dies behind real NAT, and audio device handling breaks on hotplug
and sample-rate mismatch. Both warnings are load-bearing here.

**The architecture is already decided** (ARCHITECTURE §2): audio lives in
**Rust, not the WebView** — `webrtc-rs` for transport, `cpal` for devices. This
is not a preference. WebKitGTK's WebRTC is the weakest of the three engines, and
keeping audio out of the page removes it from the critical path entirely.

**Do not test this on one machine.** Two processes on one laptop connect over
loopback and prove nothing at all.

**The signalling landed 2026-09-01 (T-1401)** and it is the one piece of M12
that a single machine *can* prove, because there is no audio in it: the frames
are exercised by two real WebSocket clients in-process. **T-1402's transport
half landed the same day** — real peer connections, real ICE, no microphone.
Everything past that needs real networks. **SPEC §4.14 is written** — read it
before the rest of this milestone, because the decision it records ("voice
happens in a room, not in a call") is what the other four tasks assume.

**⚠ Two system packages are missing and block the rest of T-1402**:
`libasound2-dev` (for `cpal`) and `libopus-dev` or `cmake` (for the Opus
encoder). Installing them needs a password. Until then there is no audio, so
nothing in M12 can be listened to — see T-1402.

- ✅ **T-1401 · Signalling over the gateway** — effort: **high** — Matt, 2026-09-01
  Three client frames (`voice.join`, `voice.leave`, `voice.signal`) and two
  server ones (`voice.state`, `voice.signal`), all in `gateway/`. **No audio
  anywhere** — the payloads are opaque strings and nothing in this server parses
  one. **SPEC §4.14 and PROTOCOL §8's voice section were written in the same
  commit**; voice had a one-line scope entry and nothing else.

  **Voice happens *in a room*, not in a call.** You are already in the room, and
  joining voice is turning your microphone on where you are. That is the
  decision the rest follows from: no call object, nothing to be invited to, no
  ringing — and M13's "a room you leave running" becomes a small step rather
  than a rewrite, because a room you leave running is what this already is.

  ### A peer is a session, not a person

  A peer connection is between two *clients*, and somebody signed in on a laptop
  and a desktop is two of them. So the seat is keyed by session id, and
  `voice.signal` is addressed to a session — which needed `publish_to_session`,
  narrower than the `publish_to` a knock uses. `visible_to` grew a rule 0 for
  it.

  **Who offers is decided by the ids, not by who arrived first.** Of any pair,
  the lower `session_id` sends the offer. Both ends read the same `voice.state`
  and reach the same answer, so no pair ever sends two offers at each other.
  "Whoever joined later offers" needs an order both sides agree on, and a
  reconnect is exactly when they stop agreeing. `voice.state` is sorted by
  session id so a client can read the answer off the list rather than derive it.

  ### The half-connected state, which is what the criterion is about

  Two ways to get there, opposite directions, and each has a test.

  **A dropped socket must keep its seat.** A blip is two seconds of bad wifi,
  and tearing down every peer connection over it is worse than waiting. So the
  seat is released when the *session* ends, not when the socket does — which is
  also why a resumed session finds its peers still there and replays the
  signals sent while it was away.

  **A closed socket must lose its seat**, and this is the part that was wrong
  first. Saying goodbye and being cut off are different things, and the code did
  not distinguish them: closing the app held a seat for the full 120-second
  resume window, so everybody else was talking to somebody who had left. The
  reader loop now notices a WebSocket close frame and releases the seat there.
  The *session* still survives either way — a client that says goodbye and then
  resumes is resumed, it simply has to join voice again, which is what leaving
  means.

  ### Three rules that keep the frame from being something else

  - **Both ends must be in voice, in the same room.** Without that check
    `voice.signal` is a way to hand an arbitrary string to any session on the
    server — a side channel nothing else here has.
  - **A signal to a session that is gone is dropped, not refused.** Somebody's
    client closing mid-exchange is the ordinary end of a call; an error frame
    would be noise about a thing that is not wrong.
  - **`voice.state` names a room, so it is filtered like every other frame that
    does.** Voice in a DM is as private as the DM (SPEC §4.13) — and `room_of`
    having no wildcard arm is what forced that decision rather than leaving it
    to be remembered. It did not compile until both new frames were classified.

  ### Tests

  `crates/linger-server/tests/voice.rs` — eleven, over real WebSockets, with
  the acceptance criterion as one of them: a full offer→answer→candidate
  exchange that survives a forced disconnect, including a signal sent into the
  gap and replayed on resume.

  **Checked by breaking the code.** Removing the same-room check fails the
  cross-room test; releasing the seat on every socket close fails the resume
  test; removing the DM check on `voice.join` fails the outsider test.

  **One test was passing for the wrong reason and one mutation was aimed at the
  wrong line.** The DM-outsider test wrapped its assertion in `if let Some(state)
  = …`, so it passed by never running; it now asserts whether or not a frame
  arrives, because "no frame" is the right answer and "a frame naming Dave" is
  the wrong one. And the mutation meant for `voice.join`'s visibility check hit
  `typing.start`'s identical block first — the test only proved itself once the
  mutation was aimed properly.

  ### What this does not do

  No audio, by design — that is T-1402, and it is the treacherous one.
  **Nothing was tested across two machines**, because there is nothing to hear
  yet; the frames are what this task delivers and they are exercised by two real
  WebSocket clients in-process. The moment audio exists, AGENTS §"Where you will
  be wrong" applies in full: WebRTC generated from memory works on localhost and
  dies behind real NAT.

- ⏳ **T-1402 · The audio path** — effort: **treacherous** — Matt, 2026-09-01
  **The transport half landed; the microphone half is blocked on two packages
  nobody has installed.** Read this whole entry before picking it up.

  ### What is built and runs

  `client/src-tauri/src/voice/` — real `RTCPeerConnection`s, a full mesh, real
  DTLS, real ICE, real RTP, driven by T-1401's frames. Three Tauri commands
  (`voice_join`, `voice_leave`, `voice_frame`) and a `voice:peer` event; the
  frontend forwards the two voice frames to the core and does nothing else with
  them, because audio lives in Rust (ARCHITECTURE §2).

  **`webrtc` needs nothing installed.** That was the open question and the
  answer is good: it is pure Rust down to the crypto, builds in about thirty
  seconds, and adds no system package to anybody's build.

  `mesh.rs` is the part worth reading. It is pure — a peer list in, a plan out —
  because that is where the bugs live that only show up on a bad network: a peer
  dropped and never rebuilt, two clients that both offer, a reconnect that
  leaves one side waiting for an answer nobody will send.

  **The lower session id offers.** Both ends read the same `voice.state` and run
  the same comparison, so no pair sends two offers at each other — that is
  *glare*, and it leaves both sides waiting on an offer the other discarded.
  "Whoever joined later offers" needs an order both sides agree on, and a
  reconnect is exactly when they stop agreeing.

  **Candidates that arrive before the answer are held, not dropped.** ICE
  trickles, so the far end starts sending them before its answer has been
  applied here. Adding one to a connection with no remote description is an
  error and dropping it is a call that takes the long way round or never
  connects — and on a good network you never notice, which is what makes it
  exactly the kind of bug AGENTS warns about.

  ### ⚠ What is blocked, and on what

  **Two system packages, and installing them needs a password:**

  ```
  sudo apt install libasound2-dev   # cpal: ALSA headers, for the microphone
  sudo apt install libopus-dev      # or cmake, to build libopus from source
  ```

  `cpal` fails at `alsa-sys`'s build script without the first. The `opus` crate
  vendors libopus and builds it with cmake, which is also not installed. Neither
  is optional: WebRTC audio *is* Opus, and there is no mature pure-Rust encoder.

  **CI will need them too** — `.github/workflows/ci.yml`'s `tauri-shell` job
  installs the webview deps and will need these beside them. They are
  deliberately not added yet, because a package CI installs for code that does
  not exist is a line nobody can explain later. Add them in the same commit as
  the code that needs them, and update the README's system-dependency list in
  the same breath (AGENTS: a new system dependency is a README change).

  ### What is left, in order

  1. `cpal` capture → `audio::Source`, and `audio::Sink` → `cpal` playback.
     The seam is already there and it is one 20 ms frame at 48 kHz mono, which
     is what Opus and WebRTC both want — so this is filling a hole, not
     designing one.
  2. Opus encode on the way out, decode on the way in.
  3. The loop that pulls frames from the source into each peer's track, and the
     `on_track` handler that pushes received audio at the sink. `Engine::outbound`
     hands out the track; nothing drives it yet.
  4. Mixing several peers on playback. `Sink::play` takes the peer it came from
     so that decision is still open.

  ### What none of this proves

  **Nothing here has been across two machines.** Both ends of every test are on
  loopback with no NAT between them, which AGENTS §"Where you will be wrong"
  names as the arrangement that works right up until somebody is behind
  carrier-grade NAT — and TASKS says it in fewer words: *do not test this on one
  machine*. `Engine::new` takes an ICE server list and it is **empty**, so today
  there is not even STUN: host candidates reach another machine on the same
  network and nothing beyond it. That is T-1403.

  *Accept:* four people, four networks, one hour, no drops. Anything less than
  that is not evidence. **Unchanged, and not met.**

- ⬜ **T-1403 · A TURN server in the deploy** — effort: **high**
  coturn in `deploy/`, credentials that are not shared secrets in a compose
  file, and the host guide updated. Without this, anybody behind a phone network
  or a corporate router cannot connect at all — and it will look like a bug in
  the app rather than missing infrastructure.
  *Accept:* two clients connect where at least one is behind carrier-grade NAT.

- ⬜ **T-1404 · The voice surface** — effort: **medium**
  Join and leave, who is speaking, per-person volume, push-to-talk, a device
  picker, mute. Console design system: no bubbles, no glow, no animated rings.
  *Accept:* usable by somebody who has not read anything.

- ⬜ **T-1405 · Devices that change under you** — effort: **high**
  Its own task because AGENTS says so: headphones unplugged mid-call, the OS
  default device changing, a device that wants a different sample rate.
  *Accept:* unplug and replug headphones during a call; audio continues on the
  new device without a restart.

---

### M13 — ambient voice

*Milestone check: leave a room running for a working day; it costs almost no
CPU and nobody had to "join" anything.*

The differentiator, and it only makes sense on top of M12. "A room you leave
running, not a call you join" — no ringing, no joining ceremony, no call that
somebody has to end.

**The privacy shape matters more than the code.** An always-on microphone is
exactly the thing this product's whole pitch is against, so it needs the same
treatment activity sharing got in SPEC §4.3: **off by default, a persistent
visible indicator whenever it is on, and one obvious way to kill it.**

- ⬜ **T-1501 · Ambient mode** — effort: **high**
  Open mic with voice detection so silence costs nothing, no join step, idle
  cost low enough to leave on all day.
  *Accept:* eight hours in a room, CPU and battery measured, and a number
  written down in the landing note.

- ⬜ **T-1502 · The controls that make it safe** — effort: **medium**
  Default off. A persistent indicator, visible without opening anything, any
  time the mic is live. One-click kill from the roster. Quiet hours.
  *Accept:* somebody who did not set it up can tell at a glance whether their
  microphone is on.

---

## V3 — further out than V2, and not all of it will happen

SPEC §6's *V3 or never* list, plus the one Matt added on 2026-08-31. **Nothing
here is next**, and nothing here starts before V2 is done and V1 has been lived
with. This section exists so the ideas are written down with their problems
attached, which is the only useful form to keep an idea in.

**Numbering** continues the V2 rule — the hundreds digit is the area. `T-17xx`
is themes. The rest of the V3 list (an opt-in directory, sandboxed client
scripting, custom emoji) has no numbers yet because nobody has thought about it
hard enough to write a task.

### M14 — custom themes

*Matt, 2026-08-31: "I would like for us to support themes where people can
create custom color schemes and themes for the application, which will allow for
some great flexibility and maybe even a themes-community."*

**Why it fits.** Personalization is not a bolt-on here, it is the thesis — SPEC
§2 names AIM-era self-expression as half of what this product is for, and the
styled name is already the one expressive element. A person choosing how their
whole app looks is the same idea one size up. Nothing on the anti-goals list
forbids it.

**Why it is not simply "let people write CSS", and why this is three tasks and
not one.** Four things in this repo are load-bearing and a theme walks straight
into all of them:

1. **The 16-colour palette is a contract, not a preference.** It is defined once
   in `linger-core::PALETTE`, validated *server-side* (AGENTS rule 8), and a
   property test in CI asserts every one of the 16 keys clears 4.5:1 contrast
   against both theme backgrounds. It is what makes "there is no way to pick a
   colour nobody can read" true. A theme that repaints those keys deletes that
   guarantee unless it is checked the same way — so the check has to move to
   wherever the theme is applied, and a theme that fails it has to be refused
   or corrected, in front of the person who made it.
2. **Colours are palette keys everywhere** — on the wire and in the database
   (AGENTS rule 12) — so a theme cannot be stored as somebody's name colour. It
   is a different kind of thing: the reader's own view of *their* app, closer to
   density and dark/light than to a styled name. That points at it being a local
   preference, not a server object, which is a much smaller feature.
3. **The Console rules are the product, not a default skin** (SPEC §5.1): no
   chat bubbles, no shadows, no gradients on surfaces, no rounded panels,
   monospace for metadata only. If a theme can turn those off, the design system
   is advisory and the answer to "why does this look like Discord" becomes "you
   installed a theme". A theme almost certainly gets colours and maybe warmth,
   and does *not* get geometry, spacing or typography.
4. **"A themes-community" is a distribution problem wearing a cosmetics hat.**
   Rule 14 forbids a payment surface of any kind, and SPEC §6 already puts an
   opt-in directory at V3 with "must never be load-bearing" attached. A theme
   that is a file somebody sends a friend is nothing to build. A gallery inside
   the app is a store without prices, and it is also a moderation surface. If
   themes are CSS rather than a list of colours, sharing them is
   **arbitrary-code-shaped**, and SPEC §6 already flags sandboxed client
   scripting as "a real security surface".

**Three decisions are Matt's and none of the tasks below can start without
them.** They are repeated in the *Parking lot*.

- ⬜ **T-1701 · What a theme is allowed to change** — *not a task; a decision.*
  Matt. Colours only, or colours plus warmth, or something wider? Does a theme
  repaint the 16 name colours, or only the surfaces and text around them? The
  contrast guarantee follows the answer, and so does whether this is a weekend
  or a month.
- ⬜ **T-1702 · The theme format and the editor** — effort: **high**
  *Blocked on T-1701.* Whatever a theme turns out to be, it is a **list of
  values, not a stylesheet** — the tokens in `styles/tokens.css` are already
  exactly this shape, which is why density and evening warmth are one variable
  swap. Applying one is then the same move: set custom properties on `<html>`.
  A theme lives on the reader's machine beside density and theme preference, not
  on the server, unless T-1701 says otherwise.
  **The contrast check comes with it, in the app, live** — the CI property test
  in `linger-core` is the model, and the editor should refuse to let somebody
  build something they cannot read rather than warning them afterwards.
  *Accept:* somebody makes a theme, restarts the app, and it is still there —
  and cannot save one that fails contrast.
- ⬜ **T-1703 · Sharing a theme** — effort: **medium**
  *Blocked on T-1701, and on Matt saying how far this goes.* The cheap version
  is the whole feature: a theme exports as one small file, and importing one
  shows what it will look like before it applies. That is a themes-community
  with no infrastructure, no gallery, no moderation and no store — people send
  each other files, the way they always have.
  **Anything beyond that is a scope decision, not an implementation detail.**
  *Accept:* a theme made on one computer is applied on another, with nothing in
  between but a file.

---

## Human checks — things only you can do

Seven things are built, tested, and **never once used by a person** — or, in
HC-6 and HC-7's case, never used across two of them. Automated tests prove the
code does what it says. They cannot prove that a window opens, that a 400 MB
upload survives a real network, or that a sound is one you would want to hear.
That is this list.

None of these are agent tasks. Each one needs you, a real computer, and a few
minutes. The first five are ordered so that **doing the first one knocks out
most of the second and third at the same time.**

---

### HC-1 · Cut a release and watch a machine update itself

*Closes M7's milestone check, and most of HC-2. The biggest one, and everything
it needs is already built.*

This is the one thing no test can do: prove that a copy of Linger installed on
somebody else's computer can replace itself with a newer one.

**Before the first tag, once ever — two things:**

1. **Prove the container image builds on both chip types.** Go to the repo's
   *Actions* tab → *image* → *Run workflow*. It builds for regular PCs and for
   ARM (Raspberry Pi and similar) and pushes nothing. The ARM half has never
   been built, so if it is going to fail, this is where you want to find out —
   not on release day. It takes a while; ARM is built by emulation and is slow.
2. **Check the four version numbers agree.** Run `scripts/version-check.sh`. If
   it complains, bump all four files it names and commit that first.

**Then, the release itself:**

3. Tag it and push the tag:
   ```bash
   git tag v0.1.0 && git push origin v0.1.0
   ```
4. Wait for *Actions* to finish. It builds a Windows installer and a Linux one,
   publishes the server image, and opens a **draft** release. Nothing is public
   yet — a draft is invisible to everybody.
5. **Make the container image public.** First time only. Go to
   `github.com/users/<you>/packages/container/linger/settings` and set the
   visibility to public. Skip this and every host's `docker compose up` fails
   with `unauthorized`, including yours.
6. Read the draft release, then press **Publish**. That is what makes it real.
7. **Install it on a machine that did not build it.** A second laptop, a
   virtual machine, a friend's PC — anything but your dev box. On Windows you
   will see *"Windows protected your PC"*; that is expected, click *More info*
   → *Run anyway* (see [`docs/decisions.md`](docs/decisions.md)).
8. Bump the version in all four files, commit, tag `v0.1.1`, push, wait, and
   publish that release too.
9. On the machine from step 7, open Linger → **settings → updates**. It should
   say a new version is waiting. Press install. It should download, replace
   itself, and come back as 0.1.1.

**Done when:** step 9 works. If it does not, the thing to look at first is
whether the signing key in the repository secrets is the mate of the public key
in `tauri.conf.json` — run the *release* workflow manually from the Actions tab
and it checks exactly that, without releasing anything.

---

### HC-2 · Watch a window open on all three operating systems

*Left over from T-002/T-003 (`docs/tasks/m0.md`). Was meant to close before M7.*

Steps 7 and 9 of HC-1 cover **Linux and Windows** — if the app opened, this is
two thirds done. What is left is **macOS**, and it cannot be done from a
release, because Linger deliberately does not build a Mac version yet
([`docs/decisions.md`](docs/decisions.md) says why).

**For the macOS third**, on a Mac, from a checkout:

```bash
cd client && pnpm install && pnpm tauri build
```

Then open the app it produces in `client/src-tauri/target/release/bundle/`.
macOS will complain that it cannot verify the developer — that is expected for
an unsigned build.

**Done when:** a window opens on all three, and you have seen it.

---

### HC-3 · Share a 400 MB video for real

*Closes M5's milestone check (`docs/tasks/m5.md`). Every piece is tested;
nobody has ever clicked `+ file` in a running app.*

Needs a real server on a real domain (see
[`docs/host-guide.md`](docs/host-guide.md)), not a local one — the point is the
network and the two domain names, which is where uploads actually break.

1. Sign in to your server from the desktop app.
2. Drag a **400 MB or larger video** into the message box. Watch the progress
   bar.
3. **Kill the network halfway** — turn wifi off and back on. The upload should
   pick up where it left off rather than starting again.
4. When it lands, check the message shows a **poster frame** (a still from the
   video), not a blank box.
5. Open **media** in the left rail. The video should be there. Star it.
6. Look at the **storage figure** in the status bar. It should have gone up by
   roughly 400 MB.
7. On a **second computer**, sign in as somebody else and check the video is
   there and plays.
8. Set a **status image** on one machine and check it appears on the other at a
   sensible size.

**Done when:** all eight work. If the upload fails but chat works, the `cdn.`
name is the first thing to check — see the host guide's troubleshooting.

---

### HC-4 · Watch a styled name go past in a room

*Left over from M6 (`docs/tasks/m6.md`).*

The style picker, the themes and the evening warmth were all driven live in a
browser — but a browser session has no rooms and no messages, because the live
connection only exists inside the desktop app. So the names in the message
stream have only ever been checked as computed values, never seen.

1. Two computers (or one computer and a virtual machine), signed in as two
   different people.
2. On the first: **settings → how your name looks**. Set a gradient of two
   colors, a different face, and turn on shimmer.
3. Send a few messages.
4. On the second machine, look at the stream. The name should be drawn the way
   it was set, in colour, in that face.
5. Turn on **normalize everyone** on the second machine. Every name should go
   plain immediately, including in the stream.
6. Switch **density** to compact, then to IRC. Effects should switch off.
7. Wait until after 7pm local time (or change the clock) and check the
   background goes slightly warmer, and that names are still readable.

**Done when:** you have watched a styled name scroll past in a real room.

---

### HC-5 · Press the export button

*Closes T-802 (`docs/tasks/m8.md`). The smallest one on this list.*

1. In the app: **settings → take everything with you**.
2. Press **export everything**. Watch the line underneath — it should count up.
3. When it says the archive is ready, press **download it**. Your normal
   browser should take the download.
4. Unzip the file. Open `rooms/<something>.md` in any text editor and read it.
   Open something in `media/`.
5. Press **export everything** again straight away. It should tell you, in
   words, roughly how long until you can ask again — not show an error.

**Done when:** you have read a room out of a zip that Linger did not open for
you. That is the whole promise of the feature.

---

### HC-6 · Hear a knock, on a second computer

*Closes M9's milestone check (T-1102). The newest one on this list, and the
only one not left over from V1.*

The knock was built and tested with one real client on one machine and a second
member knocking over the endpoint. That proves the card and the endpoint. It
does not prove the two things a second computer proves: that a knock crosses a
network, and that the sound is a sound you would want to hear.

1. Two computers, both signed into the same server as different people.
2. On one: open the other person's card in the roster and press **knock**.
3. On the other: a card should appear bottom-right, say who knocked, and go
   away by itself after about eight seconds. **Nothing should be left** — not in
   the stream, not on the roster, not anywhere.
4. **Do this outside 22:00–08:00**, or you will hear nothing and it will look
   broken. Quiet hours are on by default, which is the point.
5. Listen to it. It is two soft taps built out of an oscillator, not a recorded
   sound, and nobody has heard it on speakers yet. If it is annoying, say so —
   it is about twenty lines in `client/src/lib/sound.ts` and easy to change.
6. Press knock four times inside an hour. The fourth should say *"That's three
   this hour. Give them a bit."* rather than failing.

**Done when:** a knock has crossed two machines and you have heard it.

---

### HC-7 · Hold a DM on two computers, with a third watching

*Closes M11's milestone check (T-1301…T-1303). The same shape as HC-6 and for
the same reason: everything was verified with one real client and two scripted
sockets on one machine.*

The frames a scripted socket receives are the frames a client receives, so what
this is really checking is the part a script cannot: that three real windows
behave, and that a DM feels like a private conversation rather than a room with
a filter on it.

1. **Three computers**, or two plus a phone browser you can sign in on — all on
   the same server, as three different people. Call them A, B and C.
2. On **A**: open B's card in the roster and press **message**. A `direct`
   section appears in the rail with B's name in it.
3. Say something. It should turn up on **B** immediately, in a conversation
   that appeared in their rail without them doing anything.
4. **On C, look for it.** There should be nothing: no `direct` section entry,
   nothing in `media`, nothing in `search` for a word only used in the DM. C's
   roster should show A as *around* — **not** "in a message with" anybody.
5. Share a file in the DM. Check `media` on B (it is there) and on C (it is
   not).
6. On **C**, press **export everything** in settings and open the zip. There
   should be no `direct/` folder in it at all.
7. On **B**, do the same. There should be `direct/<A's username>.md`, readable,
   with the conversation in it.

**Done when:** a DM has crossed two machines and a third person has looked for
it in four places and not found it.

---

## Parking lot (decisions needed, not tasks yet)

- **What is a custom theme allowed to change, and how far does sharing go?**
  Raised by Matt on 2026-08-31 and written up as M14 (T-1701…T-1703). Three
  answers are needed before any of it starts. *What a theme touches:* colours
  only, or colours plus the evening warmth, or more — and in particular whether
  it repaints the 16 name colours, because those are validated server-side and
  their 4.5:1 contrast is guaranteed by a test in CI. *Whether the Console rules
  are themeable at all:* if a theme can turn off the no-bubbles, no-shadows,
  no-rounded-panels rules, the design system is advice rather than the product
  (SPEC §5.1). *How far sharing goes:* a theme as a file people send each other
  costs nothing and is probably the whole feature; a gallery inside the app is a
  store without prices and a moderation surface, and rule 14 plus SPEC §6's
  "must never be load-bearing" both point at it.
- Bundle identifier is `com.linger.desktop` — fine? Changing after M7 is painful.
- **Mobile push goes through Apple and Google, or it does not exist.** There is
  no third way to wake a phone app. Whatever a notification says, and the fact
  that it happened, passes through a company that is not you. The README's
  privacy section does not currently allow for that. Three answers: no push
  (the app only notifies while it is open), push with the README changed to say
  exactly what leaves the server, or a self-hosted relay — which is a second
  piece of infrastructure for every host and probably ends the idea. **This
  blocks mobile** and nothing else. Raised 2026-08-28 while planning V2.
- ~~**Where does search live, and what does it cover?**~~ **Answered by Matt,
  2026-08-30**, and written into SPEC §4.12: a destination in the rail next to
  `media`, opening in place of the stream, with `Ctrl`/`Cmd`+`K` as a shortcut
  into it rather than a second surface; and it covers what people typed plus the
  names of files, not link titles. Raised 2026-08-28.
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
