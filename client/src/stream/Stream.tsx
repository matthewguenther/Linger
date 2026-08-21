/**
 * The message stream: the room you are looking at.
 *
 * Three things here are load-bearing.
 *
 * **It is virtualized from day one** (AGENTS). Ten thousand messages is one
 * evening in a room that gets used, and a list that renders all of them is a
 * list that has to be rewritten later. Only the rows on screen exist in the
 * DOM; every row has a stable key, so a row that was already measured stays
 * measured when older history is prepended above it.
 *
 * **The stream hangs from the bottom.** New messages push the view along when
 * you are already at the end, and history loading in above you must not move
 * what you are reading. Both are the virtualizer's `anchorTo: "end"`.
 *
 * **Nothing renders as HTML.** A message body is parsed into typed nodes and
 * drawn as React elements (`markdown.ts`, `Markdown.tsx`), so markup somebody
 * typed is text that looks like markup and can never be anything else.
 *
 * This file owns the two pieces of state that belong to the room rather than to
 * any one message: which message you are answering, and which one you are
 * editing. Both are one-at-a-time and both are dropped when you change rooms.
 */
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  type CSSProperties,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import type { MessageId } from "../generated/MessageId";
import type { Room } from "../generated/Room";
import type { User } from "../generated/User";
import type { AuthedApi } from "../lib/api";
import { type Density, DENSITIES } from "../lib/density";
import {
  deleteMessage,
  editMessage,
  loadOlder,
  openRoom,
  setReaction,
  useGateway,
} from "../lib/gateway";
import Composer from "./Composer";
import MessageRow, { type MessageActions } from "./MessageRow";
import { buildRows } from "./rows";
import { sessionLabel } from "./time";
import "./stream.css";

/**
 * Start fetching older history once the top of the loaded range comes this
 * close. Roughly two screens: far enough ahead that the page has landed before
 * anyone reaches the end of what is there.
 */
const BACKFILL_MARGIN_PX = 1200;

interface StreamProps {
  api: AuthedApi;
  room: Room;
  /** Everyone the server has told us about, for author names and colors. */
  users: User[];
  density: Density;
  onDensityChange: (density: Density) => void;
}

