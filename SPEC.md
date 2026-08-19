# Linger — Product Specification

**Version:** 0.1 (pre-implementation)
**Status:** Draft for V1 build
**Author:** Matt Guenther
**License:** AGPL-3.0

---

## 1. What this is

Linger is a small, self-hosted place for a group of friends to hang out. Text rooms,
presence, and file sharing. One person runs a server; their friends install a client
and connect to it.

It is not federated. It is not a platform. There is no company in the middle, no
account system that spans servers, and no discovery mechanism in V1.

It borrows the good parts of Discord, IRC, Ventrilo, and AIM, and deliberately omits
the machinery those platforms added once they had to serve millions of people or
generate revenue.

**The name.** To linger is to stay somewhere with no agenda and no obligation to be
doing anything. That is the product thesis in one word.

An instance is **a server**. Earlier drafts coined a private vocabulary for these
concepts; it was dropped before any UI was built. Plain words travel further than
clever ones, and everyone joining a Linger server already knows what a server, a
room, and a status are.

### Vocabulary (use these terms everywhere — UI, code, docs, error messages)

| Concept | Term | Never call it |
|---|---|---|
| An instance | **a server** | stoop, guild, workspace, community |
| A text channel | **a room** | channel |
| Being present in a room | **in the room** | sitting in, joined, connected |
| The person running it | **the host** | admin, owner |
| Media/link archive | **media** | the shelf, gallery |
| A user's status card | **their status** | their sign, bio, about me |

Language does more design work than features. Hold this vocabulary in the code too:
`RoomId`, not `ChannelId`. One thing "server" is *not*: the `linger-server` binary is
the process that hosts an instance, never the instance itself.

---

## 2. Design thesis

Discord's interface is built for a stadium of 50,000 strangers. Linger is built for a
dinner party of eight. Nearly every complaint about Discord traces back to that
mismatch:

| Stadium feature | Effect on 8 friends |
|---|---|
| Unread badge counts | Turns friends into a to-do list |
| Role colors and hierarchy | Invents status where none existed |
| `@everyone` | A weapon nobody should have in a group of 8 |
| Infinite scroll, no archive | Everything you shared is functionally destroyed |
| "Online" member list | Meaningless — everyone is always online |

**Core problem to solve: a quiet Discord server looks abandoned. A quiet Linger
server with four people in it must feel populated.**

### Three principles

Every feature decision resolves against these, in order.

1. **Presence over messages.** The app should feel alive when nobody is typing.
   Ambient signal — who is around, what they are doing — carries the room.
2. **Remove obligation.** No counters. No streaks. No red dots. No "you're behind."
   Nothing that converts a friendship into a queue.
3. **Keep the artifact.** A friendship produces things: photos, clips, links, jokes.
   Do not let them scroll away into nothing.

### Anti-goals (never build these)

- XP, levels, streaks, leaderboards, or engagement metrics of any kind
- Federation
- A bot marketplace
- A role/permission matrix
- Video calls or screen sharing
- Threads
- Any algorithmic ordering
- Analytics or telemetry of any kind, opt-in or otherwise
- Ephemeral-by-default messaging (fights principle 3)

---

## 3. Layout

Discord's layout is `[server rail] [channel list] [messages] [member list]`. Linger
inverts the priority. **People are the primary surface, not a gutter.**

```
┌──────────┬────────────────────────────────────┬─────────────────┐
│ SERVERS  │  #garage                           │  WHO'S AROUND   │
│          │  Matt, Callie in the room          │                 │
│  ● home  │                                    │ ┌─────────────┐ │
│  ○ work  │                                    │ │● Callie     │ │
│  + add   │  ─── Saturday morning ───          │ │ in #garage  │ │
│          │                                    │ │ Elden Ring  │ │
├──────────┤  Callie   9:14                     │ │ 40m         │ │
│ ROOMS    │  │ did the drive get here          │ └─────────────┘ │
│          │  │ yet                             │ ┌─────────────┐ │
│  #porch  │                                    │ │● Dave       │ │
│  #garage │  Matt     9:31                     │ │ Blender     │ │
│  #shop   │  │ yeah, mounting it now           │ │ ♪ Bill Evans│ │
│          │  │ [img]                           │ └─────────────┘ │
│  media   │                                    │ ┌─────────────┐ │
│          │                                    │ │○ Jen        │ │
│          │  ┌──────────────────────────────┐  │ │ 2h · "back  │ │
│          │  │ say something                │  │ │ after work" │ │
│          │  └──────────────────────────────┘  │ └─────────────┘ │
└──────────┴────────────────────────────────────┴─────────────────┘
```

