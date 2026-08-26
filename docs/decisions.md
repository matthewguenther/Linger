# Decisions — settled questions and why

Moved from `TASKS.md` on 2026-08-25. These are settled. Reopening one is
Matt's call, not a maintenance decision. Tasks that lean on a decision here
point at this file.

## Decided — the password stays, the friction goes

**Matt, 2026-08-21.** The question was whether a locally-run server needs a
password at all, since Ventrilo asked for a name and nothing else. Answer: keep
it, but stop making people think about it.

The password is not protecting the messages — it is protecting *being you*. The
roster is the product, and the documented deployment is a box on the open
internet (ARCHITECTURE §7), so name-only would mean anyone who ever received an
invite link can connect as anybody. What was actually annoying was the
**12-character floor**, which is friction paid on every fresh install and buys
very little when the client already keeps the password in the OS keyring.

**Done in the same pass:** the floor is now **8**, which with no composition
rules is the NIST SP 800-63B position. One constant,
`linger-core::limits::MIN_PASSWORD_CHARS`; the server's error message counts off
it instead of spelling the number out, and the two client forms mirror it the
same way `Stream.tsx` mirrors `MAX_MESSAGE_CHARS`. PROTOCOL §2 says 8 and says
why.

The two bigger options were considered and **not** taken: making the invite link
itself the credential (a real change to PROTOCOL §2 and the refresh-token family
logic), and a host-set no-auth LAN mode (honest for a LAN party, dangerous the
day the box gets a public IP). If the friction comes back, those are the next
two rungs, in that order.

---

## Decided — the host's side

**Matt, 2026-08-21.** Four questions came out of reading T-410 before starting it.
The server-admin story is otherwise unchanged and already built: the host deploys
the image, reads the one-time setup URL out of the logs, pastes it into the
client, and from then on is a normal user with `is_host = 1`. Rooms and server
settings are database rows edited from inside the app — never a config file.

**No host transfer.** `is_host` stays a boolean that nobody can hand to anybody
else. If the host goes quiet, the friends stand up a new server. That is a real
answer for a group of eight, and it keeps a second root from growing on a product
whose anti-goals include a permission matrix (AGENTS rule 10).

**Removal, not banning** — T-413. A ban needs something durable to ban *by*, an
address or a device id, and Linger does not store either and should not. It would
not work anyway: housemates share one address and phone networks reshuffle them.
A removal here is already ban-shaped — usernames are unique and immutable, the
account row stays, and registration is invite-only, so the host is the only door
back in. One action, plus the reverse, so a removal made in a bad moment is
fixable.

**The M5 storage knobs are environment variables.** The 50 GB pool and the
365-day file expiry (SPEC §7) go in the compose file with everything else, per
the position `config.rs` already takes. No config-file format to document,
version, and migrate. Built that way in T-505: `LINGER_POOL_BYTES` (a plain
byte count or a size like `250GB`) and `LINGER_FILE_EXPIRY_DAYS` (a number of
days, or `off`). T-501 had parked the pool in a `server_config` row against a
host-facing endpoint that this decision says will not exist, so that row is
gone — one source of truth, and it is the environment.

**The printed setup URL is https when a domain is set** — fixed 2026-08-21, not a
task. The client keeps whatever scheme it is handed, for the REST base URL and
the gateway socket both, so printing `http://` pinned the host's own session to
plaintext on the very first thing they ever do. It falls back to `http` only for
a bare bind address, which has no certificate and is honestly plaintext.

