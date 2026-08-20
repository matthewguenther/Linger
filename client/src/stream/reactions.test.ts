/**
 * The reaction keys are written out in TypeScript and defined in Rust, which is
 * exactly the shape of thing that drifts. So this reads the Rust source and
 * compares. If someone edits `linger-core::REACTIONS`, `pnpm test` says so.
 */
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { reactionFor, REACTIONS, weightOf } from "./reactions";

function keysFromRust(): string[] {
  const source = readFileSync(
    fileURLToPath(new URL("../../../crates/linger-core/src/lib.rs", import.meta.url)),
    "utf8",
  );
  const declaration = /pub const REACTIONS: \[&str; \d+\] = \[([^\]]*)\]/s.exec(source);
  if (!declaration) throw new Error("REACTIONS is no longer declared the way this test reads it");
  return [...(declaration[1] ?? "").matchAll(/"([a-z-]+)"/g)].map((match) => match[1] ?? "");
}

describe("the twelve", () => {
  it("matches linger-core::REACTIONS, in order", () => {
    expect(REACTIONS.map((reaction) => reaction.key)).toEqual(keysFromRust());
  });

  it("gives every key a glyph and a spoken name", () => {
    for (const reaction of REACTIONS) {
      expect(reaction.glyph.length).toBeGreaterThan(0);
      expect(reaction.label.length).toBeGreaterThan(0);
    }
  });

  it("still draws a key from a newer server", () => {
    expect(reactionFor("something-new")).toEqual({
      key: "something-new",
      glyph: "something-new",
      label: "something-new",
    });
  });
});

describe("weight", () => {
  it("rises with the crowd and then levels off", () => {
    expect([1, 2, 3, 4, 5, 6, 7, 8, 40].map(weightOf)).toEqual([0, 1, 2, 2, 3, 3, 3, 4, 4]);
  });

  it("never goes below the lightest mark", () => {
    expect(weightOf(0)).toBe(0);
  });
});