**The roster (right) is a card stack, not a name list.** Each card shows: name in the
user's own styling, presence dot, which room they are in, their current activity
(if shared), and their status. Offline users show last-seen and their away message.
This panel is what makes an empty server feel like a house with the lights on.

On narrow windows the roster collapses to a horizontal strip above the composer, not
into a hamburger menu. It is never fully hidden by default.

---

## 4. Feature specifications

### 4.1 Rooms you are in

A room is a place, not a filing cabinet.

- Focusing the app on a room means you are **in the room**. Others see this.
- Room headers show occupancy: `#garage · Matt, Callie`
- Sidebar rooms show a small stack of who is in them.
- Backgrounding the app or idling >90 seconds takes you out of the room.

**Entrance sounds.** Each user picks a personal sound that plays for others already
in a room when they arrive. This is the cheapest piece of emotional design
available and no modern app does it, because a 50,000-member server can't.

Requirements:
- Ship 12–16 curated sounds (<1.2s, normalized to −16 LUFS).
- Custom upload allowed: max 2 seconds, max 200 KB, transcoded server-side to Opus and
  loudness-normalized. Reject anything longer — do not truncate.
- Rate limit: a given user's sound plays at most once per 5 minutes per listener,
  regardless of how many times they enter.
- Global mute, per-user mute, and automatic mute between 22:00–08:00 listener-local
  time (default on).

### 4.2 No unread counts

Delete the badge. It is a slot machine that converts friends into obligations.

Replace with:
- A **"you left off here"** divider at the last-read message. It persists until scrolled
  past and stays visible for the rest of the session.
- Rooms with new activity get a **weight change only** — label opacity goes from 60% to
  100%. No number, no dot, no color.
- A **"since you were gone"** view the user *pulls* from the room header. Never pushed.

**One exception:** direct mentions produce a real notification and a marker. Mentions
are person-to-person only. `@everyone` and `@here` do not exist and will not be added.

**Person-centric notifications.** Users may set "always notify me when [person] posts",
per person, per room. This is the notification setting people actually want, and no
keyword-based system delivers it.

Expect pushback on removing counts from people conditioned by ten years of them. Hold
the line for at least one month of real use before revisiting.

### 4.3 Presence and activity

Detection implementation is in `ARCHITECTURE.md` §6. Product rules:

**Default deny.** The client detects the foreground process, resolves it against a
bundled app registry, and reports only a resolved app identity. An unrecognized process
reports nothing at all.

**Never report window titles.** Not to the server, not to other clients, not in logs.
This kills the entire class of leak where `Re: settlement offer — Gmail` shows up in
someone's friend list.

**Browsers report as the browser.** "Firefox", never the site or tab title. Site-level
detection requires a browser extension and is out of scope, permanently.

Controls, all client-side (the server never receives what it is not allowed to show):
- Global off switch, one click from the roster
- Per-server off switch
- Per-app hide list ("never show that I'm in X")
- Idle-only mode (share presence but not activity)
- A persistent visible indicator whenever activity sharing is on

Presence states: `in_room`, `around` (app focused, no room), `idle` (no
input >10 min), `away` (explicit, with message), `offline`.

### 4.4 Media

Everything shared on a server accumulates into a browsable collection: images, video,
audio, links, files, and pinned messages.

- Grid view, filterable by person, type, and date range.
- Each item links back to the message and moment it was posted in.
- Anyone can star an item; starred items sort first and never expire.
- First-class sidebar destination, not a search result.