export default function Stream({ api, room, users, density, onDensityChange }: StreamProps) {
  const gateway = useGateway();
  const stream = gateway.streams[room.id];
  const loaded = stream !== undefined;
  const now = useNow();
  const scroller = useRef<HTMLDivElement | null>(null);

  const [replyToId, setReplyToId] = useState<MessageId | null>(null);
  const [editingId, setEditingId] = useState<MessageId | null>(null);

  // A room re-opens itself if the store drops it, which happens when a
  // re-identify throws loaded history away.
  useEffect(() => {
    if (!loaded) void openRoom(api, room.id);
  }, [api, room.id, loaded]);

  // Answering somebody in one room and editing something in another are both
  // states about a message you can no longer see. Drop them at the door.
  useEffect(() => {
    setReplyToId(null);
    setEditingId(null);
  }, [room.id]);

  const people = useMemo(() => new Map(users.map((person) => [person.id, person])), [users]);

  const messages = stream?.messages;
  const atStart = stream?.atStart ?? false;
  const rows = useMemo(
    // IRC mode is one self-contained line per message, so it does not group.
    () => buildRows(messages ?? [], { group: density !== "irc", atStart }),
    [messages, atStart, density],
  );

  const byId = useMemo(
    () => new Map((messages ?? []).map((message) => [message.id, message])),
    [messages],
  );
  const replyTo = replyToId === null ? null : (byId.get(replyToId) ?? null);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scroller.current,
    // A one-line message in Comfortable. Wrong for most rows, which is fine —
    // it is a starting guess and every row is measured for real once drawn.
    estimateSize: () => 26,
    getItemKey: (index) => rows[index]?.key ?? index,
    overscan: 12,
    anchorTo: "end",
    followOnAppend: true,
  });

  // Walking into a room puts you at the newest message, not where you last
  // were.
  //
  // Getting there takes several goes, and this is the part that is easy to get
  // wrong. Every row starts life as an estimated height and only gets its real
  // one once it has been drawn, so the first jump aims at the bottom of a list
  // of guesses and lands somewhere in the middle of the real one. Drawing the
  // rows it lands on corrects their heights, which moves the bottom again.
  //
  // So jump once per frame until the last row is genuinely on screen. A frame
  // is the right beat because the browser reports a scroll asynchronously: jump
  // twice inside one frame and the second jump is aiming with the first one's
  // stale numbers. And the test is "the last row is drawn", not "the total
  // stopped growing" — the total holds still for a frame all the time while
  // measurements are still arriving.
  const landing = useRef({ room: "", done: false });
  useEffect(() => {
    if (rows.length === 0) return;
    if (landing.current.room !== room.id) landing.current = { room: room.id, done: false };
    if (landing.current.done) return;

    let frames = 0;
    let pending = 0;
    const step = (): void => {
      virtualizer.scrollToIndex(rows.length - 1, { align: "end" });
      const drawn = virtualizer.getVirtualItems();
      frames += 1;
      // The frame cap is a seatbelt, not a mechanism: a room that never settles
      // has to give the scrollbar back rather than fight for it forever.
      if (drawn[drawn.length - 1]?.index === rows.length - 1 || frames >= 60) {
        landing.current.done = true;
        return;
      }
      pending = requestAnimationFrame(step);
    };
    pending = requestAnimationFrame(step);
    return () => cancelAnimationFrame(pending);
  }, [room.id, rows.length, virtualizer]);

  const backfill = useCallback(() => {
    const element = scroller.current;
    if (!element || element.scrollTop > BACKFILL_MARGIN_PX) return;
    void loadOlder(api, room.id);
  }, [api, room.id]);

  // A room whose first page doesn't fill the window needs the next one before
  // anybody scrolls, or there is nothing to scroll.
  useEffect(() => {
    const element = scroller.current;
    if (!element || !stream || stream.loading || stream.atStart) return;
    if (element.scrollHeight - element.clientHeight <= BACKFILL_MARGIN_PX) {
      void loadOlder(api, room.id);
    }
  }, [api, room.id, stream]);

  const actions = useMemo<MessageActions>(
    () => ({
      onReply: (message) => {
        setEditingId(null);
        setReplyToId(message.id);
      },
      onEditStart: (message) => {
        setReplyToId(null);
        setEditingId(message.id);
      },
      onEditCancel: () => setEditingId(null),
      onEditSave: async (message, body) => {
        await editMessage(api, message.id, body);
        setEditingId(null);
      },
      onDelete: (message) => {
        // The refusal that matters — someone else's message — is the server's
        // to make, and it already made it before the button was drawn. A
        // failure here leaves the message alone, which is the honest outcome.
        void deleteMessage(api, message.id).catch(() => undefined);
        if (replyToId === message.id) setReplyToId(null);
        if (editingId === message.id) setEditingId(null);
      },
      onReact: (message, key, on) => {
        void setReaction(api, message.id, key, on).catch(() => undefined);
      },
      onJumpToParent: (parent) => {
        const index = rows.findIndex((row) => row.kind === "message" && row.key === parent.id);
        if (index >= 0) virtualizer.scrollToIndex(index, { align: "center" });
      },
    }),
    [api, rows, virtualizer, replyToId, editingId],
  );

  // Everyone typing here except us: our own keystrokes are not news.
  const typing = useMemo(() => {
    const here = gateway.typing[room.id] ?? {};
    return Object.keys(here)
      .filter((userId) => userId !== gateway.me?.id)
      .map((userId) => people.get(userId)?.display_name ?? "someone");
  }, [gateway.typing, gateway.me, room.id, people]);

  const items = virtualizer.getVirtualItems();

  return (
    <main className="stream">
      <header className="stream-header">
        <span className="room-name">#{room.slug}</span>
        {room.topic ? <span className="room-topic meta">{room.topic}</span> : null}
        <DensityPicker density={density} onChange={onDensityChange} />
      </header>

      {/* `tabIndex` is not decoration: scrollback has to be reachable without a
          mouse, and focused, this takes arrow keys and Page Up like any other
          scrolling region. */}
      <div
        className="stream-body"
        ref={scroller}
        onScroll={backfill}
        role="log"
        aria-label={`messages in #${room.slug}`}
        tabIndex={0}
      >
        {rows.length === 0 ? (
          <p className="placeholder">
            {loaded && atStart ? "Nothing here yet. Say the first thing." : "…"}
          </p>
        ) : (
          <div className="stream-rows" style={{ height: virtualizer.getTotalSize() }}>
            {items.map((item) => {
              const row = rows[item.index];
              if (!row) return null;
              const author =
                row.kind === "message" ? people.get(row.message.author_id) : undefined;
              return (
                <div
                  key={item.key}
                  className="stream-row"
                  data-index={item.index}
                  data-head={row.kind === "message" && row.head}
                  ref={virtualizer.measureElement}
                  style={{ transform: `translateY(${item.start}px)`, ...personStyle(author) }}
                >
                  {row.kind === "divider" ? (
                    <p className="stream-divider">
                      <span className="divider-label">{sessionLabel(row.at, now)}</span>
                    </p>
                  ) : (
                    <MessageRow
                      row={row}
                      author={author}
                      people={people}
                      me={gateway.me}
                      parent={
                        row.message.reply_to === null
                          ? undefined
                          : byId.get(row.message.reply_to)
                      }
                      now={now}
                      irc={density === "irc"}
                      editing={editingId === row.message.id}
                      actions={actions}
                    />
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      <Composer
        api={api}
        room={room}
        replyTo={replyTo}
        replyToAuthor={replyTo === null ? undefined : people.get(replyTo.author_id)}
        onClearReply={() => setReplyToId(null)}
        typing={typing}
      />
    </main>
  );
}

function DensityPicker({
  density,
  onChange,
}: {
  density: Density;
  onChange: (density: Density) => void;
}) {
  return (
    <div className="density" role="group" aria-label="density">
      {DENSITIES.map((mode) => (
        <button
          key={mode}
          type="button"
          className="density-option meta"
          aria-pressed={mode === density}
          onClick={() => onChange(mode)}
        >
          {mode}
        </button>
      ))}
    </div>
  );
}

/**
 * The two colors a person carries: the 3px rule beside their messages, and
 * their name.
 *
 * Both point at a palette variable rather than a color. M7 generates those
 * variables from `linger-core::PALETTE`, the one place the sixteen colors are
 * defined — so this file never learns what "azure" looks like, and the stream
 * picks up real colors the moment that stylesheet exists.
 */
function personStyle(author: User | undefined): CSSProperties {
  const key = paletteKey(author);
  if (key === null) {
    return { "--person-rule": "var(--hairline-strong)", "--person-name": "var(--text-primary)" };
  }
  return {
    "--person-rule": `var(--name-${key}, var(--hairline-strong))`,
    "--person-name": `var(--name-${key}, var(--text-primary))`,
  };
}

function paletteKey(author: User | undefined): string | null {
  if (!author) return null;
  // A gradient name takes its rule from the first of its two colors. Painting
  // the gradient itself is M7's, along with the palette these keys name.
  const fill = author.style.fill;
  const key = fill.kind === "solid" ? fill.color : fill.from;
  // The server validates keys against linger-core::PALETTE (AGENTS rule 8);
  // this is the second lock on the door, because the key is about to become
  // part of a CSS variable name and user content is hostile (ARCHITECTURE §7).
  return /^[a-z]{2,16}$/.test(key) ? key : null;
}

/** `Date.now()`, refreshed every minute, so bodies fade as they age. */
function useNow(): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 60_000);
    return () => window.clearInterval(timer);
  }, []);
  return now;
}
