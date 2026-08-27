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


---

## Decided — v1 ships unsigned, and macOS does not ship at all yet

**Matt, 2026-08-27.** M7 assumed installers would be code-signed on Windows and
notarized on macOS. Both turn out to be blocked on money rather than on work: a
Windows OV certificate is a few hundred dollars a year, and notarization needs an
Apple Developer Program membership at $99 a year. For a server you hand to a
friend group, neither is obviously worth buying yet.

**A correction that shaped this:** notarizing does not need a Mac in the room.
`release.yml`'s `macos-latest` runner is a Mac and can sign and notarize on its
own. The membership is the blocker. Somebody's Mac is still wanted for *opening*
the installer once — that is the T-002/T-003 sign-off, a different errand.

**So, for v0.1.0:**

- **Linux and Windows ship, unsigned.** Windows shows SmartScreen's
  "unrecognized app" warning, which is clickable past via *More info → Run
  anyway*. The README says so plainly rather than letting a friend discover it
  and assume the download is broken.
- **macOS does not ship.** Not even unsigned. Two reasons, and the second is the
  real one. Gatekeeper on recent macOS no longer takes a right-click → Open; it
  sends the user into System Settings to click past a malware warning. And the
  eventual move from an ad-hoc signature to a Developer ID one is precisely the
  transition that breaks an app the updater has replaced in place — so an
  unsigned macOS build now is not a step towards a signed one later, it is a
  thing that has to be uninstalled by hand. **Shipping nothing orphans nobody.**

**None of this is expensive to reverse.** The minisign updater key is
OS-independent and does not change. Adding a `darwin-aarch64` entry to a later
`latest.json` is a pure addition. With no macOS installs in the world, there is
nobody to strand.

**What this does to the queue:** T-702 keeps the half that is not blocked — the
release CSP hardening, and documenting the warning honestly. The signing itself
becomes **T-705**, parked until there is a certificate and an account. Note that
M7's milestone check says *a signed installer per OS*, so M7 closes on two
operating systems and unsigned; that is the decision, not an oversight.
