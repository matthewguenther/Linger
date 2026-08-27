import { describe, expect, it } from "vitest";

import { updateLine, type UpdateCheck } from "./updates";

describe("updateLine", () => {
  it("says what it is doing while it is doing it", () => {
    expect(updateLine(null, true)).toBe("Looking…");
    expect(updateLine({ kind: "current" }, true)).toBe("Looking…");
  });

  it("is silent before anything has been asked", () => {
    expect(updateLine(null, false)).toBe("");
  });

  it("names the version that is waiting", () => {
    const ready: UpdateCheck = { kind: "ready", version: "0.2.0", notes: null };
    expect(updateLine(ready, false)).toBe("Version 0.2.0 is ready to install.");
  });

  it("says so plainly when there is nothing to do", () => {
    expect(updateLine({ kind: "current" }, false)).toBe("This is the newest version.");
  });

  it("tells an unsigned build to go and fetch one, rather than pretending", () => {
    expect(updateLine({ kind: "unconfigured" }, false)).toMatch(/not built to update itself/);
  });

  it("passes the reason through instead of swallowing it", () => {
    const failed: UpdateCheck = { kind: "failed", reason: "the network is down" };
    expect(updateLine(failed, false)).toBe("Couldn't check for updates: the network is down");
  });
});
