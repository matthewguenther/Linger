/**
 * The 90-second leave and the ten-minute idle, as arithmetic.
 *
 * These are the edges that look right and are wrong if the clocks are off
 * by a second, so they are written as numbers rather than slept through.
 */
import { describe, expect, it } from "vitest";

import type { PresenceClock } from "./presence";
import {
  decide,
  frameFor,
  IDLE_AFTER_MS,
  LEAVE_AFTER_MS,
  nextCheckAt,
  stillHere,
  wantedRoom,
} from "./presence";

const NOW = 1_700_000_000_000;
const ROOM = "room-garage";
const OTHER = "room-porch";

function clock(extra: Partial<PresenceClock> = {}): PresenceClock {
  return {
    focused: true,
    lastInputAt: NOW,
    blurredAt: 0,
    roomId: ROOM,
    sentRoomId: null,
    sentState: "around",
    ...extra,
  };
}

describe("stillHere", () => {
  it("is true while you are looking and have touched the window recently", () => {
    expect(stillHere(clock(), NOW + 1_000)).toBe(true);
  });

  it("is false after 90s with no input, even if the window is still focused", () => {
    expect(stillHere(clock(), NOW + LEAVE_AFTER_MS)).toBe(false);
  });

  it("gives 90s after backgrounding when you were just here", () => {
    const left = clock({
      focused: false,
      lastInputAt: NOW,
      blurredAt: NOW,
    });
    expect(stillHere(left, NOW + LEAVE_AFTER_MS - 1)).toBe(true);
    expect(stillHere(left, NOW + LEAVE_AFTER_MS)).toBe(false);
  });

  it("does not extend a nearly-expired idle clock just because you alt-tabbed", () => {
    const left = clock({
      focused: false,
      lastInputAt: NOW - 80_000,
      blurredAt: NOW,
    });
    expect(stillHere(left, NOW + 10_000)).toBe(false);
  });

  it("is already false if you had been sitting still before you backgrounded", () => {
    const left = clock({
      focused: false,
      lastInputAt: NOW - LEAVE_AFTER_MS,
      blurredAt: NOW,
    });
    expect(stillHere(left, NOW)).toBe(false);
  });
});

describe("wantedRoom", () => {
  it("is the open room while you are here", () => {
    expect(wantedRoom(clock(), NOW)).toBe(ROOM);
  });

  it("is null with no room open", () => {
    expect(wantedRoom(clock({ roomId: null }), NOW)).toBeNull();
  });

  it("is null once either 90s clock has run out", () => {
    expect(wantedRoom(clock(), NOW + LEAVE_AFTER_MS)).toBeNull();
  });
});

