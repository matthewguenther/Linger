/**
 * Grouping and session breaks. The interesting cases are the boundaries: a gap
 * of exactly ten minutes, a session break that also has to break the group, and
 * a page of older history whose top is not the start of anything.
 */
import { describe, expect, it } from "vitest";

import type { Message } from "../generated/Message";
import { buildRows, type StreamRow } from "./rows";

const MINUTE = 60 * 1000;
const HOUR = 60 * MINUTE;

let counter = 0;

/** A message that is only as real as these tests need it to be. */
function message(author: string, at: number): Message {
  counter += 1;
  return {
    id: `m${String(counter).padStart(4, "0")}`,
    room_id: "room",
    author_id: author,
    body: "hello",
    reply_to: null,
    attachments: [],
    reactions: [],
    pinned_at: null,
    edited_at: null,
    deleted_at: null,
    created_at: at,
  };
}

function shape(rows: StreamRow[]): string[] {
  return rows.map((row) =>
    row.kind === "divider" ? "divider" : row.head ? `head:${row.message.author_id}` : "cont",
  );
}

const grouped = { group: true, atStart: false };

describe("buildRows", () => {
  it("returns nothing for an empty room", () => {
    expect(buildRows([], { group: true, atStart: true })).toEqual([]);
  });

  it("groups consecutive messages from one author", () => {
    const start = Date.now();
    const rows = buildRows(
      [message("matt", start), message("matt", start + MINUTE), message("matt", start + 2 * MINUTE)],
      grouped,
    );
    expect(shape(rows)).toEqual(["head:matt", "cont", "cont"]);
  });

  it("breaks the group when the author changes", () => {
    const start = Date.now();
    const rows = buildRows([message("matt", start), message("callie", start + MINUTE)], grouped);
    expect(shape(rows)).toEqual(["head:matt", "head:callie"]);
  });

  it("breaks the group after ten minutes of the same author's silence", () => {
    const start = Date.now();
    const justUnder = buildRows([message("matt", start), message("matt", start + 10 * MINUTE)], grouped);
    expect(shape(justUnder)).toEqual(["head:matt", "cont"]);

    const justOver = buildRows(
      [message("matt", start), message("matt", start + 10 * MINUTE + 1)],
      grouped,
    );
    expect(shape(justOver)).toEqual(["head:matt", "head:matt"]);
  });

  it("breaks the session after three hours, and breaks the group with it", () => {
    const start = Date.now();
    const rows = buildRows([message("matt", start), message("matt", start + 3 * HOUR + 1)], grouped);
    expect(shape(rows)).toEqual(["head:matt", "divider", "head:matt"]);
  });

  it("labels the start of history, but not the top of a page", () => {
    const start = Date.now();
    const messages = [message("matt", start)];
    expect(shape(buildRows(messages, { group: true, atStart: true }))).toEqual([
      "divider",
      "head:matt",
    ]);
    expect(shape(buildRows(messages, grouped))).toEqual(["head:matt"]);
  });

  it("gives every row a key that survives older history arriving above it", () => {
    const start = Date.now();
    const older = [message("matt", start - 4 * HOUR)];
    const newer = [message("callie", start)];

    const before = buildRows(newer, grouped);
    const after = buildRows([...older, ...newer], grouped);

    const keptKeys = after.map((row) => row.key);
    for (const row of before) expect(keptKeys).toContain(row.key);
    expect(new Set(keptKeys).size).toBe(keptKeys.length);
  });

  it("puts every message on its own head row in IRC mode", () => {
    const start = Date.now();
    const rows = buildRows(
      [message("matt", start), message("matt", start + MINUTE), message("matt", start + 2 * MINUTE)],
      { group: false, atStart: false },
    );
    expect(shape(rows)).toEqual(["head:matt", "head:matt", "head:matt"]);
  });

  it("still breaks sessions in IRC mode", () => {
    const start = Date.now();
    const rows = buildRows([message("matt", start), message("matt", start + 4 * HOUR)], {
      group: false,
      atStart: false,
    });
    expect(shape(rows)).toEqual(["head:matt", "divider", "head:matt"]);
  });
});