This fixes the deepest wound: you shared something great eight months ago and it is
gone. For a friend group, that collection *is* the relationship.

### 4.5 Name styling (the AIM feature)

AIM had no profile pictures. It had decorated names, and people put real effort into
them. Bring that back with modern typography.

Each user controls the rendering of their own display name:

| Property | Options |
|---|---|
| Font | One of ~12 curated bundled faces (§5) |
| Weight | 400 / 500 / 700 |
| Style | roman / italic |
| Fill | one of the 16 named palette colors, or a gradient of any two of them |
| Effect | none / slow shimmer / soft glow |

**Implementation constraints, non-negotiable:**

1. **Colors come from the named 16-color palette (§5.4), never from a picker.** The wire
   format is a palette key (`"azure"`), not a color value. Contrast safety is structural
   — the palette is defined once with theme-mirrored lightness, so there is nothing to
   clamp at runtime and no way to produce an unreadable name.
2. **No arbitrary fonts.** Curated bundled set only. Arbitrary font URLs are a
   fingerprinting vector and a remote-load dependency.
3. **Gradient angle is fixed at 92°**, not user-configurable.
4. **Shimmer and glow respect `prefers-reduced-motion`** and are disabled entirely in
   compact and IRC density modes.
5. **A global "normalize everyone" toggle** renders all names in the reader's default
   style. Some people will want this. Give it to them without friction.

**Message body styling** gets a much lighter touch, deliberately. The AIM era's
hot-pink-Comic-Sans-on-black was funny for a week and unreadable forever. Users get:
- Optional font override from the same curated set
- Nothing else. No message-text color, no per-message formatting, no size changes, no
  backgrounds. The name carries the identity; the body stays legible.

"Normalize everyone" flattens message styling too.

### 4.6 Statuses and away messages

The AIM away message is the most-missed feature of that era — a status, a mood board,
and a joke delivery mechanism in one field.

A user's **status** is a small card, not a bio field:
- One line of free text (240 chars, rendered in their name styling)
- Optional: reading / listening to / working on (three labeled short fields)
- Optional: one image, max 512 KB, displayed at 400×200
- Optional: an away message that supersedes the status when set

Statuses appear in the roster card when expanded, and in the user popover.

### 4.7 Text presentation

Discord is simultaneously low-density and visually noisy. Fix both.

**Aging.** Message opacity steps down with age: <1h at 100%, <1d at 88%, <1w at 76%,
older at 66%. Scrolling up feels like walking into the past. One CSS custom property
computed from the timestamp.

**Sessions, not a stream.** Group consecutive messages by the same author; break the
group on a 10-minute gap. Break the *session* on a 3-hour gap, which inserts real
whitespace and a soft divider labeled in natural language: `late Tuesday night`,
`Saturday morning`, `yesterday afternoon`.

**No avatar column.** A 3px colored gutter rule per person, name in the group header.
Density up, noise down.

**Restrained embeds.** A link renders as a one-line inline card: favicon, title, domain.
Not a 400px billboard. Images render inline at true aspect ratio, capped at 400px
height, click to expand.

**Density modes.** `Comfortable` / `Compact` / `IRC`. IRC mode is genuinely tight: one
line per message, timestamps in a fixed-width gutter, no grouping, no aging, no effects.
A real first-class option, not a joke.

**Time-of-day warmth.** Background and text colors shift ~200K warmer after local
sunset. Subtle enough that most people never consciously notice. User-disableable.

### 4.8 Reactions

Fixed palette of 12. No custom emoji in V1, no emoji picker.

Reactions accumulate **visibly by weight, not by number**. Six people hitting the same
reaction produces a denser, larger mark, not `👍 6`. Hover reveals who.

Rationale: numbers invite comparison. Weight carries the same information without
inviting anyone to count.

### 4.9 Knock

A lightweight nudge to a specific person that explicitly does not demand a reply. A soft
sound and a transient card on their screen — no message, no thread, no unread state,
nothing to dismiss or respond to.

Low-obligation contact is the thing group chats are worst at.

