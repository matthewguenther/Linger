/**
 * The store holds one server's world per server, and does not mix them up.
 *
 * This is the part of T-412 that would fail silently. Two servers have their
 * own rooms, their own people and their own read markers, and every one of
 * those is keyed by an id that means nothing on the other server. A frame
 * folded into the wrong snapshot would look like a room appearing out of
 * nowhere — no error, no failing request, just the wrong thing on screen.
 *
 * The Tauri core is mocked down to what this file actually talks to: the two
 * events it listens for, and the commands it sends. That is enough to push
 * frames at one server and prove the other never saw them.
 */
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ReadyData } from "../generated/ReadyData";
import type { Room } from "../generated/Room";
import type { ServerFrame } from "../generated/ServerFrame";
import type { User } from "../generated/User";
import type { AuthedApi } from "./api";

/** Every command the store sent down to the core, in order. */
const invoked: { cmd: string; args: Record<string, unknown> }[] = [];

vi.mock("@tauri-apps/api/core", () => ({
  isTauri: () => true,
  invoke: async (cmd: string, args: Record<string, unknown>): Promise<boolean> => {
    invoked.push({ cmd, args });
    return true;
  },
}));

type Handler = (event: { payload: unknown }) => void;
const handlers = new Map<string, Handler>();

vi.mock("@tauri-apps/api/event", () => ({
  listen: async (name: string, handler: Handler): Promise<() => void> => {
    handlers.set(name, handler);
    return () => handlers.delete(name);
  },
}));

// The notifier reaches for a notification plugin and a markdown parser, and
// neither has anything to do with which snapshot a frame lands in.
vi.mock("../notify/notify", () => ({ considerFrame: () => undefined }));

// The sound player reaches for an `AudioContext` that a test runner does not
// have. Whether a knock makes a noise is `sound.test.ts`'s question; this file
// is about where the card ends up.
vi.mock("./sound", () => ({ playKnock: () => false }));

const {
  anyNewActivity,
  connect,
  disconnect,
  dismissKnock,
  hasNewActivity,
  KNOCK_TTL_MS,
  send,
  serverState,
} = await import("./gateway");

const HOME = "https://home.example";
const WORK = "https://work.example";

/**
 * Just enough of an `AuthedApi` for the store: the URL it is filed under, and
 * a token to hand the core. Nothing here makes an HTTP request.
 */
function fakeApi(baseUrl: string): AuthedApi {
  const stub = {
    baseUrl,
    accessToken: async () => ({ token: `token-${baseUrl}`, expiresAt: 0 }),
  };
  // The store only ever touches those two members; the cast is confined to
  // this helper rather than spread through the tests.
  return stub as unknown as AuthedApi;
}

function person(id: string, name: string): User {
  return {
    id,
    username: name,
    display_name: name,
    is_host: false,
    style: {
      font_key: "inter",
      weight: 400,
      italic: false,
      fill: { kind: "solid", color: "azure" },
      effect: "none",
      msg_font_key: null,
    },
    status: null,
    entrance_sound: null,
    last_seen_at: null,
  };
}

function room(id: string, slug: string, lastMessageId: string | null): Room {
  return {
    id,
    slug,
    name: slug,
    topic: null,
    position: 0,
    archived_at: null,
    last_message_id: lastMessageId,
  };
}

function ready(data: Partial<ReadyData> & { user: User }): ServerFrame {
  return {
    s: 1,
    op: "ready",
    d: {
      session_id: "session",
      users: [data.user],
      rooms: [],
      presence: [],
      ...data,
    },
  };
}

/** Push one frame up from the core, as if it had arrived on that server. */
function arrive(server: string, frame: ServerFrame): void {
  handlers.get("gateway:frame")?.({ payload: { server, frame } });
}

function statusOf(server: string, status: unknown): void {
  handlers.get("gateway:status")?.({ payload: { server, status } });
}

