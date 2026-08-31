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
| M9 — knock | tasks 2026-08-29 | `POST /knock` addressed to one person's sessions, a card that fades on its own, the sound player the entrance sounds will extend. **Its check is half-open** — see *Human checks*, HC-6 | still in this file, below |

**Everything a person still has to do by hand lives in one place now:
[Human checks](#human-checks--things-only-you-can-do), at the bottom of this
file.** Six of them: five left over from closed V1 milestones, and one from M9
(hearing a knock on a second computer). They are not optional and they are not
tasks an agent can take — each one needs somebody sitting in front of a real
computer.

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

## V2 — M9 built, the rest planned

**M9 (knock) landed 2026-08-29**, and **M10's server half landed 2026-08-30** —
the index and the endpoint; what is left of search is the surface, T-1203.
Everything below M10 is still planned and not started. **Nothing below M10 is
next.** V1 is done except the things in *Human checks*, and those come first — a
release nobody has installed is not a finished V1. This section exists so the
shape of V2 is written down while it is fresh, not so somebody starts it.

**Read this before touching any of it:** SPEC §6 lists what V2 is; anything not
on that list is not V2, it is scope creep. Voice and DMs each add a whole
category of thing this product does not currently have, and each one has at
least one decision in it that is **Matt's, not an implementing session's**.
Those are called out per milestone and repeated in the *Parking lot*.

**Numbering.** V1 used `T-1xx`–`T-8xx` for milestones M1–M8, plus `T-9xx` for
the V1 work that came off the critical path. V2 starts a new band at **`T-1xxx`,
where the hundreds digit is the area** — `T-11xx` knock, `T-12xx` search,
`T-13xx` DMs, `T-14xx` voice, `T-15xx` ambient voice — and `T-16xx` mobile,
which is on the backburner rather than in V2. It does not continue the
milestone-matches-number rule, because `T-9xx` is already spoken for.

**The order is not SPEC §6's order, on purpose.** SPEC lists voice first because
it is the headline. It is sequenced last here because it is the largest and
riskiest thing in the project, and because knock and search are small,
self-contained, and make the app better next week rather than next quarter.
Build order, cheapest and safest first:

```
M9 knock (built) → M10 search → M11 DMs → M12 voice rooms → M13 ambient voice
```

**Mobile is not in this sequence** (Matt, 2026-08-28). It was going to be M14;
it is on the *Backburner* instead. Desktop has to be finished and used by real
people first.

---

### M9 — knock ✅ *(both tasks landed 2026-08-29)*

*Milestone check: a knock crosses two machines, shows up as something that
fades on its own, and leaves nothing behind.* **Half-open:** everything but the
two machines is done and tested. The two-machine half is **HC-6**.

SPEC §4.9 was the whole spec and the hard part was what it *refuses*: no
message, no thread, no unread state, nothing to dismiss. A knock that leaves a
notification sitting there has become a message, and the feature is gone. What
shipped keeps that: nothing is stored at either end, and the card has no
controls on it at all.

- ✅ **T-1101 · Knock, server side** — effort: **low** — Matt, 2026-08-29
  `POST /knock` is mounted, and `crates/linger-server/tests/knock.rs` covers all
  three acceptance cases plus three more: a stranger is `NOT_FOUND`, knocking
  yourself is `VALIDATION_FAILED`, and the schema still has no table with
  "knock" in its name.

  **What this changed structurally.** The gateway bus used to carry a bare
  `ServerEvent` and fan it to everybody; a knock is the first frame with an
  audience of one. The bus now carries a small `Fanout { event, to }` and
  `Gateway::publish_to` sets `to`. `publish` is unchanged and every existing
  caller is untouched. Two things worth knowing next time somebody adds an
  addressed frame:

  - **The address is not on the wire.** `knock` carries `from_user_id` only.
    The receiver is the only person who gets the frame, so a `target_user_id`
    field would carry nothing and would be one more thing to get wrong.
  - **It is sequenced, like everything else.** That means a resume inside the
    120s window replays it. That is deliberate: an unsequenced frame is
    *dropped* by the Tauri core (`gateway.rs` ignores frames without `s`), so a
    control-frame knock would never reach the frontend at all.

  A knock at somebody who is not connected lands nowhere and returns 204. That
  is the design, not a gap — nothing is queued, because nothing is stored.

- ✅ **T-1102 · Knock, client side** — effort: **medium** — Matt, 2026-08-29
  You knock from the person's card in the roster, one control, one click, no
  confirmation. What arrives is `client/src/knock/KnockCards.tsx`: a card in the
  bottom-right that appears, is never focusable, and takes itself off after
  `KNOCK_TTL_MS` (8s). It has no buttons, `pointer-events: none`, and
  `aria-live="polite"` so a screen reader hears it without anything stealing
  focus.

  **The sound is `client/src/lib/sound.ts`, and it is the one T-901 extends —
  do not write a second.** It owns the gate (global mute, plus quiet hours
  22:00–08:00 listener-local, default **on**) and plays a knock synthesized from
  an oscillator, so this task did not have to reach into T-903's curation for an
  asset. What T-901 adds to *this* file: bundled `.opus` playback, the
  5-min-per-listener cooldown, and per-user mute. Both switches are in settings
  under `sound`.

  **A knock lives in the gateway store**, as `GatewayState.knocks`, next to
  `typing` and for the same reason: it is transient state with no server copy.
  Not a second store (AGENTS: local state plus one gateway store). Cards from
  every signed-in server are drawn, not just the active one.

  **Verified on this machine, not on two.** One real client signed in, a second
  member knocking over the real endpoint: the card appeared, faded on its own,
  and left nothing in the stream or the roster; knocking from the card reached
  the other person's socket; the fourth knock inside an hour showed "That's
  three this hour. Give them a bit." **Still unverified:** the sound by ear (the
  test ran at 07:50, inside quiet hours, so it correctly stayed silent), and the
  acceptance criterion's *two machines*. Both belong in a human check.