Rate limit: 3 knocks per person per hour.

### 4.10 File sharing

- 500 MB per file
- 50 GB per server pool (host-configurable)
- Non-starred, non-pinned files expire after 365 days (host-configurable, can be off)
- Resumable uploads
- **EXIF stripped from all images on upload, always, no toggle.** Camera photos carry
  GPS coordinates; silently sharing your home address in a privacy-focused app would be
  an embarrassing bug.
- Video: server generates a poster frame and a blurhash. No transcoding in V1.

### 4.11 Export

Any member can export the entire server at any time: messages as markdown, media as
files, in one archive. No gatekeeping, no host approval, one export per hour.

This is a trust feature and an anti-lock-in guarantee. It costs a weekend and it is the
most credible thing in the product.

---

## 5. Design system — "Console"

The aesthetic is **a refined instrument, not a terminal and not a document.** It takes
mIRC's *structure* (hairline panel separation, density, the aligned nick column, the
live status bar) and AIM's *personalization*, and renders both with modern craft.

Reference class: Zed, Bitwig Studio, Raycast, well-made audio software. Braun and Swiss
typographic discipline underneath.

**Explicitly not:** terminal cosplay (green phosphor, everything monospace, 11px
everything), neon gamer, enterprise Slack-gray, warm paper/newspaper, or a dark-mode
Discord reskin.

### 5.1 Hard visual rules

| Rule | Reason |
|---|---|
| **No chat bubbles.** Messages are text on the surface. | Bubbles are the iMessage/Discord signature |
| **No avatars.** Identity is carried by styled names and color. | The whole personalization thesis |
| **No shadows** except 1px focus rings. | Shadows imply floating cards; panels don't float |
| **No gradients on surfaces.** Gradients exist only in user name fills. | Keeps the one expressive element expressive |
| **No rounded panels.** Radius: 4px controls, 6px inline media, **0 on panels.** | Panels butt against each other via hairlines |
| **No colored icon squares in the rail.** | Discord's signature; instant clone read |
| Panel separation is **1px hairlines, full-bleed.** | The mIRC structural inheritance |

### 5.2 Typography

| Role | Face | Size | Notes |
|---|---|---|---|
| Message body | Geist Sans (or IBM Plex Sans) | 13.5px / 1.6 | **Sans, not mono** |
| UI labels, room names | same | 12–14px / 500 | |
| Timestamps, status bar, all numerals, file sizes, code | Geist Mono (or JetBrains Mono) | 11–12px | **Mono is metadata-only** |
| Section dividers | mono, uppercase, `0.1em` tracking | 11px | `SATURDAY MORNING` |

Mono appearing in a message body is a defect. That was the failure mode of the earlier
direction.

### 5.3 Color

Cool neutral gray. **Not green-shifted, not blue-black, not warm.**

```
                    DARK (primary)      LIGHT
surface-0  app bg   #16181C             #F7F8F9
surface-1  rails    #1A1D22             #FFFFFF
surface-2  raised   #21252B             #FFFFFF
hairline            #2A2E35             #E3E5E9
hairline-strong     #363B44             #CFD3DA
text-primary        #E4E7EC             #16181C
text-secondary      #8B929E             #5C636F
text-muted          #5C636F             #8B929E
text-faint          #4A515C             #A8AEB8
accent              #6E9BFF             #2563C9
```

Accent is used for exactly four things: the "you left off here" line, focus rings, the
active-room rule, and the send affordance. Nowhere else.

### 5.4 The 16-color name palette

**This replaces free color picking.** Users choose a *named* color, not a hex or a wheel.

Derived from the IRC 16, remapped through OKLCH. Each color is one hue with a
theme-mirrored lightness, so contrast is guaranteed by construction and no runtime
clamping is needed.

```
ember 32   rust 50    amber 68   brass 90
lime 118   fern 145   mint 162   teal 180
cyan 200   sky 230    azure 255  indigo 275
violet 295 orchid 320 rose 350   slate 250*
```
*(numbers are OKLCH hue; slate uses chroma 0.02, the rest 0.13)*

