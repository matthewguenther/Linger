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

Discord's interface is built for a stadium of 50,000 strangers. Linger is built for a
dinner party of eight. Every feature decision resolves against three principles, in
order:

1. **Presence over messages.** The app should feel alive when nobody is typing.
2. **Remove obligation.** No counters, no streaks, no red dots, no "you're behind."
3. **Keep the artifact.** Photos, clips, links, and jokes don't scroll away into nothing.

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
  friend is a card showing presence, room, activity, and their status
- 🔕 **No unread counts** — a "you left off here" line and a subtle label-weight change,
  never a badge; direct person-to-person mentions are the only real notification
- 🎮 **Ambient activity** — opt-in, default-off sharing of the app you're using,
  resolved against a bundled registry; **window titles are never read or transmitted**
- ✍️ **Styled names** (the AIM feature) — curated fonts, a named 16-color palette,
  gradients, shimmer/glow; and **statuses** with away messages
- 🗄️ **Media** — everything ever shared, browsable and filterable; star things to
  keep them forever
- 🎚️ **Reactions by weight** — a fixed palette of 12; six identical reactions render
  denser and larger, not "👍 6"
- 📁 **File sharing** — 500 MB files, resumable uploads, **EXIF always stripped**
- 🖥️ **Desktop client** for Linux, Windows, and macOS (Tauri 2, not Electron)
- 📦 **Full export** — any member can export all messages and media, any time, no
  gatekeeping

## 🚫 What it will never have

XP, levels, streaks, or engagement metrics. Federation. A bot marketplace. A
role/permission matrix. Threads. `@everyone`. Algorithmic ordering. Unread badges.
**Telemetry or analytics of any kind.** AI participants in the conversation.

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

**No AI in the conversation.** No AI participants, no suggested replies, no sentiment
analysis — ever. Planned V2 features (semantic search over history and media, catch-up
summaries, transcription/alt-text) run **only against a host-configured local endpoint**
(Ollama-compatible). No cloud default, no cloud fallback; if no endpoint is configured,
the features don't appear at all.

A self-hosted Linger server can run all of this on the box. A hosted competitor
structurally cannot — their version requires shipping your friends' conversations to
a third party.

## 🚀 Running a server

Target: a working server in under 15 minutes. One binary plus one data directory, or:

```bash
cd deploy
# edit compose.yaml: set LINGER_DOMAIN to your domain
docker compose up -d
docker compose logs linger   # prints a one-time host-setup URL on first run
```

Caddy is bundled so TLS certificates are automatic.

**Backup** is the whole point of self-hosting your friendships — it's two paths:
`data/linger.db` and `data/objects/`. A cron one-liner:

```bash
sqlite3 data/linger.db ".backup data/backups/linger-$(date +%F).db"
```

## 🛠️ Development

```
crates/linger-core/       shared types, IDs, palette — the wire contract
crates/linger-server/     axum REST + WS gateway, SQLite (WAL), object store
crates/linger-activity/   foreground-app detection, per-OS backends
client/                   Tauri 2 shell + React/TypeScript frontend
registry/apps.json        bundled app registry for activity detection
deploy/                   Dockerfile, compose, Caddyfile
```

Read first, in order: [SPEC.md](SPEC.md) → [ARCHITECTURE.md](ARCHITECTURE.md) →
[PROTOCOL.md](PROTOCOL.md) → [AGENTS.md](AGENTS.md). The docs are the source of truth.

```bash
# server + core + activity (no GUI deps needed)
cargo test

# frontend
cd client && pnpm install && pnpm check

# desktop client (needs system webview deps; on Debian/Ubuntu:
#   libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev)
cd client && pnpm tauri dev
```

Wire types are defined once in `linger-core` and exported to TypeScript via `ts-rs`
(`client/src/generated/`). Never hand-write a type that crosses the wire.

Current work queue lives in [TASKS.md](TASKS.md).

## 🗺️ Roadmap

- **V1** — replaces the text half of Discord for one friend group (see [SPEC.md §6](SPEC.md))
- **V2** — voice rooms, ambient voice, DMs, search, knock, local-AI features, mobile
- **V3 or never** — opt-in directory, sandboxed client scripting, custom emoji

## 📜 License

[AGPL-3.0](LICENSE). If you run a modified server for other people, they get the source.
That's the deal.
