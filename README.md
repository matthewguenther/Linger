<div align="center">
<img src="assets/logo/linger_logo.png" width="600px">
</div>

# Linger

**A small, self-hosted place for a group of friends to hang out.**

Text rooms, presence, and file sharing. One person runs a server and their friends
install a client and connect to it. Not federated. Not a platform. No company in the
middle.

> To linger is to stay somewhere with no agenda and no obligation to be doing
> anything. That is the product thesis in one word.

**Status: pre-alpha, under active construction.** See [SPEC.md](SPEC.md) for the full
product specification.

---

## 🧭 Why this exists

The big chat platforms are built for a stadium of fifty thousand strangers. Linger is
built for a dinner party of eight. Every feature decision resolves against three
principles, in order:

1. **Presence over messages.** The app should feel alive when nobody is typing.
2. **Remove obligation.** No counters, no streaks, no red dots, no "you're behind."
3. **Keep the artifact.** Photos, clips, links, and jokes don't scroll away into nothing.

## 🏠 The case for running your own

There was a stretch of the internet where a group of friends just *had* a place. Somebody
ran it. You knew who that was, you could ask them for things, and the whole arrangement
was a few files on a machine somebody owned. Then everyone moved into one enormous
building owned by a company, and the terms changed:

- **Your conversations sit on someone else's disk, and the company's business depends on
  what it can learn from them.** You are not the customer.
- **Features get held back and sold back to you.** The free experience gets a little
  worse on purpose, because friction is what makes an upsell work. That is not a bug in
  the design... it *is* the design.
- **There's a storefront in the middle of your conversation**, selling cosmetics and
  subscriptions nobody asked for.
- **The product changes under you** whenever a growth target does, and nobody asks.

Linger is the other arrangement. One person runs a server for their friends. The whole
thing is one binary, one SQLite file, and a folder of uploads — you can back it up, move
it to another box, read it with off-the-shelf tools, or walk away with all of it. There
is no account that spans servers, no directory, no company in the middle.

And there is nothing to sell you, structurally: no paid tier, no store, no cosmetics, no
premium anything, and none of it held back for later. It's AGPL-3.0, so if someone runs a
modified server for other people, those people get the source. **Zero telemetry** — not
opt-in, not anonymous, not crash reports.

This is not an attempt to build a better platform. It's an attempt to not need one.

## 🗣️ Vocabulary

These terms are used everywhere — UI, code, docs, error messages:

| Concept | Term |
|---|---|
| An instance | **a server** |
| A text channel | **a room** |
| Being present in a room | **in the room** |
| The person running it | **the host** |
| Media/link archive | **media** |
| A user's status card | **their status** |

## ✨ What it does (V1)

- 🪑 **Rooms you're in** — focusing a room means you're in it; others see
  occupancy, and each person has a personal **entrance sound** that plays on arrival
- 👥 **A roster-forward layout** — people are the primary surface, not a gutter; each
  friend is a card showing presence, the room they are in, and their status
- 🔕 **No unread counts** — a "you left off here" line and a subtle label-weight change,
  never a badge; direct person-to-person mentions are the only real notification
- ✍️ **Styled names** (the AIM feature) — curated fonts, a named 16-color palette,
  gradients, shimmer/glow; and **statuses** with away messages
- 🗄️ **Media** — everything ever shared, browsable and filterable; star things to
  keep them forever
- 📝 **Text that stays text** — a small markdown subset (bold, italic, strikethrough,
  code, quotes, lists, links), plus edit, delete and reply. Message bodies are parsed
  into a tree and drawn as elements; **no raw HTML from a message, ever**
- 🎚️ **Reactions by weight** — a fixed palette of 12; six identical reactions render
  denser and larger, not "👍 6"
- 📁 **File sharing** — 500 MB files, resumable uploads, **EXIF always stripped**, a
  poster frame and a blurhash generated for you