describe("the gateway store, with two servers", () => {
  beforeEach(async () => {
    await Promise.all([disconnect(HOME), disconnect(WORK)]);
    invoked.length = 0;
  });

  it("dials each server by name", async () => {
    await connect(fakeApi(HOME));
    await connect(fakeApi(WORK));
    const dialled = invoked.filter((call) => call.cmd === "gateway_connect");
    expect(dialled.map((call) => call.args.baseUrl)).toEqual([HOME, WORK]);
  });

  it("folds a frame into the server it came from and no other", async () => {
    await connect(fakeApi(HOME));
    await connect(fakeApi(WORK));

    const matt = person("u-matt", "Matt");
    arrive(HOME, ready({ user: matt, rooms: [room("r-garage", "garage", null)] }));

    expect(serverState(HOME).rooms.map((r) => r.slug)).toEqual(["garage"]);
    expect(serverState(HOME).me?.display_name).toBe("Matt");
    // Work heard nothing, so it still has nothing.
    expect(serverState(WORK).rooms).toEqual([]);
    expect(serverState(WORK).me).toBeNull();
  });

  it("keeps two servers' connection states apart", async () => {
    await connect(fakeApi(HOME));
    await connect(fakeApi(WORK));

    statusOf(HOME, { kind: "ready", latency_ms: 12 });
    statusOf(WORK, { kind: "waiting", retry_in_ms: 4000, reason: "refused" });

    expect(serverState(HOME).status).toEqual({ kind: "ready", latency_ms: 12 });
    expect(serverState(WORK).status.kind).toBe("waiting");
  });

  it("marks a server that is holding something you have not read", async () => {
    await connect(fakeApi(HOME));
    await connect(fakeApi(WORK));

    const matt = person("u-matt", "Matt");
    arrive(HOME, ready({ user: matt, rooms: [room("r-garage", "garage", "m-9")] }));
    arrive(WORK, ready({ user: matt, rooms: [room("r-standup", "standup", null)] }));

    // A room with a newest message and no read marker is a room with something
    // in it. Still a boolean — nothing here counts anything.
    expect(hasNewActivity(serverState(HOME), "r-garage")).toBe(true);
    expect(anyNewActivity(serverState(HOME))).toBe(true);
    // An empty room is not "something new" on the other server.
    expect(anyNewActivity(serverState(WORK))).toBe(false);
  });

  it("sends a frame to the server it was meant for", async () => {
    await connect(fakeApi(HOME));
    await connect(fakeApi(WORK));
    invoked.length = 0;

    await send(WORK, { op: "typing.start", d: { room_id: "r-standup" } });

    const sends = invoked.filter((call) => call.cmd === "gateway_send");
    expect(sends).toHaveLength(1);
    expect(sends[0]?.args.baseUrl).toBe(WORK);
  });

  it("signing out of one server leaves the other exactly where it was", async () => {
    await connect(fakeApi(HOME));
    await connect(fakeApi(WORK));
    const matt = person("u-matt", "Matt");
    arrive(HOME, ready({ user: matt, rooms: [room("r-garage", "garage", null)] }));
    arrive(WORK, ready({ user: matt, rooms: [room("r-standup", "standup", null)] }));

    // Connecting closes whatever that URL had before, so only what happens
    // from here counts as the sign-out.
    invoked.length = 0;
    await disconnect(HOME);

    expect(serverState(HOME).rooms).toEqual([]);
    expect(serverState(HOME).me).toBeNull();
    expect(serverState(WORK).rooms.map((r) => r.slug)).toEqual(["standup"]);
    // And only that server's socket was told to close.
    const closed = invoked.filter((call) => call.cmd === "gateway_disconnect");
    expect(closed.map((call) => call.args.baseUrl)).toEqual([HOME]);
  });

  it("still hears the server that is left after the other one goes", async () => {
    await connect(fakeApi(HOME));
    await connect(fakeApi(WORK));
    await disconnect(HOME);

    const matt = person("u-matt", "Matt");
    arrive(WORK, ready({ user: matt, rooms: [room("r-standup", "standup", null)] }));

    expect(serverState(WORK).rooms.map((r) => r.slug)).toEqual(["standup"]);
  });

  it("survives an effect that connects, disconnects and reconnects at once", async () => {
    // React's StrictMode double-invoke, exactly: create, destroy, create, with
    // nothing awaited in between. This left a live socket in the core with no
    // listener in the WebView — the app said `ready` and then never applied
    // another frame. It was sighted once under T-410 and reproduced under
    // T-412 with two servers signed in.
    const api = fakeApi(HOME);
    await Promise.all([connect(api), disconnect(HOME), connect(api)]);

    const dialling = invoked.filter(
      (call) => call.cmd === "gateway_connect" || call.cmd === "gateway_disconnect",
    );
    expect(dialling.at(-1)?.cmd).toBe("gateway_connect");

    // And the listener still routes, which is the half that used to go quiet.
    const matt = person("u-matt", "Matt");
    arrive(HOME, ready({ user: matt, rooms: [room("r-garage", "garage", null)] }));
    expect(serverState(HOME).rooms.map((r) => r.slug)).toEqual(["garage"]);
  });

  it("ends where the last call asked, not where the slowest one did", async () => {
    // Signing out while the connect is still dialling. Unserialized, the
    // half-finished connect lands *after* the disconnect and quietly signs you
    // back in — a socket nobody asked for, on an account that just left.
    const api = fakeApi(HOME);
    const opening = connect(api);
    await disconnect(HOME);
    await opening;

    expect(invoked.filter((call) => call.cmd === "gateway_connect")).not.toHaveLength(0);
    const dialling = invoked.filter(
      (call) => call.cmd === "gateway_connect" || call.cmd === "gateway_disconnect",
    );
    expect(dialling.at(-1)?.cmd).toBe("gateway_disconnect");

    // Nothing is following it, so its frames go nowhere.
    const matt = person("u-matt", "Matt");
    arrive(HOME, ready({ user: matt, rooms: [room("r-garage", "garage", null)] }));
    expect(serverState(HOME).rooms).toEqual([]);
  });

  it("takes a removed member off the roster and leaves the other server alone", async () => {
    // T-413. The card has to go without a reload, and it has to go on this
    // server only — a user id means nothing next to the server it came from.
    await connect(fakeApi(HOME));
    await connect(fakeApi(WORK));
    const matt = person("u-matt", "Matt");
    const callie = person("u-callie", "Callie");

    arrive(HOME, ready({ user: matt, users: [matt, callie] }));
    arrive(WORK, ready({ user: matt, users: [matt, callie] }));
    arrive(HOME, {
      s: 2,
      op: "presence.update",
      d: {
        user_id: callie.id,
        state: "around",
        room_id: null,
        away_message: null,
      },
    });

    arrive(HOME, { s: 3, op: "user.remove", d: { user_id: callie.id } });

    expect(serverState(HOME).users.map((person) => person.id)).toEqual([matt.id]);
    expect(serverState(HOME).presence).toEqual([]);
    expect(serverState(WORK).users.map((person) => person.id)).toEqual([matt.id, callie.id]);

    // And letting them back in is the same frame a rename arrives on: the fold
    // appends when the id is unknown, so the card grows back on its own.
    arrive(HOME, { s: 4, op: "user.update", d: callie });
    expect(serverState(HOME).users.map((person) => person.id)).toEqual([matt.id, callie.id]);
  });

  it("drops frames for a server nobody is following", async () => {
    await connect(fakeApi(HOME));
    const matt = person("u-matt", "Matt");

    arrive(WORK, ready({ user: matt, rooms: [room("r-standup", "standup", null)] }));

    expect(serverState(WORK).rooms).toEqual([]);
  });
});