---

### M10 — search

*Milestone check: type a word, get the messages that contain it, click one, land
on it in the room it was said in.* **Open, and waiting on one task:** the index
and the endpoint landed 2026-08-30, so the words are findable over HTTP and
nothing draws them. T-1203 is the whole of what is left.

**The spec section is written** — SPEC §4.12, and the search entry in PROTOCOL
§6, both landed with T-1201. The two questions in it were Matt's and were
answered on 2026-08-30:

- **Search is a destination in the left rail, under the rooms, next to
  `media`**, opening in place of the message stream. `Ctrl`/`Cmd`+`K` is a
  shortcut *into* that destination, never a second surface with its own
  behavior.
- **It covers what people typed and the names of the files they shared.** Link
  titles are deliberately out: a title is a cache the server refreshes on its
  own schedule, so indexing one means results changing under people with nobody
  having edited anything.

- ✅ **T-1201 · The index** — effort: **high** — Matt, 2026-08-30
  `message_fts` in `migrations/0004_search.sql`: an FTS5 table with two columns,
  `body` and `filenames`, keyed by the message's implicit rowid. Five triggers
  keep it — three on `messages`, two on `attachments` — and every one of them is
  "throw the row away and write what is true now" rather than an incremental
  edit, so the index cannot drift from the table it indexes. **Nothing in Rust
  writes to it.** The backfill is one `INSERT ... SELECT` in the same migration.

  **No new dependency and nothing for a host to install.** FTS5 ships inside the
  SQLite that sqlx already bundles (`libsqlite3-sys` compiles it with
  `-DSQLITE_ENABLE_FTS5`).

  Four things worth knowing before touching this:

  - **The index carries its own copy of the text.** An external-content table
    (`content='messages'`) would save a few megabytes and cannot produce a
    `snippet()`, which is the whole result list — and a filename does not live
    in `messages` at all.
  - **A deleted message has no row here, not an empty one.** A tombstone empties
    `body` but keeps its attachments, so indexing "the message minus its words"
    would leave it findable by the name of the file it was carrying. The trigger
    deletes the row outright and there is a test for exactly that.
  - **Filenames are joined with `char(31)`**, not a space, because a filename may
    contain spaces and a hit has to be able to say *which* file it matched.
    `validate::filename` strips control characters, so U+001F cannot occur inside
    one.
  - **The tokenizer is `porter unicode61 remove_diacritics 2`.** Stemming is why
    `photo` finds `photos`. Query and index are stemmed the same way, so they
    cannot disagree.

  **Two surprises.** First, **there is no such thing as a 5,000-word message** —
  PROTOCOL §4 caps a body at 8,000 characters, which is nearer 1,300 words. The
  acceptance test posts the largest message the API will take, nine of them, and
  asserts the insert path has not become something you can feel; it is a guard
  against a quadratic regression, not a benchmark. Second, the backfill can only
  be tested by *removing* the index: the test posts real messages, drops the
  table and its five triggers (which puts the server exactly where an old one
  is), then replays the shipped migration file over it with `include_str!`.