- 🖥️ **Desktop client** for Linux, Windows, and macOS (Tauri 2, not Electron)
- 🏘️ **Several servers at once** — a list in the rail with a live dot each, and
  `+ add` to join another. Each one is its own sign-in, its own people and its own
  rooms; signing out of one leaves the rest alone
- 📦 **Full export** — any member can export all messages and media, any time, no
  gatekeeping

## 🚫 What it will never have

XP, levels, streaks, or engagement metrics. Federation. A bot marketplace. A
role/permission matrix. Threads. `@everyone`. Algorithmic ordering. Unread badges.
**Telemetry or analytics of any kind.** A paid tier, a store, or anything else to sell
you.

The scope discipline is the product.

## 🔐 Privacy and the threat model

Stated plainly:

> Messages and files are encrypted in transit (TLS) and at rest on the host's disk.
> **The person running the server can read everything on it.** There is no
> end-to-end encryption. Run your own server, or trust the person who runs the one
> you're on. If you need cryptographic guarantees against your host, use Signal.

Other privacy properties that *are* guaranteed:

- Activity sharing is **default off**, resolves only against a bundled app registry,
  and **never reports window titles** — the type system has no field for them
- Browsers report as the browser ("Firefox"), never the site or tab
- EXIF (including GPS) is stripped from every uploaded image, no toggle
- Zero telemetry, zero crash reporting, zero phone-home — not even opt-in

## 🤖 The AI stance

**None. No AI features anywhere in Linger.** No participants, no suggested replies, no
drafting, no sentiment analysis, no summaries, no semantic search, no transcription, and
no model endpoint — not even a local one.

An earlier draft of the spec planned a set of local-only, opt-in features for after V1
shipped. That's cut. Standing up a model alongside a chat app for eight friends is a real
cost pushed onto whoever is hosting, for something nobody asked for — and a "local only"
promise is one bad afternoon away from becoming an API key and your friends' messages
leaving the box. Not having the feature is the only version of that promise that can't
rot.

There's also a simpler reason. The value of a friend replying is that a person chose to
spend attention on you. Put something in the room that also replies and every reply gets
cheaper, because you can no longer tell what was chosen.

If it's ever revisited it'll be a deliberate decision, written down in
[SPEC.md §8](SPEC.md) with the reasoning, not a thing that quietly appears in a release.

## 📦 Installing the client

**New here? The [user guide](docs/user-guide.md) covers installing and using
Linger in plain language** — no command line, nothing assumed.