```
dark theme:   oklch(0.76 0.13 <hue>)
light theme:  oklch(0.50 0.14 <hue>)
```

Build generates hex fallbacks from these. **Verify `oklch()` renders in the target
WebKitGTK version during M0** — if it doesn't, ship the generated hex and drop the
runtime function.

Why this is better than a color wheel:
- Contrast safety is structural, not enforced by a clamp nobody can see working
- Two clicks instead of a fiddly picker
- Instantly familiar to anyone who ever typed `Ctrl-K` in mIRC
- 16 distinct colors across ~15 people means near-unique identity per person

**Gradients** are "pick two of the sixteen." Angle is fixed at 92°, not user-configurable
— one less control, and it guarantees every gradient name reads consistently.

### 5.5 Layout metrics

```
rail (servers + rooms)     200px
roster (who's around)      240px
message stream             flex, min 420px
panel gutters              14–16px
gap between message groups 14px
hairlines                  1px
radius                     4px controls / 6px media / 0 panels
```

### 5.6 Behavior

- **Message aging** applies to the message *body only* — never the name or timestamp.
  Steps: <1h 100%, <1d 88%, older 78%. Floor at 78%; do not go lower.
- **System messages** (joins, leaves, pins, "dave stood up") are hairline rules with
  centered mono small-caps text. Never chat lines.
- **Status bar** is permanent, mono, 11px: connection state, latency in ms, storage used,
  and `sharing: <app>` whenever activity sharing is on. This is the cheapest "real tool"
  signal available and no competitor has it.
- **Connection states show protocol text, not spinners:**
  `connecting… tls ok… identify… ready (28ms)`
- **Motion:** 120–160ms, ease-out. No spring, no bounce. The only slow animation in the
  app is name shimmer (4s linear).
- **Density modes:** Comfortable (13.5px/1.6) · Compact (13px/1.45) · IRC (12.5px mono
  body, one line per message, no grouping, no aging, no effects).

### 5.7 Bundled fonts

Open-licensed, subset and bundled, no CDN:
Geist Sans, Geist Mono, IBM Plex Sans, IBM Plex Mono, JetBrains Mono, Inter, Space
Grotesk, Commit Mono, Newsreader, Instrument Serif, Departure Mono, Silkscreen.

The last three are deliberately characterful — people will want them for names. The
first two are the system defaults.

---

## 6. Scope

### V1 — "it replaces the text half of Discord for one friend group"

| # | Feature | Spec |
|---|---|---|
| 1 | Self-hosted server: single binary + Docker image | ARCHITECTURE |
| 2 | Invite-link registration; host/member roles only | §2 |
| 3 | Rooms with occupancy and in-room presence | §4.1 |
| 4 | Entrance sounds | §4.1 |
| 5 | Text: markdown, edit, delete, reply | §4.7 |
| 6 | No unread counts; "left off here" line | §4.2 |
| 7 | Roster-forward layout | §3 |
| 8 | Activity detection, default off, app-registry only | §4.3 |
| 9 | Name styling + message accent color | §4.5 |
| 10 | Statuses and away messages | §4.6 |
| 11 | Reactions by weight | §4.8 |
| 12 | File upload 500 MB, EXIF stripped | §4.10 |
| 13 | Media collection | §4.4 |
| 14 | Density modes incl. IRC | §4.7 |
| 15 | Full export | §4.11 |
| 16 | Desktop client: Linux, Windows, macOS | ARCHITECTURE |
| 17 | Multi-server list in the client | §3 |

### V2

- Voice rooms (WebRTC mesh + coturn), push-to-talk, per-user gain
- Ambient voice: a room you leave running, not a call you join
- DMs and group DMs
- Search
- Knock (§4.9)
- Local AI features and the agent surface — see §8
- Mobile client

### V3 or never

- Opt-in public directory (must never be load-bearing)
- Sandboxed client scripting (the real mIRC nostalgia answer; also a real security
  surface)
- Custom emoji

---

## 7. The honesty section

Read this before building and again at month three.

