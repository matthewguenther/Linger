import { describe, expect, it } from "vitest";

import { emptyRoom, emptyRoster, noRoomsBody, noRoomsRail } from "./copy";

describe("noRoomsBody", () => {
  it("does not pretend the member can make a room", () => {
    const member = noRoomsBody(true, false);
    expect(member).toMatch(/host/i);
    expect(member).not.toMatch(/make the first room/i);
  });

  it("leaves the host a short fact, because they have a button", () => {
    expect(noRoomsBody(true, true)).toBe("This server has no rooms yet.");
  });

  it("does not claim there are no rooms while we are still connecting", () => {
    expect(noRoomsBody(false, false)).toMatch(/connecting/i);
    expect(noRoomsBody(false, true)).toMatch(/connecting/i);
  });
});

describe("noRoomsRail", () => {
  it("is words, not a dash", () => {
    expect(noRoomsRail()).toMatch(/room/i);
    expect(noRoomsRail()).not.toBe("—");
  });
});

describe("emptyRoster", () => {
  it("does not say the server is empty while we are still connecting", () => {
    expect(emptyRoster(false)).toMatch(/finding/i);
    expect(emptyRoster(true)).toMatch(/nobody/i);
  });
});

describe("emptyRoom", () => {
  it("points at the composer", () => {
    expect(emptyRoom()).toMatch(/type below/i);
  });
});
