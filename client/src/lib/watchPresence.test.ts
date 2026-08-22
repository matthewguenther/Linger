/**
 * The presence driver's one job: when something changes, a frame goes out.
 *
 * `presence.test.ts` covers the deciding. This covers the part around it — the
 * guard that stops two passes running at once — because that guard had a bug
 * that no test of `decide` could ever have caught: the frames simply stopped,
 * quietly, and the last ones sent stayed true on the server.
 */
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ClientFrame } from "../generated/ClientFrame";

const sent: ClientFrame[] = [];
let accept = true;

vi.mock("./gateway", () => ({
  send: async (frame: ClientFrame): Promise<boolean> => {
    if (accept) sent.push(frame);
    return accept;
  },
}));

const { setAway, setPresenceLive, setPresenceRoom, startPresence } = await import(
  "./watchPresence"
);

const ROOM = "room-garage";

/** A fresh driver: `startPresence` returns the stop function, which resets it. */
function reset(): void {
  const stop = startPresence();
  stop();
  startPresence();
  sent.length = 0;
  accept = true;
}

/** The driver is async inside; let its sends settle. */
async function settle(): Promise<void> {
  for (let i = 0; i < 8; i += 1) await Promise.resolve();
}

describe("the presence driver", () => {
  beforeEach(() => {
    reset();
    setPresenceLive(false);
  });

  it("says nothing until the gateway is ready", async () => {
    setPresenceRoom(ROOM);
    await settle();
    expect(sent).toEqual([]);
  });

  it("joins the room once it is", async () => {
    setPresenceLive(true);
    setPresenceRoom(ROOM);
    await settle();
    expect(sent).toEqual([{ op: "room.focus", d: { room_id: ROOM } }]);
  });

  it("keeps working after a pass that had nothing to say", async () => {
    // The regression. A tick with no frame to send used to leave the in-flight
    // guard set forever, because the body finished before the promise it was
    // storing existed. Every later tick was swallowed and presence went silent
    // for the rest of the session — and the no-op tick is the common case: it
    // happens on the first pointer move after everything is in sync.
    setPresenceLive(true);
    setPresenceRoom(ROOM);
    await settle();
    sent.length = 0;

    setPresenceRoom(ROOM); // no change: decide returns null, nothing is sent
    await settle();
    expect(sent).toEqual([]);

    setAway("back after work");
    await settle();
    expect(sent).toEqual([
      { op: "room.focus", d: { room_id: null } },
      { op: "presence.update", d: { state: "away", activity: null, away_message: "back after work" } },
    ]);
  });

  it("leaves the room before going away, in that order", async () => {
    setPresenceLive(true);
    setPresenceRoom(ROOM);
    await settle();
    sent.length = 0;

    setAway("brb");
    await settle();
    expect(sent.map((frame) => frame.op)).toEqual(["room.focus", "presence.update"]);
    expect(sent[0]).toEqual({ op: "room.focus", d: { room_id: null } });
  });

  it("comes back, then rejoins the room", async () => {
    setPresenceLive(true);
    setPresenceRoom(ROOM);
    await settle();
    setAway("brb");
    await settle();
    sent.length = 0;

    setAway(null);
    await settle();
    expect(sent).toEqual([
      { op: "presence.update", d: { state: "around", activity: null, away_message: null } },
      { op: "room.focus", d: { room_id: ROOM } },
    ]);
  });

  it("does not say the same thing twice", async () => {
    setPresenceLive(true);
    setPresenceRoom(ROOM);
    await settle();
    setAway("brb");
    await settle();
    sent.length = 0;

    setAway("brb");
    await settle();
    expect(sent).toEqual([]);
  });

  it("says it again when the message is reworded", async () => {
    setPresenceLive(true);
    setAway("brb");
    await settle();
    sent.length = 0;

    setAway("back at six");
    await settle();
    expect(sent).toEqual([
      { op: "presence.update", d: { state: "away", activity: null, away_message: "back at six" } },
    ]);
  });

  it("re-announces away after a reconnect, because the server forgot", async () => {
    setPresenceLive(true);
    setAway("back at six");
    await settle();
    sent.length = 0;

    // A fresh `ready` is a new session on the server: its presence map is
    // per-connection. A dropped socket is not somebody coming back to their desk.
    setPresenceLive(false);
    setPresenceLive(true);
    await settle();
    expect(sent).toContainEqual({
      op: "presence.update",
      d: { state: "away", activity: null, away_message: "back at six" },
    });
  });

  it("tries again after a send the gateway refused", async () => {
    setPresenceLive(true);
    accept = false;
    setPresenceRoom(ROOM);
    await settle();
    expect(sent).toEqual([]);

    accept = true;
    setAway("brb");
    await settle();
    // Still owes the leave, because the join never landed either.
    expect(sent.at(-1)).toEqual({
      op: "presence.update",
      d: { state: "away", activity: null, away_message: "brb" },
    });
  });
});