**Novelty features do not retain people.** Everything in §4 gets tried once. What keeps
a friend group on this app is that it opens in 400ms, does not nag them, and does not
lose their photos. The differentiators buy the first week. The absence of friction buys
the next two years.

**The scope discipline is the product.** The moment a role editor appears, the rebuild
of the thing you left has begun. When someone requests a feature, the question is not
"is this good?" but "does this belong at a dinner party?"

**The hardest parts are not chat.** They are cross-platform activity detection and
code-signed multi-platform distribution. Both are in `ARCHITECTURE.md`. Neither is
interesting. Both will consume more time than the entire message pipeline.

**Do not claim end-to-end encryption.** The threat model is stated plainly in
`ARCHITECTURE.md` §7 and must be stated plainly to users too. Half-implemented E2EE is
worse than none, because it launders a false promise.

**Prior art to read before writing code.** Stoat (formerly Revolt, AGPL-3.0, GitHub org
`stoatchat`) is the closest existing project — a self-hostable Discord-alike with a Rust
backend. Read their schema and gateway protocol before designing yours. What they do not
have is the presence model, the AIM-style personalization, or a Tauri-weight client;
that is the differentiation. Do not fork them, but do not re-derive solved problems.

---

## 8. AI — where it belongs and where it does not (V2)

The position: **no AI in the conversation. Yes to local AI features and an
agent-accessible architecture.**

### 8.1 Why not in the conversation

The thesis is presence over messages and remove obligation. An AI participant attacks
both. The entire value of a friend replying is that a person chose to spend attention on
you; introduce a thing that also replies and every reply gets cheaper, because you can no
longer tell what was chosen. Suggested replies are worse — they turn a friendship into
autocomplete.

**Permanent anti-goals. Add to §2:**

- AI participants that post in rooms
- Suggested replies, message drafting, or autocomplete on message content
- Sentiment, tone, or mood analysis of any kind
- Any network call containing message content to an endpoint the host did not configure
- Any AI feature that is on by default

### 8.2 Where it earns a place — all local, all opt-in

| Feature | Notes |
|---|---|
| **Semantic search over history and media** | Highest-value item. "that video of the drive" actually finds it. Local embeddings, stored alongside the SQLite DB. |
| **Catch-up summaries** | "Since you were gone" (§4.2), summarized. Pulled, never pushed. |
| **App registry auto-classification** | Classify unknown processes locally so the 200-entry registry isn't hand-maintained forever. Touches zero conversation content. |
| **Transcription and alt-text** | Accessibility. Local Whisper for voice memos, local vision model for image alt-text. |

**Hard constraint:** every feature above runs against a host-configured local endpoint
(Ollama-compatible). There is no cloud default and no cloud fallback. If no endpoint is
configured, these features do not appear in the UI at all.

The strategic point, worth stating in the README: a self-hosted Linger server can run
all of this on the box. A hosted competitor structurally cannot — their version
requires shipping your friends' conversations to a third party.

### 8.3 The agent surface

"Agent-friendly" mostly means "has a clean, documented, authenticated API," which V1
already produces. The delta is small and worth taking.

1. **`linger-mcp`: a separate optional binary** wrapping the existing REST API as an MCP
   server. Roughly 300 lines once M1 is done. **Ships disabled.** The host must enable it,
   and enabling it is a config change, not a UI toggle.

2. **Delegated capability tokens, not bot accounts.** A user issues a token to their *own*
   agent with an explicit scope and expiry:
   ```
   scope:  read:rooms[#shop]  write:none  expires:24h
   ```
   Revocable by the issuer at any time, listed in their settings, and never a separate
   identity in the member list.

3. **Everything an agent does is attributed to the delegating human and visibly marked.**
   `matt (via agent)`, always, in a visually distinct style that cannot be turned off.
   **An agent must never be renderable as indistinguishable from a person.** This is the
   rule that makes the rest of it safe. It is a hard rule in `AGENTS.md`.

4. Agent actions are rate-limited more aggressively than human actions, and an agent
   cannot issue tokens, create invites, or change another user's settings — ever.