/**
 * Knocks (SPEC §4.9, T-1102).
 *
 * The store is the only place a knock exists — nothing is written down at
 * either end — so these are the tests that prove it arrives, that it can be
 * taken away, and above all that it does not stay.
 */
describe("knocks", () => {
  const matt = person("u-matt", "Matt");
  const callie = person("u-callie", "Callie");

  beforeEach(async () => {
    await Promise.all([disconnect(HOME), disconnect(WORK)]);
    invoked.length = 0;
  });

  it("holds a knock on the server it came from, and names who knocked", async () => {
    await connect(fakeApi(HOME));
    await connect(fakeApi(WORK));
    arrive(HOME, ready({ user: matt, users: [matt, callie] }));

    arrive(HOME, { s: 2, op: "knock", d: { from_user_id: callie.id } });

    expect(serverState(HOME).knocks.map((knock) => knock.from)).toEqual([callie.id]);
    expect(serverState(WORK).knocks).toEqual([]);
  });

  it("keeps two knocks from the same person as two cards", async () => {
    await connect(fakeApi(HOME));
    arrive(HOME, ready({ user: matt, users: [matt, callie] }));

    arrive(HOME, { s: 2, op: "knock", d: { from_user_id: callie.id } });
    arrive(HOME, { s: 3, op: "knock", d: { from_user_id: callie.id } });

    const held = serverState(HOME).knocks;
    expect(held).toHaveLength(2);
    // Distinct ids, or React draws one card and the second knock is invisible.
    expect(new Set(held.map((knock) => knock.id)).size).toBe(2);
  });

  it("takes one away when its card is done, and leaves nothing behind", async () => {
    await connect(fakeApi(HOME));
    arrive(HOME, ready({ user: matt, users: [matt, callie] }));
    arrive(HOME, { s: 2, op: "knock", d: { from_user_id: callie.id } });

    const [knock] = serverState(HOME).knocks;
    expect(knock).toBeDefined();
    dismissKnock(HOME, knock?.id ?? "");

    expect(serverState(HOME).knocks).toEqual([]);
  });

  it("drops a knock that is already older than its card's life", async () => {
    await connect(fakeApi(HOME));
    arrive(HOME, ready({ user: matt, users: [matt, callie] }));

    arrive(HOME, { s: 2, op: "knock", d: { from_user_id: callie.id } });
    // A window that was asleep, or a card whose timer never ran: the stale one
    // must not still be there when the next knock arrives.
    vi.useFakeTimers();
    vi.setSystemTime(Date.now() + KNOCK_TTL_MS + 1);
    arrive(HOME, { s: 3, op: "knock", d: { from_user_id: matt.id } });
    vi.useRealTimers();

    expect(serverState(HOME).knocks.map((knock) => knock.from)).toEqual([matt.id]);
  });

  it("forgets knocks when the session starts over", async () => {
    await connect(fakeApi(HOME));
    arrive(HOME, ready({ user: matt, users: [matt, callie] }));
    arrive(HOME, { s: 2, op: "knock", d: { from_user_id: callie.id } });

    // A fresh `ready` is a new session. A tap on the shoulder from before the
    // reconnect means nothing now.
    arrive(HOME, ready({ user: matt, users: [matt, callie] }));

    expect(serverState(HOME).knocks).toEqual([]);
  });
});