- ✅ **T-1202 · The endpoint** — effort: **medium** — Matt, 2026-08-30
  `GET /search?q=&room_id=&author_id=&before=&limit=` in
  `routes/search.rs`, over `repo::search`. `crates/linger-server/tests/search.rs`
  covers all three acceptance cases and ten more.

  **The query is rebuilt, never forwarded.** FTS5 has a query language of its
  own, and handing it somebody's raw typing means either a syntax error thrown
  at whoever typed an apostrophe, or a search that quietly did something other
  than what it looked like. `Terms::parse` takes the searchable runs out of the
  string and wraps each in quotes, and quoting is what makes every operator
  inert — `AND`, `OR`, `NEAR`, `*`, `(`, `^` all arrive as words to look for.
  There is no input that reaches `MATCH` as syntax, and a test fires eight of
  them at it. **Do not "simplify" this by passing `q` through.**

  **A snippet arrives as runs, not as a marked-up string.** `SearchHit.snippet`
  is a `SearchSnippetPart[]` of `{ text, matched }`, because any marker
  character is a character a message could contain — and because the obvious
  rendering of a marked-up string is to drop it into HTML. Nothing to parse,
  nothing to escape.

  **Paging is keyset on the message id** and the cursor is that id, hex. UUIDv7
  bytes sort chronologically, so `before` is a range scan and the order is the
  same one the stream reads in. There is no relevance ranking and SPEC §4.12
  says there will not be one.

  Two things that caught the tests out, both about limits rather than search.
  A validation refusal here is **422**, not 400 (`ApiError::validation`). And
  `RATE_SEARCH` (30/min) refills while a burst of requests is in flight, so the
  test asserts the refusal lands *after* the burst is spent rather than on an
  exact request number — the same shape any future rate-limit test wants.

- ⏳ **T-1203 · The search surface** — effort: **medium** — Matt, 2026-08-31
  Whatever SPEC §4.12 decided. Results as a list of snippets with room, author
  and time; clicking one opens that room **and scrolls to that message**. The
  scroll is the hard part and the precedent is T-303's *go to where you left
  off* — the stream is virtualized, so a row's height is a guess until it has
  been drawn, and anything that jumps has to keep re-aiming as real heights
  arrive.
  *Accept:* search for a word from six months ago in a 10,000-message room,
  click the hit, and land on the message with it on screen — not near it.

---

### M11 — DMs and group DMs

*Milestone check: two people hold a conversation nobody else on the server can
see, in any surface — stream, media, search, export, notifications.*

**This is the first time Linger has a private space**, and that is a bigger
change than it looks. Today every member can see every room, so the gateway fans
every frame to everybody and nothing has to check. A DM breaks that assumption
in every fan-out path at once, and **each surface that forgets the check is a
leak**: the media grid showing a DM photo, search returning somebody else's
words, an export containing a conversation you were not in.

**Not a permission system.** AGENTS rule 10 stands. A DM has members; that is
all. No roles, no per-room settings, no admin view of other people's DMs — the
host is a member of a DM or they cannot see it, and there is no override.

- ⬜ **T-1301 · The model and the endpoints** — effort: **high**
  `rooms.kind` (`room` | `dm`), a `room_members` table, create-or-find a DM for
  a set of people (asking twice for the same set gives the same DM), and listing
  the ones you are in. Then the part that matters: **every gateway fan-out grows
  a membership filter**, and the default for a new frame type must be "members
  only" rather than "everybody".
  *Accept:* integration tests where a third member's socket never receives a DM
  frame, cannot fetch its messages by id, and cannot list it.

- ⬜ **T-1302 · DMs in the client** — effort: **high**
  A section of the rail under the rooms, started from a person's card in the
  roster. Typing and presence work inside a DM the way they do in a room.
  **No unread counts** — AGENTS rule 3 does not get an exception because a DM
  feels more urgent. A DM with something new in it can be marked the same
  quiet way a room is, and no number appears anywhere.
  *Accept:* two clients hold a DM; a third client shows no sign it exists.

- ⬜ **T-1303 · Everywhere else a DM can leak** — effort: **high**
  Its own task because it is the security-critical half and it is the half that
  gets forgotten. Media grid, search, export, notifications, link previews and
  the media origin all have to respect membership. An export contains the DMs
  you are in and no others.
  *Accept:* a test per surface, each one asserting a non-member gets nothing —
  and a test that a **removed** member stops seeing a DM they used to be in.

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

- ⬜ **T-1401 · Signalling over the gateway** — effort: **high**
  Offer, answer and ICE candidate frames; who is in voice in which room; join
  and leave. No audio yet — this task ends with two clients having exchanged
  everything they need and nothing playing.
  *Accept:* two clients complete a full exchange across a forced reconnect
  without the session ending up half-connected.

- ⬜ **T-1402 · The audio path** — effort: **treacherous**
  `webrtc-rs` peer connections in the Tauri core, `cpal` capture and playback, a
  full mesh (V2 is eight people, so mesh is fine and an SFU is not needed).
  **Coordinate with Matt before starting.**
  *Accept:* four people, four networks, one hour, no drops. Anything less than
  that is not evidence.

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

## Human checks — things only you can do

Six things are built, tested, and **never once used by a person** — or, in
HC-6's case, never used across two of them. Automated tests prove the code does
what it says. They cannot prove that a window opens, that a 400 MB upload
survives a real network, or that a sound is one you would want to hear. That is
this list.

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

## Parking lot (decisions needed, not tasks yet)

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
