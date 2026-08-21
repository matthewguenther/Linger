/**
 * The constants in `limits.ts` are copies of Rust ones. This reads the original
 * and compares, so a change on the server side shows up as a failing test here
 * rather than as a message the composer lets you type and the server refuses.
 */
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { MAX_MESSAGE_CHARS, TYPING_INTERVAL_MS } from "./limits";

const RUST = readFileSync(
  fileURLToPath(new URL("../../../crates/linger-core/src/limits.rs", import.meta.url)),
  "utf8",
);

function constant(name: string): string {
  const found = new RegExp(`pub const ${name}[^=]*= ([^;]+);`).exec(RUST);
  if (!found) throw new Error(`${name} is no longer declared the way this test reads it`);
  return (found[1] ?? "").replace(/_/g, "").trim();
}

describe("limits mirror linger-core", () => {
  it("agrees about the message body cap", () => {
    expect(MAX_MESSAGE_CHARS).toBe(Number(constant("MAX_MESSAGE_CHARS")));
  });

  it("stays inside the server's typing rate limit", () => {
    // `RATE_TYPING_PER_ROOM: (u32, u64) = (1, 4)` — one frame per four seconds.
    const [events, seconds] = constant("RATE_TYPING_PER_ROOM")
      .replace(/[()]/g, "")
      .split(",")
      .map((part) => Number(part.trim()));
    expect(events).toBe(1);
    expect(TYPING_INTERVAL_MS).toBeGreaterThan((seconds ?? 0) * 1000);
  });
});
