# Running a Linger server

This is for the one person in a group who runs the server. You do not need to be
a developer. You need a computer that stays on, a domain name, and about half an
hour.

If you are here to *use* Linger, you want the [user guide](user-guide.md)
instead.

---

## What you are actually running

Two small programs in containers: Linger itself, and Caddy, which handles the
web address and the security certificate for you.

Everything the server owns lives in one folder: a database file with all the
messages, and a folder of uploaded files next to it. Copy that folder and you
have copied the whole server.

## What you need first

- **A computer that stays on and is reachable from the internet.** A cheap
  virtual server (about $5–10 a month) is the usual answer. An old machine at
  home works if you can forward ports 80 and 443 to it.
- **A domain name.** Any registrar. This is the address your friends type.
- **Docker.** Install it from [docker.com](https://docs.docker.com/engine/install/).

---

## 1. Point your domain at the machine

Add **two** records at your domain registrar. Both point at your server's IP
address.

| Type | Name                     | Points to      |
|------|--------------------------|----------------|
| A    | `linger.example.com`     | your server IP |
| A    | `cdn.linger.example.com` | your server IP |

(Use `AAAA` instead of `A` if your server has an IPv6 address.)

**Why two.** Uploaded files are served from the `cdn.` name, never from the main
one. That way a file somebody uploads can never pretend to be the app itself. If
you skip the second record, chat works and file uploads break.

DNS changes can take a few minutes to spread. Get this done first and it will be
ready by the time you need it.

## 2. Get the two setup files

```bash
mkdir linger && cd linger
curl -O https://raw.githubusercontent.com/matthewguenther/Linger/main/deploy/compose.yaml
curl -O https://raw.githubusercontent.com/matthewguenther/Linger/main/deploy/Caddyfile
```

## 3. Put your domain in both of them

- In **compose.yaml**, change `LINGER_DOMAIN` to your domain.
- In **Caddyfile**, replace `linger.example.com` everywhere it appears. There
  are two blocks and both need it — the second one is the `cdn.` name.

Nothing else has to change to get started.

## 4. Start it

```bash
docker compose up -d
docker compose logs linger
```

In the log you will see a box like this:

```
  ┌─────────────────────────────────────────────────
  │  This server isn't set up yet.
  │  Open:  https://linger.example.com/setup?token=…
  │  (the link works once, then never again)
  └─────────────────────────────────────────────────
```

**Copy that whole link. It goes into the Linger app, not into a web browser** —
there is no website to visit, and a browser will just show an error.

## 5. Make your account

[Install the app](user-guide.md#installing-linger), open it, and paste the setup
link into the box that says *server or link*. Pick a username and a password
(eight characters or more, no silly rules).

That account is the host. There is no separate admin login, and there are no
roles — you are a normal member who also has the host controls.

## 6. Invite people

In the app, open the host controls from the small control on the left rail, then
**invites → make a link**. You choose how many times it can be used and when it
expires. Send the link however you normally talk to your friends.

An invite link is the only way to get an account. There is no public sign-up.

---

## Running it day to day

Almost everything is done inside the app, not in a config file.

- **Rooms** — host controls → *rooms*. Create, rename, set a topic, reorder,
  archive.
- **The server's name and accent color** — host controls → *server*. The name is
  what the rail shows and what an invite link tells a stranger.
- **Removing someone** — open their card in the roster and remove them. They
  disappear from everywhere.
- **Letting them back in** — host controls → *people*. Removals are reversible;
  that is the point.

Two things worth knowing. There is **no way to hand the host role to somebody
else**, on purpose. And there are no permissions to configure — if a group needs
that, it has outgrown what this app is for.

---

## Settings you might want to change

These go in `compose.yaml`, under `environment:`. Most people never touch them.
After a change, run `docker compose up -d` again.

| Setting | What it does | Default |
|---|---|---|
| `LINGER_POOL_BYTES` | Total storage the server will use. Write `250GB`, `500MB`, or a plain number. | `50GB` |
| `LINGER_FILE_EXPIRY_DAYS` | How long a file stays before it is deleted. `off` keeps everything forever. Starred files never expire. | `365` |
| `LINGER_MEDIA_DOMAIN` | The name files are served from, if you want something other than `cdn.` + your domain. It must be a different name from the main one. | `cdn.<your domain>` |
| `LINGER_STORAGE` | `local` keeps files on the machine. `s3` keeps them in a cloud bucket. | `local` |
| `LINGER_DATA_DIR` | Where the database and files live inside the container. | `/data` |

One file can be up to 500 MB.

**Using a cloud bucket instead of the machine's disk.** Set `LINGER_STORAGE: s3`
and fill in the five `LINGER_S3_*` lines already written in `compose.yaml` as
comments. Cloudflare R2 is the one to pick, because it does not charge for data
going out. The server refuses to start if any of them are missing, so you will
know straight away.

---

## Backups

The whole server is the `data` folder next to your `compose.yaml`. It holds
`linger.db` (every message) and `objects/` (every uploaded file).

Copy it while the server is stopped, so you never catch the database mid-write:

```bash
docker compose stop linger
tar czf linger-backup-$(date +%F).tar.gz data
docker compose start linger
```

That is a few seconds of downtime. Put it in a scheduled job and keep the copies
somewhere that is not this machine.

To restore: stop everything, put the `data` folder back, start again.

(If you moved files to a cloud bucket, `data/objects/` is empty and the bucket
is the other half of your backup.)

## Updating the server

```bash
docker compose pull
docker compose up -d
```

Nothing updates itself. You decide when.

## Somebody forgot their password

The server has one maintenance command. Stop it first — the database allows one
writer at a time.

```bash
docker compose stop linger
docker compose run --rm linger reset-password their-username
docker compose start linger
```

It prints a new password. Send it to them; they can change it in the app under
*settings → password*.

---

## When something is wrong

**Start here:** `docker compose logs linger` and `docker compose logs caddy`.

- **The address does not load at all.** Usually DNS has not caught up, or ports
  80 and 443 are not reaching the machine. Caddy's log will say if it could not
  get a certificate.
- **Chat works but uploads fail.** The `cdn.` record is missing, or the second
  block of the Caddyfile still says `linger.example.com`.
- **`docker compose pull` says `unauthorized`.** The prebuilt image is not
  available to you. Clone the repository and build it yourself:
  `docker build -f deploy/Dockerfile -t ghcr.io/matthewguenther/linger:latest .`
- **The setup link does not work.** It works once. If you already made an
  account, it is gone for good — that is deliberate.

---

## What you are taking on

Say this out loud to the people you invite, because it is true:

> **Whoever runs the server can read everything on it.** Messages and files are
> encrypted while they travel and sit on an encrypted disk if you set one up,
> but there is no end-to-end encryption. Your friends are trusting you, not the
> software.

What you are *not* taking on: there is no telemetry, no analytics, and no crash
reporting anywhere in Linger. Nothing on your machine phones home, to Matt or to
anyone else. Nobody is counting your users.
