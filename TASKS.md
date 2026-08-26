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
| **treacherous** | The strongest model and highest reasoning setting available to you, and coordinate with Matt before claiming. Currently T-701+T-702 (signing/notarization) and, on the backburner, T-911 (Wayland/KWin). AGENTS.md §"Where you will be wrong" territory. |

Running everything at maximum is not better — it is slower, pricier, and prone
to overbuilding simple tasks. Match the effort to the label and escalate only if
a task fails its acceptance criteria twice.

---

## Status

Closed milestones are archived in `docs/tasks/` with every landing note and
surprise intact. Tasks T-001…T-415 live there. Decisions that shaped the queue
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

**Still open from closed milestones:** the visual "a window opens" sign-off on
Linux, Windows and macOS (T-002/T-003 in `docs/tasks/m0.md`) is a human errand,
outstanding, and must be closed before M7.

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

---


## M5 — uploads, media pipeline, the media grid

*Milestone check: a 400 MB video uploads, resumes after a killed connection, appears in the media grid.*

- ✅ **T-501 · Upload pipeline (local backend)** — effort: **high** — landed 2026-08-25
  ARCHITECTURE §8 + PROTOCOL §6. Slot creation validates size/quota/MIME
  allowlist; token-authenticated direct-PUT URLs (bytes never traverse app
  routes — separate upload listener path); multipart >8MB with per-part URLs
  (this is the resumability); complete: re-verify size, sniff real MIME,
  re-encode images (kills EXIF + polyglots in one step — `image` crate),
  blurhash, video poster via ffmpeg. Reject oversize at slot *and* at complete.
  *Accept:* the milestone check, scripted: kill mid-upload, resume, complete;
  EXIF-GPS test image comes out clean; fake-MIME file is caught.

  All three accept criteria are `crates/linger-server/tests/uploads.rs`
  (15 tests): `a_killed_upload_resumes_and_completes` sends a body that dies
  mid-part, proves complete refuses, resends and completes;
  `exif_never_survives_an_upload` builds a real JPEG with a hand-written EXIF
  APP1 + GPS IFD and asserts nothing of it is in the stored bytes;
  `a_file_that_lies_about_its_type_is_refused` sends zip bytes declared as PNG.

  **Decisions and surprises, for whoever picks up T-502…T-506:**

  - **An upload id is an attachment id.** Same UUID, two newtypes, no `uploads`
    table and no migration. Nothing about an in-flight upload needs storing that
    the attachment row does not already hold, and the part layout is a pure
    function of the declared size (`storage::part_plan`), so a resumed upload
    recomputes the plan rather than looking it up.
  - **Failing at complete is two different things**, and getting this wrong is
    what the first version did. Parts missing = the ordinary dropped connection:
    the slot stays pending, the client sends what is missing, complete again.
    Anything else (wrong size, a file that isn't the type it claimed) = final,
    parts discarded. Without that split, one flaky part burned the whole upload.
  - **A message with a file on it may have an empty body.** PROTOCOL §4 said
    1–8000 chars, which would have made "share a photo without a caption"
    impossible. PROTOCOL §4 is updated in the same commit; `validate::caption`
    is the version that allows empty and `validate::message_body` still doesn't.
  - **`ObjectStore` exists with one implementation** (`storage::LocalStore`), so
    T-502 has a target: slot, assemble-into-a-local-file, put/read/delete,
    discard. `assemble` returning a local path is deliberate — S3 will download
    the completed object there, because sniffing and re-encoding need the bytes.
  - **The listener is `PUT /upload/{id}/{part}` and serving is
    `GET /objects/{key}`**, both outside `/api/v1`, neither authenticated. Part
    URLs carry an HMAC over (upload id, part, expiry); the key is
    `data/upload_hmac.key` and must survive restarts or resume breaks. Serving
    is unauthenticated on purpose — the key holds a UUIDv7, the URL is the
    secret, and that is the only arrangement an `<img>` tag can use. What makes
    it safe is the headers, and **T-503 is what finishes it** by moving these
    responses off the app origin.
  - **WebP comes back as PNG.** The `image` crate reads WebP and cannot write
    it. Rather than skip re-encoding (and keep EXIF), a WebP is re-encoded to
    PNG and its filename extension corrected. GIFs are re-encoded frame by frame
    so animation survives. AVIF is not on the allowlist: decoding it needs a
    native dav1d build, which is not worth a system dependency yet.
  - **`server_config['pool_bytes']` already works** — `repo::attachments::
    pool_limit` reads it and falls back to the 50 GB default. T-505 needs the
    endpoint that sets it and the used figure on `GET /server`;
    `repo::attachments::pool_used` is the used figure, and it counts pending
    uploads so a full server cannot hand out fifty more slots.
    *(Superseded by T-505: the pool is `LINGER_POOL_BYTES` and there is no
    endpoint. The row and `pool_limit` are gone; `pool_used` stands.)*
  - **Abandoned uploads are swept on the way in**, not by a background task:
    slot creation deletes pending rows older than 48h and their part files. It
    is the only moment the answer matters (pending bytes count against the
    pool) and slot creation is 20/hour/person, so it is nowhere near hot.
  - **ffmpeg/ffprobe are optional and shelled out to.** No poster and no
    duration without them; nothing fails. `deploy/Dockerfile` installs ffmpeg
    now, so T-701's note about adding it is done. The video test skips when
    ffmpeg is absent, so **CI is currently not exercising the poster path** —
    add ffmpeg to the CI runner in T-701 and it starts running.
  - The scripted resume test moves 16 MB over three parts, not the milestone's
    400 MB over fifty. Same loop, fifty times; `storage::tests` pins the part
    arithmetic for 400 MB and 500 MB. **A real 400 MB video over a real network
    is still a human check** and belongs at the end of M5, with T-504's grid.
  - `LINGER_STORAGE=s3` now refuses to start rather than booting a server whose
    uploads cannot work. T-502 removes that.

- ✅ **T-502 · S3 storage adapter** — effort: **medium** — landed 2026-08-25
  Same `ObjectStore` trait, presigned URLs, `LINGER_STORAGE=s3` now boots.
  `crates/linger-server/tests/s3.rs` (7 tests) drives the public endpoints
  against a real MinIO; CI has an `s3` job that starts one, and
  `scripts/minio-test.sh` does the same locally.

  **Decisions and surprises, for whoever picks up T-503…T-506:**

  - **No S3 multipart upload, on purpose.** Each part is a presigned PUT to its
    own key, `uploads/{upload_id}/{part:05}`, and `assemble` streams them down
    into `data/staging/`. S3's own multipart would assemble the object inside
    the bucket, and the next thing the server does is download it anyway —
    sniffing and EXIF-stripping need the bytes locally — so the file would
    cross the wire three times. It would also hand out an upload id of S3's
    own, which would have to be stored: exactly the per-upload row T-501 got
    rid of. ARCHITECTURE §8 is updated to say this.
  - **A server on S3 still needs a data directory.** `linger.db` and the JWT
    key live there, and every upload passes through `data/staging/` on its way
    to the bucket. It is deleted immediately afterwards, whether the upload
    succeeded or was thrown away, but the disk has to be able to hold one file.
  - **`read_object` now takes a `ServeAs`.** The download-forcing headers
    (ARCHITECTURE §7) used to be set by the route, which works while the route
    is the thing sending bytes. With S3 it is a redirect, so the route works
    the two headers out and the store signs them into the presigned URL as
    `response-content-type` / `response-content-disposition`. The local backend
    ignores the argument. **`X-Content-Type-Options: nosniff` cannot be signed
    into an S3 URL** — S3 has no `response-` override for it — so on S3 that one
    header is missing until **T-503** puts these responses behind the CDN host,
    where the proxy can add it. Worth knowing before T-503 is called done.
  - **Serving is still `GET /objects/{key}` and then a 307.** The redirect keeps
    one URL shape in `Attachment.url` for both backends and keeps the row lookup
    that knows the filename and mime. It costs a round trip per image; if that
    ever matters, the fix is a public bucket domain, not a change to the client.
  - **`rusty-s3` + `reqwest`, not the AWS SDK.** `rusty-s3` only builds and signs
    URLs — no sockets, no runtime of its own — which is a much smaller thing to
    carry into a binary that already has an HTTP stack. Presigned URLs are the
    whole S3 API surface this needs.
  - **`reqwest` is a real dependency now**, not just a dev one, with
    `rustls-tls`. Integration tests can still use it: cargo gives test targets
    the regular dependencies as well as the dev ones.
  - **MinIO in CI is `docker run`, not a `services:` block.** A service container
    cannot pass a command, and the MinIO image needs `server /data` to start.
    The image tag is pinned.
  - **The S3 tests skip when `LINGER_TEST_S3_ENDPOINT` is unset**, printing a
    line saying so. That is what keeps `cargo test --workspace` green on a
    laptop — and it means a green workspace run proves nothing about this
    backend. Run `scripts/minio-test.sh` before claiming it works.
  - The test variables are `LINGER_TEST_S3_*`, deliberately not `LINGER_S3_*`.
    A test must not be able to write into a real bucket by inheriting the
    environment of the machine it runs on.
  - Still a human check, as with T-501: **a real 400 MB video into a real
    bucket over a real network.** The tests move 16 MB over three parts.
- ✅ **T-503 · Separate media origin** — effort: **medium** — landed 2026-08-25
  Uploads are served from `cdn.<LINGER_DOMAIN>` and nowhere else; the Caddyfile
  block is live; the app origin's CSP is tightened.
  `crates/linger-server/tests/media_origin.rs` (4 tests) drives real HTTP with
  the `Host` header a reverse proxy would set, and
  `an_object_carries_its_own_headers_in_the_bucket` in `tests/s3.rs` covers the
  S3 half against MinIO.

  **Decisions and surprises, for whoever picks up T-504…T-506:**

  - **The split is enforced by the server, not just advertised by DNS.** Both
    names reach one process through Caddy, so a `Host` check in
    `routes::media_origin_gate` decides what each serves: on the media host,
    `/objects/...` and nothing else; on every other name, everything *but*
    `/objects/...`. Without that, "separate origin" would have been a URL shape
    and no more — the app host would still have served every uploaded file, and
    a hostile file's own origin would have had the whole API on it.
  - **`LINGER_MEDIA_DOMAIN` defaults to `cdn.<LINGER_DOMAIN>`**, so the only
    setup cost is a second DNS record. Setting it equal to `LINGER_DOMAIN` is a
    startup error rather than a quiet downgrade. A server with no domain has one
    origin and no split; that is what every test server runs as, and it is why
    `spawn_server()` still hands out root-relative URLs while
    `spawn_named_server()` is the one that exercises this.
  - **The `nosniff` gap from T-502 is closed as far as S3 permits, and it is
    worth knowing exactly how far that is.** S3 has no `response-` override for
    `X-Content-Type-Options` or `Content-Security-Policy`, and proxying the
    bytes back through this process to add them would break the rule the S3
    backend exists to keep. So: both headers are now on every response the
    server sends itself, the Caddy media block sets them on everything it
    serves, and on the S3 path three things stand in for them — active content
    is not storable at all, anything off the inline list is
    `application/octet-stream` + `attachment` (which a browser downloads
    whatever it sniffs), and those two headers are stored **on the object** as
    well as signed into the URL. That last one is new here and is what makes a
    CDN-fronted or public bucket safe. A host who wants the literal header on
    the S3 path adds a response rule at their CDN; README says so.
  - **`put_object`/`put_bytes` take a `ServeAs` now.** How a file may be served
    is decided once, at complete, from what the server made of the bytes, and
    travels with them. `ServeAs::for_object` and the RFC 6266 filename encoding
    moved out of `routes/objects.rs` into `storage/mod.rs` for that reason —
    the route was the only caller and is no longer the only place that needs
    the answer.
  - **MinIO stores `Content-Type`/`Content-Disposition` sent as unsigned headers
    on a presigned PUT**, which is what makes the above work without signing
    them into the upload URL. Real S3 behaves the same way; if some
    implementation ever does not, the symptom is
    `an_object_carries_its_own_headers_in_the_bucket` failing, not a security
    hole — the presigned GET still carries the overrides.
  - **The upload listener stays on the app host.** `PUT /upload/...` is bytes
    coming *in* under a signature this server issued; nothing about it is
    somebody else's content being handed to a browser. Moving it would have
    bought nothing and given the media host a body-accepting route.
  - **The app CSP got `base-uri`, `form-action`, `frame-src`, `worker-src` and
    `manifest-src`.** `style-src 'unsafe-inline'` is still there: it is Vite's
    dev-mode requirement, and **T-702 already owns dropping the dev
    relaxations**. Do not remove it before then or `pnpm tauri dev` breaks.
  - **T-504 and T-506 need nothing new for this.** `Attachment.url` and
    `poster_url` are already absolute and already point at the media origin —
    treat them as opaque and use them as given. Do not build a URL by hand from
    the API base; that base is the app origin and `/objects` does not answer
    there.
  - Still a human check, as with T-501/T-502: **a real 400 MB video, over a real
    network, from a server with real DNS for both names.**
- ✅ **T-504 · The media UI + link cards** — effort: **medium** — landed 2026-08-25
  `GET /media` plus the grid in the rail, filters by person/kind/date, stars,
  and each item clicks back to its message. Link cards are one line and the
  server fetches them. **The client can now share a file at all** — `+ file`,
  drag, or paste in the composer — which M5 needed and nothing before this had.
  `crates/linger-server/tests/media.rs` (10 tests) drives it all over real HTTP;
  `media.test.ts`, `upload.test.ts` and four new `markdown.test.ts` cases cover
  the client's arithmetic.

  **Decisions and surprises, for whoever picks up T-505/T-506:**

  - **`MediaItem` grew, as the parking lot expected.** It is flat and carries
    `kind`, `cursor`, `author_id`, `created_at`, `excerpt` and `starred_at`
    alongside an optional `attachment` *or* `link`. A pin carries neither and
    leans on `excerpt`. The grid sorts and filters over the fields every item
    has and only reaches for the payload once it knows which cell it is drawing.
  - **Paging is a keyset cursor, not `before=<id>`, and PROTOCOL §6 says so
    now.** The three sources are three tables and their ids are not comparable
    with each other, so the cursor is `<created_at>:<id hex>` — the sort key —
    and it is opaque. A link item appends its position so every item has a
    unique key; `before` ignores that part.
  - **A message's links stay in one page.** Each source is limited by *group*
    (an upload is one, a message's links are however many it has) and the merge
    stops on a group boundary, so a page can hold slightly more than `limit`.
    Without that, a page ending halfway through a message's links would step
    over the rest of them on the next cursor. It is why `link_groups` runs the
    filter twice — once outside, once in a subquery that picks the messages.
  - **Stars are on uploads only**, which is PROTOCOL's shape (`PUT
    /media/:attachment_id/star`) and the honest one: a star is what stops T-505
    sweeping a file at 365 days, and a link or a pin has no object to keep. The
    grid draws the control only where it means something. Anyone can star
    anything and there is no per-person star — the collection belongs to the
    server, not to a reader.
  - **`starred_at IS NOT NULL DESC` before the date makes paging subtle.** If
    the cursor points at a starred item we are still inside the starred run, so
    the attachment query has to return starred items after the cursor *and*
    every unstarred one; if it does not, everything starred is behind us and no
    starred item may come back at all. `a_star_sorts_first_and_paging_never_
    repeats_or_skips` walks nine items two at a time and compares with one big
    page, which is the test that would catch getting this backwards.
  - **Links are extracted when a message is written, into `message_links`**, and
    re-extracted on every edit — an edit that drops a link drops its card from
    the collection. The alternative was scanning every body ever written on
    every grid load. `linger-server::links::extract` and the client's
    `linkTargets` deliberately agree, including the trailing-punctuation and
    unbalanced-paren rules, and both normalise through a URL parser so the
    string the card is keyed by is identical on both sides.
  - **The SSRF guard refuses ports and bare IPs before it does any DNS**, which
    is also why the fetch cannot be integration-tested against a local server:
    a test HTTP listener is on `127.0.0.1:<port>`, and that is precisely what
    the guard exists to refuse. So `a_preview_never_goes_looking_inside_the_
    network` proves the negative — a real listener on loopback, eight shapes of
    URL aimed at it, zero connections — and the resolve half (every address a
    name answers with must be public) is unit-tested in `links::tests`. Making a
    public name resolve to a private address needs a network a test cannot have.
  - **A refused or failed fetch is stored as `state='failed'`,** not left blank.
    Without that row every reader who scrolled past the message would trigger
    the same doomed fetch. Successes stand for a week, failures for an hour.
  - **Favicons come back as `data:` URIs.** A remote `<img>` would hand the
    linked site the IP of everyone who scrolled past. Only sniffed raster bytes
    are accepted — an SVG "favicon" is a script, and it would be inlined into
    the app's own origin, which is the worst place for one.
  - **The app CSP now allows `http://localhost:*` and `http://127.0.0.1:*` in
    `img-src` and `media-src`**, matching what `connect-src` already had, so
    `pnpm dev` against a local server can draw an uploaded picture. **A LAN
    server on plain http at, say, `192.168.1.5` still cannot** — its images are
    blocked. T-702 owns the CSP; that is the decision to make there, and the
    alternative is telling LAN hosts to put a certificate on it.
  - **Going to an old item's message reuses `loadUntil`**, the same reach "since
    you were gone" uses, so it walks back at most ten pages (a thousand
    messages). Past that the room simply opens at the newest message rather than
    hanging. A real jump to an eight-month-old item would need a message window
    endpoint (`GET /messages/:id` plus paging both ways) and a stream store that
    can hold a window it did not anchor at the end — worth doing, not worth
    doing inside this task.
  - **Not verified by a human yet:** the React half. Types, unit tests, the
    production build and a server-side render of the panel and the attachments
    all pass, but nobody has clicked `+ file` in a running app. That belongs
    with the milestone check the last three tasks also deferred — **a real
    400 MB video, over a real network, from a server with real DNS** — and it is
    now possible to do end to end for the first time.
- ✅ **T-505 · Expiry + storage accounting** — effort: **medium** — landed 2026-08-26
  Files age out at `LINGER_FILE_EXPIRY_DAYS` (365, or `off`) unless starred or
  on a pinned message; `LINGER_POOL_BYTES` is the ceiling; `GET /server` carries
  `storage_used_bytes`, `storage_limit_bytes` and `file_expiry_days`, and the
  status bar draws the first two (SPEC §5.6).
  `crates/linger-server/tests/expiry.rs` (8 tests) uploads real files over real
  HTTP, ages the rows, sweeps, and checks the objects stopped being served.

  **Decisions and surprises, for whoever picks up T-506 and M6:**

  - **The pool moved out of the database.** T-501 parked it in
    `server_config['pool_bytes']` against a host-facing endpoint, and
    `docs/decisions.md` says that endpoint is not going to exist. Both knobs are
    read from the environment at startup and `repo::attachments::pool_limit` is
    gone — one source of truth, and it is the compose file. `PATCH /server` is
    untouched, so nothing about it needs a host UI.
  - **`LINGER_POOL_BYTES` takes a unit.** `250GB`, `500MB`, `2TB`, or a plain
    byte count. Writing 53687091200 in a compose file is how the number ends up
    wrong. A pool smaller than one 500 MB file is a startup error rather than a
    server that answers "storage is full" to every upload forever.
  - **The sweeper takes three kinds of object, and only the first is about
    age.** The spec's rule is one of them. The other two are things this task
    found: a **deleted message** keeps its file (delete is a tombstone, T-30x),
    and nothing anywhere will ever draw it again — the media grid and the stream
    both filter on `deleted_at` — so the bytes were unreachable *and* counted
    against the pool for good. Those go at once, and a star does not hold them:
    a star stops a file ageing out and this is not ageing out. And a **finished
    upload that never became a message** ages out on the normal window;
    `routes::uploads` only ever swept uploads that never *completed*.
  - **A status image is skipped whatever its age.** It is not on a message, so
    the orphan rule above would take it. T-506 owns the rest of that story; this
    is the half that had to be here or T-506 would land a feature that quietly
    breaks a year later. `a_status_image_is_never_swept` pins it.
  - **Bytes first, row second.** The other order can lose an object with nothing
    left pointing at it — a file nobody can see and nobody can delete. This way
    a crash in between leaves a row whose bytes are gone, and the next pass
    finishes it. A backend that cannot delete right now keeps both.
  - **The task lives in `main`, not `AppState`.** Building the state is what
    every integration test does, and none of them want a loop running behind
    them. `expiry::sweep` is public so a test drives one pass directly; ageing
    is faked by moving `created_at` back, which is the only honest way to test a
    year inside a test that has to finish.
  - **The status bar figure is polled, not pushed** — one `GET /server` per
    server every two minutes, on the existing `useNow` clock. It is a number
    rounded to two digits that moves when somebody shares something big; a
    gateway frame for it would be a protocol change for a cosmetic figure.
  - **`GET /server` reports expiry to everybody, and the media panel says it in
    words** next to the star control, because a star is the only thing that
    stops a file going and that is worth knowing where the star is.
  - **Not verified by a human yet:** the status bar and the media panel line, in
    a running app. Types, unit tests, the production build and the whole of
    `scripts/check.sh` pass. This is the same React half T-504 deferred, and it
    belongs with the milestone check those tasks also deferred — **a real 400 MB
    video, over a real network, from a server with real DNS**.
- ✅ **T-506 · The status image** — effort: **low** — landed 2026-08-26
  SPEC §4.6's last bullet: one image on a status, ≤512 KB, drawn at 400×200 in
  the roster card and the name popover at once. The editor picks a file,
  uploads it through the T-501 pipeline, and the server checks the id against
  what is actually stored before it will keep it.
  `crates/linger-server/tests/status_image.rs` (9 tests) uploads real images
  over real HTTP and drives the whole of it.

  **Decisions and surprises, for whoever picks up M6:**

  - **The wire names the image by attachment id, not by storage key.** The
    field was `image_key: string`, and the client cannot produce one: PROTOCOL
    §6 says object URLs are opaque and the media origin is not on the wire
    anywhere, so there was nothing for a client to build a key out of. So
    `UserStatus` now carries `image_id` (the client's, an `AttachmentId`) and
    `image_url` (the server's, built from the key it stores). The database
    column is unchanged and still holds the object key — the sweeper joins on
    it — and `storage::key_owner` is `object_key` read backwards, which is the
    whole of the conversion.
  - **`image_url` is server-owned the way `away_since` already was.** Send
    anything you like for it; `PATCH /me` ignores it. That pattern was already
    in this exact type, which is what made a read-only field inside a request
    type not a smell.
  - **The four checks live in `validate::status_image`**, which is the first
    thing in `validate.rs` that touches the database — everything else there is
    a pure function over a string. It returns the object key rather than a
    yes, so the id a client sent is never what reaches a URL: the answer is
    built from the row the server found. Somebody else's file is `FORBIDDEN`,
    not `NOT_FOUND` — they are telling us about a file that exists.
  - **Replacing an image deletes the old one, and that had to be the client's
    job as well as the server's.** The server drops the file a status stops
    pointing at, unless it is also on a message (then it belongs to the
    message). But an image picked in the editor and then replaced or abandoned
    was never saved to anything, so the server never hears about it — the
    editor takes those back with `DELETE /uploads/:id` on replace, on remove,
    and on cancel. Without that half, opening the editor and picking three
    pictures leaves two against the pool forever, since the sweeper skips
    status images.
  - **`repo::users` needed `&Config`** to build `image_url`, which is ten call
    sites. `repo::attachments::by_id` already took one, so the shape was
    already in the repo layer.
  - **400×200 is the box, not the size.** The roster panel is 240px wide and
    the popover 260px, so the width gives way and the 2:1 aspect ratio does
    not, with `object-fit: cover` so nothing is ever stretched.
  - **The client refuses before it uploads** (`imageProblem` in `status.ts`):
    not an image, or over 512 KB, and it says both numbers. The server refuses
    the same two things afterwards, and there is one case where only the server
    can: it re-encodes every image it takes, so a file that was just under the
    cap can land just over it. The editor shows the server's sentence when that
    happens.
  - **Not verified by a human yet:** nobody has picked a picture in a running
    app. Types, unit tests, the integration tests and the whole of
    `scripts/check.sh` pass. This is the same React half T-504 and T-505
    deferred, and it belongs with the milestone check all three deferred — **a
    real 400 MB video, over a real network, from a server with real DNS** —
    plus the acceptance line this task adds: *see the image at 400×200 in both
    the roster card and the popover, on a second client.*

## M6 — styling: names, palette, themes, fonts

*Milestone check: a gradient name from two palette keys, contrast verifiably ≥4.5:1 in both themes (the CI property test already guards the values).*

- ⬜ **T-601 · Name rendering engine** — effort: **medium**
  Build step: emit `palette.generated.css` from `linger-core::palette::css_variables`
  (single source of truth; oklch or hex per T-002's verdict). Render styled names
  everywhere names appear; gradient fixed 92°; shimmer (4s linear)/glow honor
  `prefers-reduced-motion`, disabled in compact + IRC; "normalize everyone"
  toggle flattens names *and* message fonts.
- ⬜ **T-602 · Style picker + settings** — effort: **medium**
  Two-click named-color picker (mIRC energy, modern craft), font/weight/italic/
  effect, live preview, msg-font override. Server already validates keys.
- ⬜ **T-603 · Themes + time-of-day warmth** — effort: **low**
  Light theme tokens exist; add the ~200K post-sunset warmth shift (one variable
  swap, user-disableable) and theme switching.
- ⬜ **T-604 · Font pipeline** — effort: **low**
  Script: fetch the 12 faces (`assets/fonts/README.md` table), subset
  (latin/latin-ext, 400/500/700 + italics) to woff2, keep OFL texts,
  `@font-face` wiring. No CDN.

## M7 — packaging and updates

*Milestone check: a signed installer per OS; one auto-update ships end-to-end.
Budget the full estimate; notarization is a version-sensitive slog — follow
current vendor docs, not memory (AGENTS.md).*

- ⬜ **T-701 · Updater + signing keys** — effort: **high**
  Tauri updater; generate the signing key and **back it up offline before
  anything ships** (losing it = no more updates, ARCHITECTURE §7.7). Release
  workflow: tag → build 3-OS installers → publish manifest.
- ⬜ **T-702 · Windows signing + macOS notarization** — effort: **high**
  Needs certs/Apple developer account (Matt). Harden CSP for release while here
  (drop dev relaxations from `tauri.conf.json`).
- ⬜ **T-703 · Server image publish** — effort: **low**
  ghcr.io workflow for `deploy/Dockerfile` (ffmpeg is already in it, T-501; the
  CI *runner* still needs it, or the video-poster test keeps skipping),
  version tags, compose points at it.

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
**T-911…T-917** now, so they do not sit in the M5 task block. Still V1 (SPEC §6,
item 8). The Linux and Windows spikes are already retired, `linger-activity`
already compiles, and the Null backend already reports nothing — which is the
correct product until this comes back. It is not needed for a usable chat app,
and it is large: four OS backends, a poller, a registry, and a sharing-controls
UI. **Do not start T-911.** M5 (uploads) starts when M4.5's check passes.*

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
