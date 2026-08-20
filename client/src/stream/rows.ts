/**
 * Messages in, rows out.
 *
 * The stream is not a list of messages — it is a list of *rows*, because a
 * session divider is a row, and because whether a message shows its author's
 * name depends on the message before it. Working that out here, once, keeps the
 * component down to "draw row N" and keeps the virtualizer's job simple: every
 * row has a stable key, so a row that was already measured stays measured when
 * older history is prepended above it.
 */
import type { Message } from "../generated/Message";
import { GROUP_BREAK_MS, SESSION_BREAK_MS } from "./time";

export type StreamRow =
  /** A session break: real whitespace and a soft labeled divider (SPEC §4.7). */
  | { kind: "divider"; key: string; at: number }
  /** One message. `head` means it opens a group and shows the author's name. */
  | { kind: "message"; key: string; message: Message; head: boolean };

export interface BuildOptions {
  /**
   * Group consecutive messages by the same author. False in IRC mode, which is
   * one self-contained line per message (SPEC §5.6).
   */
  group: boolean;
  /**
   * True when the oldest message here is the oldest the server has. Only then
   * does a divider above it mean anything — otherwise it would be labelling the
   * top of a page, not the start of a session.
   */
  atStart: boolean;
}

/** Build the rows for one room. `messages` must be oldest-first. */
export function buildRows(messages: readonly Message[], options: BuildOptions): StreamRow[] {
  const rows: StreamRow[] = [];
  let previous: Message | null = null;

  for (const message of messages) {
    const gap = previous === null ? Number.POSITIVE_INFINITY : message.created_at - previous.created_at;
    const opensSession = previous === null ? options.atStart : gap > SESSION_BREAK_MS;
    if (opensSession) {
      rows.push({ kind: "divider", key: `divider:${message.id}`, at: message.created_at });
    }

    const head =
      !options.group ||
      previous === null ||
      opensSession ||
      previous.author_id !== message.author_id ||
      gap > GROUP_BREAK_MS;

    rows.push({ kind: "message", key: message.id, message, head });
    previous = message;
  }

  return rows;
}