describe("decide", () => {
  it("enters the open room", () => {
    expect(decide(clock(), NOW)).toEqual({ kind: "focus", roomId: ROOM });
  });

  it("does nothing once that room is the one we sent", () => {
    expect(decide(clock({ sentRoomId: ROOM, sentState: "in_room" }), NOW)).toBeNull();
  });

  it("switches rooms in one focus, not a leave then an enter", () => {
    expect(
      decide(clock({ roomId: OTHER, sentRoomId: ROOM, sentState: "in_room" }), NOW),
    ).toEqual({ kind: "focus", roomId: OTHER });
  });

  it("leaves after 90s of no input", () => {
    expect(
      decide(clock({ sentRoomId: ROOM, sentState: "in_room" }), NOW + LEAVE_AFTER_MS),
    ).toEqual({ kind: "focus", roomId: null });
  });

  it("leaves after 90s in the background", () => {
    const left = clock({
      focused: false,
      blurredAt: NOW,
      sentRoomId: ROOM,
      sentState: "in_room",
    });
    expect(decide(left, NOW + LEAVE_AFTER_MS)).toEqual({ kind: "focus", roomId: null });
  });

  it("stays in the room through a brief alt-tab", () => {
    const left = clock({
      focused: false,
      blurredAt: NOW,
      sentRoomId: ROOM,
      sentState: "in_room",
    });
    expect(decide(left, NOW + 10_000)).toBeNull();
  });

  it("goes idle after ten minutes, and only once the room is already left", () => {
    const quiet = clock({
      sentRoomId: null,
      sentState: "around",
      lastInputAt: NOW,
    });
    expect(decide(quiet, NOW + IDLE_AFTER_MS)).toEqual({ kind: "state", state: "idle" });
  });

  it("leaves the room first if both clocks have fired at once", () => {
    expect(
      decide(clock({ sentRoomId: ROOM, sentState: "in_room" }), NOW + IDLE_AFTER_MS),
    ).toEqual({ kind: "focus", roomId: null });
  });

  it("does not send idle twice", () => {
    expect(
      decide(
        clock({ sentRoomId: null, sentState: "idle", lastInputAt: NOW }),
        NOW + IDLE_AFTER_MS,
      ),
    ).toBeNull();
  });

  it("comes back from idle into the room when you are looking at one", () => {
    expect(
      decide(clock({ sentRoomId: null, sentState: "idle" }), NOW),
    ).toEqual({ kind: "focus", roomId: ROOM });
  });

  it("comes back from idle as around when you are not looking at a room", () => {
    expect(
      decide(
        clock({
          roomId: null,
          sentRoomId: null,
          sentState: "idle",
          lastInputAt: NOW,
        }),
        NOW,
      ),
    ).toEqual({ kind: "state", state: "around" });
  });

  it("does not overwrite away", () => {
    expect(
      decide(clock({ sentState: "away", sentRoomId: null }), NOW),
    ).toBeNull();
    expect(
      decide(
        clock({ sentState: "away", sentRoomId: ROOM, lastInputAt: NOW - IDLE_AFTER_MS }),
        NOW,
      ),
    ).toBeNull();
  });
});

describe("nextCheckAt", () => {
  it("wakes at the 90s leave while you are in a room", () => {
    expect(nextCheckAt(clock({ sentRoomId: ROOM, sentState: "in_room" }), NOW)).toBe(
      NOW + LEAVE_AFTER_MS,
    );
  });

  it("wakes at the background leave when you had just been here", () => {
    const left = clock({
      focused: false,
      lastInputAt: NOW,
      blurredAt: NOW,
      sentRoomId: ROOM,
      sentState: "in_room",
    });
    expect(nextCheckAt(left, NOW)).toBe(NOW + LEAVE_AFTER_MS);
  });

  it("wakes when the input clock runs out, even if the background clock has longer", () => {
    const left = clock({
      focused: false,
      lastInputAt: NOW - 80_000,
      blurredAt: NOW,
      sentRoomId: ROOM,
      sentState: "in_room",
    });
    expect(nextCheckAt(left, NOW)).toBe(NOW + 10_000);
  });

  it("wakes at ten minutes once you have already left", () => {
    expect(
      nextCheckAt(
        clock({
          roomId: null,
          sentRoomId: null,
          sentState: "around",
          lastInputAt: NOW - LEAVE_AFTER_MS,
        }),
        NOW,
      ),
    ).toBe(NOW - LEAVE_AFTER_MS + IDLE_AFTER_MS);
  });

  it("is null once idle and nothing on the clock is left", () => {
    expect(
      nextCheckAt(
        clock({
          roomId: null,
          sentRoomId: null,
          sentState: "idle",
          lastInputAt: NOW - IDLE_AFTER_MS,
        }),
        NOW,
      ),
    ).toBeNull();
  });
});

describe("frameFor", () => {
  it("writes the protocol ops, including a null room", () => {
    expect(frameFor({ kind: "focus", roomId: ROOM })).toEqual({
      op: "room.focus",
      d: { room_id: ROOM },
    });
    expect(frameFor({ kind: "focus", roomId: null })).toEqual({
      op: "room.focus",
      d: { room_id: null },
    });
    expect(frameFor({ kind: "state", state: "idle" })).toEqual({
      op: "presence.update",
      d: { state: "idle", activity: null, away_message: null },
    });
  });
});
