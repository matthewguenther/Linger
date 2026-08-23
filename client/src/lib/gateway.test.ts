import { describe, expect, it } from "vitest";

import type { GatewayState } from "./gateway";
import { hasNewActivity, hasNewOnServer } from "./gateway";
import type { Room } from "../generated/Room";

function room(id: string, archived = false): Room {
  return {
    id,
    slug: id,
    name: id,
    topic: null,
    position: 0,
    archived_at: archived ? 1 : null,
    last_message_id: null,
  };
}

function snapshot(partial: Partial<GatewayState>): GatewayState {
  return {
    status: { kind: "offline" },
    me: null,
    users: [],
    rooms: [],
    presence: [],
    occupancy: {},
    streams: {},
    typing: {},
    read: {},
    newest: {},
    leftOff: {},
    notifyRules: [],
    offlineAt: {},
    ...partial,
  };
}

describe("hasNewOnServer", () => {
  it("is a boolean, never a count", () => {
    const current = snapshot({
      rooms: [room("a"), room("b")],
      newest: { a: "m2", b: "m4" },
      read: { a: "m1", b: "m3" },
    });
    expect(hasNewOnServer(current)).toBe(true);
    expect(hasNewActivity(current, "a")).toBe(true);
  });

  it("is false when everything has been read", () => {
    const current = snapshot({
      rooms: [room("a")],
      newest: { a: "m1" },
      read: { a: "m1" },
    });
    expect(hasNewOnServer(current)).toBe(false);
  });

  it("ignores archived rooms", () => {
    const current = snapshot({
      rooms: [room("gone", true)],
      newest: { gone: "m9" },
    });
    expect(hasNewOnServer(current)).toBe(false);
  });
});
