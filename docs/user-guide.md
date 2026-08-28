# Using Linger

Linger is a small, private place for a group of friends to hang out. Somebody
you know runs the server; you install the app and connect to it.

If you are the one running the server, you want the
[host guide](host-guide.md) instead.

---

## Installing Linger

Go to the [Releases page](https://github.com/matthewguenther/Linger/releases)
and download the file for your computer.

- **Windows** — the `.exe` installer.
- **Linux** — the `.AppImage` (works on any distribution; make it executable and
  run it) or the `.deb` if you are on Debian or Ubuntu.
- **macOS** — not built yet. You can build it from the source if you are
  comfortable doing that; otherwise, sorry, not yet.

**Windows will show you a warning.** It says *"Windows protected your PC"*, and
the *Run anyway* button is hidden behind the small *More info* link. That is
Windows saying the installer has not been through a paid signing process. It has
not. Nothing about the download is broken. Click *More info*, then *Run anyway*.

## Getting in

Open the app. There is one box, and it takes any of these:

- an **invite link** somebody sent you — this is the usual one
- the **address** of a server you already have an account on, like
  `linger.example.com`
- a **setup link**, if you are the person who just started the server

Paste it, then pick a username and a password (eight characters or more), or
sign in if you already have an account.

The app remembers you. On most computers your sign-in is kept in the system's
password store — the same place your browser keeps passwords — so you do not
type it again. If your computer has no password store, the app says so and asks
you to sign in next time.

You can be on **more than one server**. They stack up in the far-left rail and
each one is completely separate: separate account, separate friends, separate
everything.

---

## The window

Three parts.

- **The rail**, on the left: your servers at the top, then the rooms on this
  server. A room shows who is in it.
- **The stream**, in the middle: the conversation, and the box you type in.
- **The roster**, on the right: everyone on the server and what they are up to.
  On a narrow window it moves to a strip above the box you type in.

## Saying things

Type and press **Enter**. **Shift+Enter** starts a new line instead of sending.

A little formatting works, the kind you already type:

```
**bold**    *italic*    ~~crossed out~~    `code`
> a quote
- a list
```

Paste a link and it becomes a link. Type `@` and someone's username to mention
them — that is the one thing that will interrupt them.

Hover a message for its buttons:

- **react** — twelve reactions, the same twelve for everybody. No custom emoji.
- **reply** — quotes what you are answering. **Escape** cancels it.
- **edit** — your own messages only. **Shortcut: press Up arrow in an empty box**
  to edit the last thing you said.
- **delete** — asks once, then it is gone

## Sharing files

Three ways, all the same thing: the **+ file** button, drag a file onto the box,
or paste one from your clipboard.

- Up to 500 MB per file.
- **Location data is stripped from every photo, always.** Phone cameras record
  where a picture was taken, and Linger removes that before anyone else sees it.
  There is no setting for this and no way to turn it off.
- Files may be deleted after a while — a year, unless whoever runs the server
  chose differently. **Starring a file keeps it forever.**

## Finding things again

Open **media** from the rail. It is everything ever shared on the server —
pictures, video, audio, files and links — newest first, filterable by type and
by person. Every item links back to the moment it was posted.

The star does two jobs: it sorts things to the top, and it stops a file from
ever being cleaned up. Star the good ones.

## What the roster is telling you

Next to each person:

- **in a room** — they are in that room right now
- **around** — the app is in front of them, but not in a room
- **idle** — no typing or clicking for ten minutes
- **away** — they set an away message on purpose
- **offline** — the app is closed

Click anybody's name to see their status card.

## Your status

Click your own name in the roster, then *edit*. There is a line in your own
words, plus three optional fields: *reading*, *listening to*, *working on*. You
can put one image on it.

There is also an **away message**. Setting one is what makes you away, and it
shows instead of your status. Clearing it brings you back.

## Making your name yours

**Settings → how your name looks.** This is the fun part, and it is what everyone
else sees next to everything you write.

- a **face** (one of twelve fonts) and a **weight**, plus italic
- a **color**, or two colors blended, from a fixed set of sixteen
- an **effect** — a shimmer or a glow, or nothing
- optionally, a **font for your messages** too

The sixteen colors are the same for everybody, and every one of them is readable
on every background. You cannot pick something nobody can read.

## Making it comfortable to read

Also in settings:

- **Density** — how tightly the stream is packed. *Comfortable* is the default;
  *IRC* is one line per message.
- **Theme** — dark, light, or follow your desktop.
- **Evening warmth** — after about 7pm the colors go slightly warmer, the way a
  room does when the lamps come on. It is subtle. You can switch it off.
- **Normalize everyone** — turns off other people's name styling and message
  fonts, for you only. Nobody is told. Use it if a room is too loud to read.

## Being interrupted, or not

The only thing that raises a notification is **somebody naming you** — or a
person you have specifically asked to hear about. The **notify** button at the
top of the roster is where you tick those people, either everywhere or in
chosen rooms.

There are **no unread badges and no counters** anywhere in Linger. That is
deliberate. Nothing is keeping score of what you have not read, so nothing can
make you feel behind. When you come back, the stream shows a **go to where you
left off** marker; use it or ignore it.

## Taking everything with you

Any member can ask the server for a copy of **everything on it** — every
message, every file — at any time. You do not need the host's permission, and
there is nothing to ask for.

What you get is a single zip. Inside it: one plain text file per room in the
order things were said, a `media` folder with every file anybody shared, and an
index listing who shared what and when. It opens with an ordinary text editor
and an ordinary file browser. **You do not need Linger, or an account, or the
server to still exist**, which is the entire point.

**Settings → take everything with you.** Press the button, wait — it takes a
moment on a busy server — then press *download it*. The file opens in your
normal browser's downloads, like anything else you download.

You can ask for one an hour. If you ask again too soon the app tells you when
you can come back.

## Updates

The app checks for a new version when it starts and when you open settings. If
there is one, you get one quiet line in the status bar. Nothing downloads or
installs until you click it in **settings → updates**. It will never restart
itself in the middle of a sentence.

## Signing out

**Settings → this computer → sign out.** That forgets the server on this
computer. Your account and everything in it stays exactly where it is.

To change your password, use **settings → password**. If you have forgotten it,
ask whoever runs the server — they can set you a new one.

---

## Worth knowing

**Whoever runs the server can read everything on it.** There is no end-to-end
encryption in Linger, and it does not claim any. Messages are encrypted while
they travel across the internet, and they sit in a database on someone's
machine. That person is your friend, which is the whole idea — but it is a
different promise from Signal, and you should know which one you are getting.

**Nothing is collected about you.** No telemetry, no analytics, no crash
reports. Not anonymous ones either. There is nothing to opt out of.

**There is no AI in Linger.** No suggested replies, no summaries, no assistant
in the room. A reply means a person chose to spend a minute on you, and that is
the point.