Grab the installer for your platform from
[Releases](https://github.com/matthewguenther/Linger/releases). Linux and
Windows are built; **macOS is not built yet**, so on a Mac you build from a
checkout (see Development below).

**Windows will warn you.** You get *"Windows protected your PC"*, and *Run
anyway* is hidden behind the *More info* link. That is SmartScreen saying the
installer is not code-signed, which is true — see
[docs/decisions.md](docs/decisions.md) for why it isn't, and *Cutting a release*
for the difference between that and the signature on updates, which is in place.
Nothing about the download is broken.

Once installed, the app keeps itself up to date: it checks at launch, says so
quietly in the status bar, and downloads nothing until you ask it to under
*settings → updates*.

## 🚀 Running a server

**The [host guide](docs/host-guide.md) is the step-by-step version** — domain,
DNS, install, invites, backups, updates — written for somebody who is not a
developer. What follows is the short form.

Target: a working server in under 15 minutes. One binary plus one data directory, or:

```bash
cd deploy
# point two DNS records at this machine: linger.example and cdn.linger.example
# edit compose.yaml and Caddyfile: set both to your domain
docker compose up -d
docker compose logs linger   # prints a one-time host-setup URL on first run
```

Caddy is bundled so TLS certificates are automatic. That is not only about
padlocks: **an installed client only talks to a server over `https`.** The
shipped app's content-security policy allows nothing else, so a server reached
at a bare `http://` address — including one on your own machine — is reachable
from `pnpm tauri dev` and from a browser, and not from the installer. Point a
name at the box and let Caddy do the certificate.

**Why two names.** Files people upload are served from `cdn.` in front of your domain,
never from the domain itself. A file somebody sends you is somebody else's file, and a
browser gives a file a lot of latitude when it comes from the same place as the app.
Served from its own name it is a stranger, which is what it is. The server holds the line
itself rather than trusting the setup: files answer on the `cdn.` name and nowhere else,
and everything else answers everywhere else. It will not start if you point both names at
one place. If `cdn.` is taken, `LINGER_MEDIA_DOMAIN` names a different host instead — it
only has to be a different one, and the Caddyfile block has to match.

The setup URL in that log is meant for the desktop client, not a browser: open Linger
and paste it into the first box. It creates your account, makes you the host, and names
the server. It works once, and a restart replaces it.

Everything else you need to run the place is inside the app. As the host you get two
extra controls on the rail — `+ room` next to the room list, and `manage` next to the
server's name — and they open one panel with four sections: rooms, invites, people, and
the server's own name and accent. Nobody else sees those controls at all. Making a room,
renaming or archiving one, reordering the rail, handing out invite links and revoking
them, and renaming the server are all done there. None of it needs `curl` and none of
it is a config file.

**Removing somebody** is on their card in `who's around`, and only you can see it. It
takes effect at once: they lose their sign-in, their open connection is closed, and any
invite links they had made stop working. What they wrote stays where it is — removing a
person is not deleting what they said. There is no ban, because a ban needs something
durable to ban by and Linger stores no addresses and no device ids. The `people` section
of the host panel lists everybody you have removed and lets them back in; that is not an
undo, so they sign in again with the password they always had.

Inviting people works like the setup link. The invites screen makes the link and copies
it for you — it is your server's address with the code hung off it,
`https://linger.example/invite/CODE` — and whoever you send it to pastes that into the
same box the setup link goes in. The box also takes a bare address (`linger.example`)
for signing back in.

**Files.** Uploads go straight from the app to storage — the bytes never pass through
the part of the server that answers questions about messages. Anything over 8 MB is sent
in 8 MB pieces, so a connection that drops costs you one piece and not the file; sending
the missing pieces picks up where it stopped.

Every image is decoded and written out again on the way in. That is what removes EXIF —
the block of camera data attached to a photo, which for a phone photo usually includes
the GPS coordinates of where you were standing. There is no setting for this. It also
means an image that is secretly two files at once stops being the second one. A WebP
comes back as a PNG, because the encoder Linger uses can read WebP but not write it; the
picture and the file extension both stay honest.

Videos get a poster frame and their duration if the machine has `ffmpeg` on it. The
published Docker image includes it. Without it, videos still upload and play — there is
just no still frame to show first.

What a server takes: images (JPEG, PNG, GIF, WebP), video (MP4, WebM, MOV), audio (MP3,
Ogg, WAV, FLAC, AAC, M4A), and ordinary documents and archives — PDF, zip, tar, the
office formats, plain text. Not SVG, not HTML, not programs. Those are scripts wearing a
file extension, and the safe way to handle them is not to store them. A file the server
does not recognise is kept as a plain download rather than being shown in place.

**Sharing one** is `+ file` next to the composer, or dragging it onto the composer, or
pasting it. It starts going up straight away, with a small bar per file, and you can
still type while it does. A message with a file on it does not need any words. Pictures
land in the conversation at their own shape, up to 400px tall, and click to expand;
video gets a player and its poster frame; anything else is one line with the filename,
the size, and `save`, which hands the file to your browser rather than opening it in the
app.

**Media** is the third thing in the rail, above the rooms, and it is everything anybody
has ever shared here: pictures, video, sound, files, links, and the messages people
pinned. Filter it by person, by kind, or between two dates. Click anything to go back to
the message it came from, in the room it was posted in — that is the point of it. A star
keeps an item forever; starred things sort to the front and are never swept by the
year-old cleanup.

**Links** in a message get one line under it: the site's icon, the page title, and the
domain. Not a 400px billboard with somebody's hero image in it.

The server fetches that line, once per address, for everybody — your client never
touches the linked site, and neither does anyone else's. If it did, every site anyone
ever linked would collect the IP address of every person who scrolled past the message.
The icon comes down inside the answer rather than as a web address, so drawing the card
makes no request at all.

The fetch itself is careful, because a link in a message is a stranger telling your
server what to go and open. It looks up the address itself and refuses the whole name if
any part of it points inside a network — your router, your NAS, a cloud metadata service
— then holds the connection to the address it checked, so the name cannot change
underneath it. It follows at most three redirects and checks every one, gives up after a
few seconds, and stops reading after 256 KB. A link it will not fetch still gets a card
with its domain on it.

**Where files live.** By default, in the data directory next to the database — right for
a home server, and the reason backup is two paths. A server that would rather keep files
in an S3 bucket sets `LINGER_STORAGE=s3` and five more variables:

```yaml
LINGER_STORAGE: s3
LINGER_S3_BUCKET: linger
LINGER_S3_ENDPOINT: https://ACCOUNT.r2.cloudflarestorage.com
LINGER_S3_REGION: auto          # "auto" for R2; a real region on AWS
LINGER_S3_ACCESS_KEY_ID: ...
LINGER_S3_SECRET_ACCESS_KEY: ...
```

Any S3-compatible store works. **Cloudflare R2 is the one to pick** — it charges nothing
for bytes going out, which for a place people share videos is most of the bill. Backblaze
B2 is the runner-up. Plain AWS S3 will bill you for every view of every photo.

On S3 the download is a redirect to the bucket, so the last hop is the bucket's response
rather than one Linger writes. Two of the four safety headers can be signed into that
URL and are; the other two cannot, because S3 has no way to send them. What holds instead
is that nothing which could run in a browser is storable at all — no SVG, no HTML, no
programs — and every file that is not an ordinary image, video or audio clip is handed
over as a download, which a browser never renders. Those instructions are stored on the
object as well as signed into the link, so they hold even if the bucket is reached some
other way. If you put a CDN in front of the bucket and want the missing headers back, add
them as a response rule there.

The server never sits in the middle of the bytes: uploads go straight from the app to the
bucket, and downloads are a redirect to the bucket. It does pull each upload down once,
briefly, to check what it really is and strip EXIF, and deletes that copy afterwards — so
a server on S3 still needs a data directory, just not much room in it. If the endpoint or
the keys are missing, the server says so and stops rather than starting up unable to
accept a single file.

**How much, and for how long.** A server holds 50 GB of files in total and keeps each
one for a year. Both are yours to change, in the compose file with everything else:

```yaml
LINGER_POOL_BYTES: 250GB        # or a plain number of bytes
LINGER_FILE_EXPIRY_DAYS: 365    # or `off`, to keep every file for good
```

The pool is the ceiling for the whole server, and uploads in flight count against it, so
nobody can start fifty files that will not fit. When it is full, the next person to
attach something is told before they send a single byte rather than after.

Expiry is what keeps a shared server from filling up with a year of screenshots nobody
will look at again. There are two ways to say "keep this": star the file in the media
collection, or pin the message it is on. Neither ever expires. Files on messages somebody
deleted are removed straight away — they were unreachable and still taking up room. The
picture on somebody's status never expires either, whatever its age: a status quietly
losing its image after a year is not something anyone would connect to a file expiry they
never set. The sweep runs when the server starts and every few hours after that, and the
status bar at the bottom of the app shows what is used against what there is.

**Backup** is the whole point of self-hosting your friendships — it's two paths:
`data/linger.db` and `data/objects/`. (On a server using S3, `data/objects/` is empty
and the bucket is the other half.) A cron one-liner:

```bash
sqlite3 data/linger.db ".backup data/backups/linger-$(date +%F).db"
```

**Locked out?** There is no reset email and no reset link. Linger has no address to
send one to, and the setup link only exists while a server has nobody on it — so the
proof that the server is yours is that you can get to the machine it runs on. The way
back in is a command on the box:

```bash
docker compose stop linger                            # one database, one writer
docker compose run --rm linger reset-password matt
docker compose start linger
```

It prints a new password for that account. Sign in with it, then change it in the app
under your own name. Everything that account was signed in on gets signed out, because
the usual reason to reset a password is that somebody else might have had it.

To choose the password yourself, pipe it in — never type it as an extra argument, where
it would sit in your shell history and be readable by anyone running `ps`:

```bash
echo 'the one I wanted' | docker compose run --rm -T linger reset-password matt --stdin
```

Stop the server first. A second program writing to the same SQLite file is the one thing
the database is careful about; if you forget, the command waits for the server rather
than failing, but stopping it is the honest version.

What this does not do: it does not make anybody the host, it does not touch anything else
on the server, and a username that is not on this server is an error rather than a new
account.

## 🛠️ Development

```
crates/linger-core/       shared types, IDs, palette — the wire contract
crates/linger-server/     axum REST + WS gateway, SQLite (WAL), object store
crates/linger-activity/   foreground-app detection, per-OS backends
client/                   Tauri 2 shell + React/TypeScript frontend
registry/apps.json        bundled app registry for activity detection
deploy/                   Dockerfile, compose, Caddyfile
docs/                     host-guide.md and user-guide.md (the two
                          plain-language guides), screenshots; docs/tasks/
                          archives closed milestones, docs/decisions.md
                          records settled questions
scripts/                  check.sh (the whole local gate), lint-rules.sh,
                          version-check.sh (one version number, four files),
                          updater-key.sh (the update signing key),
                          signing-preflight.sh (is that key the right one?),
                          minio-test.sh (the S3 backend, against a real MinIO),
                          fetch-fonts.sh (re-vendors the twelve bundled faces)
```

Read first, in order: [SPEC.md](SPEC.md) → [ARCHITECTURE.md](ARCHITECTURE.md) →
[PROTOCOL.md](PROTOCOL.md) → [AGENTS.md](AGENTS.md). The docs are the source of truth.
Contributing — with any coding agent, or none — starts at
[CONTRIBUTING.md](CONTRIBUTING.md); the work queue is [TASKS.md](TASKS.md).

```bash
# server + core + activity (no GUI deps needed)
# optional: `ffmpeg` on PATH makes the video-poster test run instead of skipping
cargo test --workspace

# frontend
cd client && pnpm install && pnpm check && pnpm test

# desktop client (needs system webview deps; on Debian/Ubuntu:
#   sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
#                    libayatana-appindicator3-dev librsvg2-dev)
cd client && pnpm tauri dev
```

Before you push, run what CI runs — `scripts/check.sh` does all of it in one
go, including a lint that rejects AI attribution and dropped vocabulary
(`scripts/lint-rules.sh`). Piece by piece it is:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd client && pnpm check && pnpm test && pnpm exec vite build

# the desktop shell — outside the workspace, so it needs its own pass
cd client/src-tauri && cargo clippy --all-targets -- -D warnings && cargo test

# the S3 storage backend, which needs a bucket to talk to; skipped by the line
# above. Starts a throwaway MinIO, runs the tests, stops it again.
scripts/minio-test.sh
```

Things that surprise people the first time:

- **Wire types are generated, not written.** They're defined once in `linger-core` and
  exported to TypeScript by `ts-rs` into `client/src/generated/`. `cargo test -p
  linger-core` regenerates them. The output is committed, and CI fails if it drifts from
  the Rust source — so commit the regenerated files with your change. Never hand-write a
  type that crosses the wire.
- **The name palette is generated too.** The sixteen colors are defined once in
  `linger-core::palette::PALETTE`, and the same `cargo test -p linger-core` run writes
  them out as `client/src/generated/palette.generated.css` — one `--name-<key>` custom
  property per color, per theme. That file is committed and covered by the same drift
  check. Never write a hex or an `oklch()` literal into the frontend: a color is a
  palette key everywhere else in the codebase.
- **The fonts are vendored, not fetched.** The twelve faces (SPEC §5.7) are
  downloaded, subset to Latin, converted to woff2 and committed under
  `client/src/fonts/` by `scripts/fetch-fonts.sh`. You never need to run it to
  build — run it only when a face is added to `linger-core::FONTS` or to pull an
  upstream fix. **No CDN and no remote font URL, ever**: a remote face is a
  fingerprinting vector and a dependency on somebody else's uptime. The whole set
  is about 800 KB; `assets/fonts/README.md` has the details.
- **The version number lives in four files** — `client/package.json`,
  `client/src-tauri/Cargo.toml`, `client/src-tauri/tauri.conf.json` and the root
  `Cargo.toml`. The first three are the desktop app; the fourth is the server,
  and so the version its `GET /health` reports and the tag its container image
  is published under. Bump all four together; `scripts/version-check.sh` (which
  CI runs) fails if they disagree. If they drift, a release ships under the old
  number and every installed copy decides it is already up to date, which looks
  exactly like success.
- **Renaming a wire type leaves an orphan.** `ts-rs` writes files but never deletes them,
  so the old `.ts` file stays behind and the drift check won't catch it. Delete it by hand.
- **There are two content-security policies and you develop under the loose one.**
  `client/src-tauri/tauri.conf.json` carries `csp` and `devCsp`, and Tauri picks
  between them when it compiles: `pnpm tauri dev` gets the one that allows
  `http://localhost:*`, and every bundle the release workflow builds gets the one
  that does not. So a shipped copy cannot reach a server at a bare `http://`
  address, on purpose — a page that could reach `http://localhost:*` could knock on
  every other service running on that machine. If you tighten either policy, tighten
  both; `client/src-tauri/tests/csp.rs` fails if they drift apart, and it is also
  what stops somebody removing `ipc:` and `http://ipc.localhost`, which are Tauri's
  own IPC channel rather than a relaxation.
- **`client/src-tauri` is not in the root cargo workspace.** It links against system
  webview libraries that CI and server boxes don't have. Build it with `pnpm tauri`, not
  `cargo`.
- **pnpm comes from corepack** and the version is pinned in `client/package.json`. Run
  pnpm commands from inside the repo so it picks up the pin.
- **`cargo test --workspace` skips the client shell**, for the same reason. Its tests
  are `cd client/src-tauri && cargo test`, and CI runs them in the `tauri-shell` job.
  A few need a real desktop session (an unlocked keyring, for one) and are marked
  `#[ignore]` — run those with `cargo test -- --ignored` when you're sitting in front
  of the machine.
- **The WebSocket connection is not in the WebView.** It lives in the Tauri core
  (`client/src-tauri/src/gateway.rs`) and sends the frontend two events: a connection
  status and each sequenced server frame. The frontend has one store,
  `client/src/lib/gateway.ts`, and no connection code at all. Reconnecting, resume, and
  sequence accounting are Rust's problem, and they're covered by tests that talk to a
  real socket.
- **The message list is virtualized**, so only the rows on screen exist in the DOM
  (`client/src/stream/`). Two rules follow from that and both are easy to trip over.
  Space between rows must be *padding*, never margin — the virtualizer measures each
  row's own box and a margin sits outside it. And a row's height is a guess until it
  has been drawn once, so anything that scrolls to a position has to keep re-aiming as
  the real heights arrive; jumping once lands in the wrong place.
- **The roster is one component rendered in one of two places** (`client/src/roster/`).
  Wide windows get it as the right-hand column; under 880px it renders inside the stream
  column as a horizontal strip above the composer, and it is never hidden or folded into
  a menu. React decides which — `client/src/lib/layout.ts` holds the only copy of that
  width and the frame carries the answer as `data-narrow` — so there is no media query
  to go looking for, and nothing is rendered twice.
- **A status is drawn in two places by one component** (`client/src/status/`). It shows
  in the roster card when you open it, and in the card you get by clicking a name in the
  stream. The popover is rendered through a portal into the document body rather than
  inside the message: the stream is virtualized, so anything drawn inside a row is
  clipped by the scroller. Setting an away message is what makes you away — it is a
  field on the status, not a mode — and it takes two writes, in this order: `PATCH /me`
  saves it and is the only thing that stamps `away_since`, then the gateway frame tells
  everyone. The room leave has to go before the away frame, because the server sets
  `around` on any room leave.
- **A name is drawn from custom properties, not from React** (`client/src/lib/names.ts`,
  `client/src/styles/names.css`). Everything a person controls about their own name —
  face, weight, slant, one palette color or a gradient of two, shimmer or glow — becomes
  a handful of `--person-*` properties on the element, and the stylesheet does the
  painting. That is what makes the reader's `normalize everyone` switch a single
  attribute on `<html>` instead of a flag threaded through every component that draws a
  person. The gradient angle is fixed at 92° and is written down exactly once, in that
  stylesheet. Effects are off in compact and IRC density, and shimmer is off under
  `prefers-reduced-motion`.
- **Theme and warmth are attributes on `<html>`, nothing more**
  (`client/src/lib/theme.ts`). Dark, light, or follow the system; and after
  about 7pm local the background and text go roughly 200K warmer, which is one
  block of tokens in `styles/tokens.css` and can be switched off. Every warm
  value keeps its cool twin's OKLab *lightness* and only moves the hue, which is
  what keeps the name palette's contrast guarantee true after dark — the CI
  property test reads all 16 colors against the warm backgrounds too. Sunset is
  approximated by the clock on purpose: knowing the real one means knowing where
  you are, and this app does not ask.
- **The host's controls are absent for everybody else, not greyed out**
  (`client/src/host/`). One panel over the stream column with four sections — rooms,
  invites, people, the server itself — reached from two small controls on the rail that
  only a host is shown. A disabled button would be a permission matrix drawn in CSS, and this
  product does not have one; the server refuses the request either way, which is the
  actual lock. The panel keeps no copy of the room list: every save goes to the server,
  the server fans the change out, and the panel re-renders off the same store the rail
  does, so the two cannot disagree and the other clients see it too.
- **The only thing that interrupts you is somebody naming you.** A mention, or a person
  you ticked in `notify me when`, raises a desktop notification through
  `tauri-plugin-notification`. Nothing else does, and there is no badge for one to hang
  off. On Linux that means a notification service on the session bus — Plasma, GNOME and
  `dunst` all provide one. Without one, or if you turn notifications off at the OS level,
  the message still arrives in the stream; you just don't get interrupted about it.
- **The app updates itself, and only ever because you asked it to**
  (`client/src-tauri/src/updates.rs`, `client/src/lib/updates.ts`). It checks for
  a new version at launch and when you open settings. A waiting update gets one
  quiet word in the status bar; downloading and restarting is a second click in
  settings, under *updates*. Nothing installs on its own and nothing pops up
  mid-sentence. Every update is verified against the project's signing key
  before a byte of it is installed, with no bypass — a build compiled without a
  key configured refuses updates rather than taking an unsigned one. The WebView
  is granted none of the updater plugin's own permissions; it calls two narrow
  commands in the Rust shell, the same way it does for the gateway and the
  keyring.
- **Signing in needs a keyring to be remembered.** Each server's refresh token goes to
  the OS keyring — Keychain, Credential Manager, or a Secret Service provider like
  gnome-keyring or KWallet — as its own entry, plus one small entry listing which
  servers you have. Without a keyring, or with `pnpm dev` in a plain browser, the app
  still works; it just says so and asks you to sign in again next launch.

### Cutting a release

Installers are built by `.github/workflows/release.yml`, which only a tag fires.
That job cuts a release and uses the signing key, so it has no business running
on an ordinary push.

Once, before the first release ever ships:

```bash
scripts/updater-key.sh
```

That generates the update signing key, writes its public half into
`client/src-tauri/tauri.conf.json` (commit that), and prints what to do with the
private half: back it up offline, and add it plus its password as two repository
secrets:

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.local/share/linger/updater.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD   # prompts; paste the password
```

The first argument is the secret's *name*; the `<` feeds the key file in as its
*value*. **Losing that key means you can never ship an update to an installed
copy again** — the only way back is reinstalling every machine by hand. It is
generated once and never regenerated.

To check the secrets without cutting a release, run the `release` workflow from
the Actions tab. A manual run does the preflight and stops: it signs a throwaway
file with the secret and checks that the key id matches the public key in
`tauri.conf.json`. "Present" is not "correct" — a key that signs fine but isn't
the mate of the committed one produces a green release that no installed copy
will accept.

Then, per release:

```bash
# bump all three version numbers, then:
scripts/version-check.sh
git commit -am "chore: 0.2.0"
git tag v0.2.0 && git push origin main v0.2.0
```

The workflow builds Linux and Windows, signs the updater artifacts, and opens a
**draft** GitHub release carrying `latest.json`. Read it, then publish it —
publishing is what makes installed copies see the update, and it is deliberately
a human's click rather than a side effect of pushing a tag.

The same tag also publishes the server as a container image
(`.github/workflows/image.yml`): `ghcr.io/matthewguenther/linger` gets `0.2.0`,
`0.2` and `latest`, for x86-64 and ARM64. That is what `deploy/compose.yaml`
pulls, so moving `latest` is what makes a host's `docker compose pull` find a
new server. Nothing about it is signed and nothing auto-updates — a host
chooses when to pull. Running that workflow from the Actions tab builds the
image and pushes nothing, the same way a manual `release` run checks the
signing key and stops.

**One thing to do by hand, once ever:** the first image push creates the ghcr
package as *private*, and a private package means `docker compose up` fails
with `unauthorized` for everybody who is not you. Set it public at
`github.com/users/<you>/packages/container/linger/settings` after the first
tag.

**macOS is deliberately not built yet**, and the Windows installer is not
code-signed. Both are decisions rather than gaps — see
[docs/decisions.md](docs/decisions.md). The short version: signing costs a few
hundred dollars a year per platform and buys little for a server you hand to a
friend group, and an *unsigned* macOS build would be worse than none, because the
later switch to a real signature is what breaks an app the updater has replaced
in place.

Two different signatures are easy to confuse here. The **update** signature is in
place already: it is what stops a later update from being tampered with, and the
app verifies it with no way to skip. The **installer** signature is what the
operating system checks the first time you run a download, and that is the one
this project does not have yet.

Current work queue lives in [TASKS.md](TASKS.md).

## 🗺️ Roadmap

- **V1** — replaces the text half of a big chat platform for one friend group
  (see [SPEC.md §6](SPEC.md))
- **Later (still V1, not on the critical path)** — entrance sounds (T-901…T-903);
  **ambient activity detection** (T-911…T-917: opt-in, default off, bundled app
  registry, window titles never read or transmitted). Both are in the spec.
  Neither is next. See [TASKS.md](TASKS.md) *Backburner*.
- **V2** — voice rooms, ambient voice, DMs, search, knock, mobile
- **V3 or never** — opt-in directory, sandboxed client scripting, custom emoji

There is no AI phase. It was on this list once; it was cut
([SPEC.md §8](SPEC.md)).

## 📜 License

[AGPL-3.0](LICENSE). If you run a modified server for other people, they get the source.
That's the deal.
