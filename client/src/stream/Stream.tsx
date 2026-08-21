/**
 * The message stream: the room you are looking at.
 *
 * Four things here are load-bearing.
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
 * **Nothing renders as HTML.** Message bodies are parsed into a tree of known
 * node kinds and drawn as React elements (`markdown.ts`, `Markdown.tsx`), so a
 * body that looks like markup is text that looks like markup.
 *
 * **Reactions are weight, never numbers** (SPEC §4.8). The count comes down the
 * wire and goes into the hover text and the accessible label; it is never drawn
 * as a numeral.
 *
 * **Nothing here counts anything** (SPEC §4.2). Where you left off is a line in
 * the stream, not a number beside a room name, and "since you were gone" is
 * something you pull from the header rather than something that arrives.
 */
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  type CSSProperties,
  type FormEvent,
  type KeyboardEvent,
  type RefObject,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import type { Message } from "../generated/Message";
import type { MessageId } from "../generated/MessageId";
import type { Room } from "../generated/Room";
import type { RoomId } from "../generated/RoomId";
import type { User } from "../generated/User";
import { ApiError, type AuthedApi } from "../lib/api";
import { type Density, DENSITIES } from "../lib/density";
import {
  deleteMessage,
  editMessage,
  enterRoom,
  loadOlder,
  loadUntil,
  markRead,
  openRoom,
  sendMessage,
  startedTyping,
  toggleReaction,
  typistsIn,
  useGateway,
} from "../lib/gateway";
import { peopleList } from "../notify/rules";
import Markdown, { type MentionLookup } from "./Markdown";
import { mentionHandles, plainText } from "./markdown";
import { REACTIONS, reactionOf, reactionTitle, reactionWeight } from "./reactions";
import { buildRows, type StreamRow } from "./rows";
import { ageOpacity, clockTime, fullTime, sessionLabel } from "./time";
import "./stream.css";

/**
 * Start fetching older history once the top of the loaded range comes this
 * close. Roughly two screens: far enough ahead that the page has landed before
 * anyone reaches the end of what is there.
 */
const BACKFILL_MARGIN_PX = 1200;

/**
 * `linger-core::limits::MAX_MESSAGE_CHARS`. The server is the authority and
 * refuses anything longer; this copy exists so the composer can say so before
 * the round trip rather than after it.
 */
const MAX_MESSAGE_CHARS = 8000;

/** How much of a quoted message the reply line shows before it trails off. */
const REPLY_EXCERPT_CHARS = 140;

/**
 * How close to the bottom counts as having seen the newest message. A couple of
 * lines of slack, because a scroll that lands a pixel short is still a person
 * looking at the end of the room.
 */
const BOTTOM_MARGIN_PX = 48;

/** How many people "since you were gone" names before it trails off. */
const SINCE_NAMES = 5;

/**
 * What a message row can ask the stream to do. The two that talk to the server
 * hand back the promise rather than swallowing it, because the row is where a
 * refusal has to be shown — it is the thing the person was pointing at.
 */
interface Actions {
  reply: (message: Message) => void;
  edit: (message: Message) => void;
  react: (message: Message, key: string) => Promise<void>;
  remove: (message: Message) => Promise<void>;
  jumpTo: (id: MessageId) => void;
}

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
  const me = gateway.me;
  const loaded = stream !== undefined;
  const now = useNow();
  const scroller = useRef<HTMLDivElement | null>(null);

  // A room re-opens itself if the store drops it, which happens when a
  // re-identify throws loaded history away.
  useEffect(() => {
    if (!loaded) void openRoom(api, room.id);
  }, [api, room.id, loaded]);

  // Walking in is what pins the "you left off here" line. Keyed on the room
  // alone so it happens once per visit rather than once per message that
  // arrives while you are standing there.
  useEffect(() => {
    enterRoom(room.id);
  }, [room.id]);

  const people = useMemo(() => new Map(users.map((person) => [person.id, person])), [users]);

  // A mention names a username, because that is the thing that is unique on a
  // server and the thing somebody actually typed.
  const byHandle = useMemo(
    () => new Map(users.map((person) => [person.username, person])),
    [users],
  );
  const mentions = useCallback<MentionLookup>(
    (handle) => {
      const person = byHandle.get(handle);
      if (person === undefined) return null;
      return { name: person.display_name, me: person.id === me?.id };
    },
    [byHandle, me?.id],
  );

  const messages = stream?.messages;
  const atStart = stream?.atStart ?? false;
  // Pinned when the room was opened and never moved after that, so the line
  // stays somewhere you can find your way back to (SPEC §4.2).
  const leftOff = gateway.leftOff[room.id] ?? null;
  const rows = useMemo(
    // IRC mode is one self-contained line per message, so it does not group.
    () => buildRows(messages ?? [], { group: density !== "irc", atStart, leftOff }),
    [messages, atStart, density, leftOff],
  );

  // Replies point at a message by id, and a reply line has to show what it is
  // answering, so the loaded history needs to be addressable both ways.
  const byId = useMemo(() => new Map((messages ?? []).map((held) => [held.id, held])), [messages]);
  const rowOf = useMemo(() => {
    const index = new Map<MessageId, number>();
    rows.forEach((row, at) => {
      if (row.kind === "message") index.set(row.message.id, at);
    });
    return index;
  }, [rows]);

  const [replyTo, setReplyTo] = useState<Message | null>(null);
  const [editing, setEditing] = useState<MessageId | null>(null);
  const [flash, setFlash] = useState<MessageId | null>(null);
  const [since, setSince] = useState(false);

  // A half-written reply belongs to the room it was written in.
  useEffect(() => {
    setReplyTo(null);
    setEditing(null);
    setFlash(null);
    setSince(false);
  }, [room.id]);

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

  // Following a new message down is not one scroll, for the same reason
  // walking into a room is not: a row is an estimated height until it has been
  // drawn once, so the virtualizer's own `followOnAppend` aims at a bottom
  // computed from a guess and lands short by however much the guess was wrong.
  // Before markdown every message was a line or two and the error was a few
  // pixels. A message with a code block in it is off by a screenful.
  //
  // So re-aim per frame until the last row is really drawn — and only when the
  // bottom is what you were looking at. Keying this on the *last* row means a
  // page of older history loading in above never triggers it: that changes how
  // many rows there are without changing which one is last.
  const lastKey = rows[rows.length - 1]?.key;
  useEffect(() => {
    const element = scroller.current;
    if (!element || rows.length === 0 || !landing.current.done) return;
    if (element.scrollHeight - element.scrollTop - element.clientHeight > element.clientHeight) {
      return;
    }
    let frames = 0;
    let pending = 0;
    const step = (): void => {
      virtualizer.scrollToIndex(rows.length - 1, { align: "end" });
      const drawn = virtualizer.getVirtualItems();
      frames += 1;
      if (drawn[drawn.length - 1]?.index === rows.length - 1 || frames >= 30) return;
      pending = requestAnimationFrame(step);
    };
    pending = requestAnimationFrame(step);
    return () => cancelAnimationFrame(pending);
  }, [lastKey, rows.length, virtualizer]);

  /**
   * You have read what you can see.
   *
   * Two conditions, and the second one is the one people forget: the newest
   * message has to be on screen, *and* the window has to have your attention. A
   * room sitting open on a second monitor while you type somewhere else has not
   * been read, and marking it read would quietly eat the line that tells you
   * where you stopped.
   */
  const noteRead = useCallback(() => {
    const element = scroller.current;
    const newest = messages?.[messages.length - 1];
    if (!element || newest === undefined || !document.hasFocus()) return;
    if (element.scrollHeight - element.scrollTop - element.clientHeight > BOTTOM_MARGIN_PX) return;
    markRead(api, room.id, newest.id);
  }, [api, room.id, messages]);

  useEffect(() => {
    noteRead();
    window.addEventListener("focus", noteRead);
    return () => window.removeEventListener("focus", noteRead);
  }, [noteRead]);

  const backfill = useCallback(() => {
    noteRead();
    const element = scroller.current;
    if (!element || element.scrollTop > BACKFILL_MARGIN_PX) return;
    void loadOlder(api, room.id);
  }, [api, room.id, noteRead]);

  // Where "go to where you left off" goes.
  const leftOffRow = useMemo(() => rows.findIndex((row) => row.kind === "left-off"), [rows]);
  const goToLeftOff = useCallback(() => {
    if (leftOffRow < 0) return;
    // The same re-aim the rest of this file does, and for the same reason: a
    // row is an estimated height until it has been drawn, so one jump lands
    // near the target rather than on it.
    let frames = 0;
    const step = (): void => {
      virtualizer.scrollToIndex(leftOffRow, { align: "start" });
      frames += 1;
      if (frames < 6) requestAnimationFrame(step);
    };
    requestAnimationFrame(step);
  }, [leftOffRow, virtualizer]);

  // A room whose first page doesn't fill the window needs the next one before
  // anybody scrolls, or there is nothing to scroll.
  useEffect(() => {
    const element = scroller.current;
    if (!element || !stream || stream.loading || stream.atStart) return;
    if (element.scrollHeight - element.clientHeight <= BACKFILL_MARGIN_PX) {
      void loadOlder(api, room.id);
    }
  }, [api, room.id, stream]);

  const actions: Actions = useMemo(
    () => ({
      reply: (message) => {
        setEditing(null);
        setReplyTo(message);
      },
      edit: (message) => {
        setReplyTo(null);
        setEditing(message.id);
      },
      react: (message, key) => toggleReaction(api, message, key),
      remove: (message) => deleteMessage(api, message),
      jumpTo: (id) => {
        const index = rowOf.get(id);
        if (index === undefined) return;
        virtualizer.scrollToIndex(index, { align: "center" });
        // The message you jumped to is somewhere in the middle of a screen of
        // other messages, so say which one it was. It fades on its own; there
        // is nothing to dismiss.
        setFlash(id);
        window.setTimeout(() => setFlash((current) => (current === id ? null : current)), 1600);
      },
    }),
    [api, rowOf, virtualizer],
  );

  // The composer's Up-arrow shortcut, and the keyboard's only route to editing.
  const lastMine = useMemo(() => {
    const held = messages ?? [];
    for (let at = held.length - 1; at >= 0; at -= 1) {
      const message = held[at];
      if (message && message.author_id === me?.id && message.deleted_at === null) return message;
    }
    return null;
  }, [messages, me?.id]);

  const items = virtualizer.getVirtualItems();
  const newest = gateway.newest[room.id];
  // Something happened here after you stopped reading. The header offers to
  // tell you about it; nothing pushes it at you (SPEC §4.2).
  const strayed = leftOff !== null && newest !== undefined && newest > leftOff;

  return (
    <main className="stream">
      <header className="stream-header">
        <span className="room-name">#{room.slug}</span>
        {room.topic ? <span className="room-topic meta">{room.topic}</span> : null}
        {strayed ? (
          <button
            type="button"
            className="since-pull meta"
            aria-expanded={since}
            onClick={() => setSince((open) => !open)}
          >
            since you were gone
          </button>
        ) : null}
        <DensityPicker density={density} onChange={onDensityChange} />
      </header>

      {strayed && since && leftOff !== null ? (
        <SinceYouWereGone
          api={api}
          roomId={room.id}
          leftOff={leftOff}
          messages={messages ?? []}
          atStart={atStart}
          people={people}
          now={now}
          onGo={() => {
            setSince(false);
            goToLeftOff();
          }}
        />
      ) : null}

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
                  ) : row.kind === "left-off" ? (
                    // The whole of what replaces the badge. One line, in accent,
                    // which SPEC §5.3 spends on exactly four things and this is
                    // the first of them.
                    <p className="left-off">
                      <span className="left-off-label">you left off here</span>
                    </p>
                  ) : (
                    <MessageRow
                      api={api}
                      row={row}
                      author={author}
                      me={me}
                      people={people}
                      mentions={mentions}
                      repliedTo={
                        row.message.reply_to === null ? undefined : byId.get(row.message.reply_to)
                      }
                      now={now}
                      irc={density === "irc"}
                      editing={editing === row.message.id}
                      flashing={flash === row.message.id}
                      onEditDone={() => setEditing(null)}
                      actions={actions}
                    />
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      <Typing roomId={room.id} people={people} />

      <Composer
        api={api}
        room={room}
        replyTo={replyTo}
        onClearReply={() => setReplyTo(null)}
        onEditLast={() => {
          if (lastMine) actions.edit(lastMine);
        }}
      />
    </main>
  );
}

// ---------------------------------------------------------------------------
// One message
// ---------------------------------------------------------------------------

function MessageRow({
  api,
  row,
  author,
  me,
  people,
  mentions,
  repliedTo,
  now,
  irc,
  editing,
  flashing,
  onEditDone,
  actions,
}: {
  api: AuthedApi;
  row: Extract<StreamRow, { kind: "message" }>;
  author: User | undefined;
  me: User | null;
  people: Map<string, User>;
  mentions: MentionLookup;
  repliedTo: Message | undefined;
  now: number;
  irc: boolean;
  editing: boolean;
  flashing: boolean;
  onEditDone: () => void;
  actions: Actions;
}) {
  const { message, head } = row;
  const name = author?.display_name ?? "someone";
  const deleted = message.deleted_at !== null;
  // The marker half of SPEC §4.2's one exception: a message that names you is
  // findable by eye when you scroll back through a room, not just by the
  // notification you may have missed.
  const namesMe = useMemo(
    () => me !== null && !deleted && mentionHandles(message.body).includes(me.username),
    [message.body, me, deleted],
  );
  const [picking, setPicking] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  // A delete the server refuses, or a reaction that didn't land, has to say so
  // next to the message it was aimed at. Anywhere else and it reads as being
  // about something you are not looking at.
  const run = (work: Promise<void>): void => {
    setProblem(null);
    void work.catch((error: unknown) => {
      setProblem(error instanceof ApiError ? error.message : "Couldn't reach the server.");
    });
  };

  const nameStyle: CSSProperties = {
    fontWeight: author?.style.weight ?? 500,
    fontStyle: author?.style.italic === true ? "italic" : "normal",
  };
  const time = (
    <time
      className="msg-time meta"
      dateTime={new Date(message.created_at).toISOString()}
      title={fullTime(message.created_at)}
    >
      {clockTime(message.created_at, irc)}
    </time>
  );

  const body = deleted ? (
    <span className="msg-gone">deleted</span>
  ) : editing ? (
    <EditBox api={api} message={message} onDone={onEditDone} />
  ) : (
    <Markdown
      source={message.body}
      mentions={mentions}
      trailing={
        message.edited_at === null ? undefined : <span className="msg-edited meta">edited</span>
      }
    />
  );

  const mine = me !== null && message.author_id === me.id;
  const bodyStyle: CSSProperties = { "--age": ageOpacity(message.created_at, now) };

  return (
    <div
      className="msg"
      data-flash={flashing ? "true" : undefined}
      data-names-me={namesMe ? "true" : undefined}
      onMouseLeave={() => {
        setPicking(false);
        setConfirming(false);
      }}
    >
      {repliedTo === undefined && message.reply_to === null ? null : (
        <ReplyLine target={repliedTo} people={people} onJump={actions.jumpTo} />
      )}

      {/* One line per message, timestamps in a fixed-width gutter, the aligned
          nick column mIRC had (SPEC §5, §5.6). No group header, because there
          is no group. */}
      {irc ? (
        <div className="msg-body">
          {time}
          <span className="irc-name" style={nameStyle}>
            {name}
          </span>
          <span className="irc-text">{body}</span>
        </div>
      ) : (
        <>
          {head ? (
            <p className="msg-head">
              <span className="msg-author" style={nameStyle}>
                {name}
              </span>
              {time}
            </p>
          ) : null}
          {/* Aging is one custom property, computed from the timestamp and
              applied to the body only — never the name, never the time
              (SPEC §5.6). */}
          <div className="msg-body" style={bodyStyle}>
            {body}
          </div>
        </>
      )}

      {problem ? <p className="msg-problem meta">{problem}</p> : null}

      {message.reactions.length === 0 ? null : (
        <Reactions
          message={message}
          me={me}
          people={people}
          onReact={(target, key) => run(actions.react(target, key))}
        />
      )}

      {deleted || editing ? null : (
        <div className="msg-actions">
          {picking ? (
            REACTIONS.map((reaction) => (
              <button
                key={reaction.key}
                type="button"
                className="msg-action msg-action-glyph"
                title={reaction.label}
                aria-label={`react with ${reaction.label}`}
                onClick={() => {
                  run(actions.react(message, reaction.key));
                  setPicking(false);
                }}
              >
                {reaction.glyph}
              </button>
            ))
          ) : (
            <>
              {/* Twelve fixed marks, not an emoji picker (SPEC §4.8) — they
                  take over this same strip rather than opening a layer, which
                  keeps the row the height it already was. */}
              <button
                type="button"
                className="msg-action meta"
                onClick={() => setPicking(true)}
                aria-label={`react to ${name}'s message`}
              >
                react
              </button>
              <button
                type="button"
                className="msg-action meta"
                onClick={() => actions.reply(message)}
                aria-label={`reply to ${name}`}
              >
                reply
              </button>
              {mine ? (
                <button
                  type="button"
                  className="msg-action meta"
                  onClick={() => actions.edit(message)}
                >
                  edit
                </button>
              ) : null}
              {mine || me?.is_host === true ? (
                confirming ? (
                  <>
                    <button
                      type="button"
                      className="msg-action msg-action-danger meta"
                      onClick={() => run(actions.remove(message))}
                    >
                      delete for good
                    </button>
                    <button
                      type="button"
                      className="msg-action meta"
                      onClick={() => setConfirming(false)}
                    >
                      keep
                    </button>
                  </>
                ) : (
                  <button
                    type="button"
                    className="msg-action meta"
                    onClick={() => setConfirming(true)}
                  >
                    delete
                  </button>
                )
              ) : null}
            </>
          )}
        </div>
      )}
    </div>
  );
}

/**
 * "Since you were gone", pulled from the room header.
 *
 * SPEC §4.2 is specific that this is something you ask for and never something
 * that arrives, so it does not exist until you open it. What it tells you is
 * *who* has spoken and *when* it started — never how much, because how much is
 * the number this whole app exists to not show you. A count would also be the
 * one thing here that is a lie, since the client only holds the pages it has
 * fetched.
 *
 * Opening it reaches back through history until the message you stopped at is
 * loaded, which is what lets the "you left off here" line be drawn at all when
 * you have been away longer than one page.
 */
function SinceYouWereGone({
  api,
  roomId,
  leftOff,
  messages,
  atStart,
  people,
  now,
  onGo,
}: {
  api: AuthedApi;
  roomId: RoomId;
  leftOff: MessageId;
  messages: readonly Message[];
  atStart: boolean;
  people: Map<string, User>;
  now: number;
  onGo: () => void;
}) {
  useEffect(() => {
    void loadUntil(api, roomId, leftOff);
  }, [api, roomId, leftOff]);

  const found = atStart || messages.some((message) => message.id <= leftOff);
  const after = messages.filter(
    (message) => message.id > leftOff && message.deleted_at === null,
  );

  // Everyone who has spoken since, in the order they first did.
  const names: string[] = [];
  for (const message of after) {
    const who = people.get(message.author_id)?.display_name ?? "someone";
    if (!names.includes(who)) names.push(who);
  }
  const opened = after[0];

  return (
    <section className="since" aria-label="since you were gone">
      {!found ? (
        <p className="meta">looking back…</p>
      ) : opened === undefined ? (
        <p className="meta">nothing since.</p>
      ) : (
        <>
          <p className="since-who">
            {names.length > SINCE_NAMES
              ? `${peopleList(names.slice(0, SINCE_NAMES))} and others`
              : peopleList(names)}
            <span className="since-when meta">{sessionLabel(opened.created_at, now)}</span>
          </p>
          <button type="button" className="since-go meta" onClick={onGo}>
            go to where you left off
          </button>
        </>
      )}
    </section>
  );
}

/** The line above a reply saying what it is answering. */
function ReplyLine({
  target,
  people,
  onJump,
}: {
  target: Message | undefined;
  people: Map<string, User>;
  onJump: (id: MessageId) => void;
}) {
  // The message being answered may simply not be loaded — it is older than the
  // pages we hold. Saying so is better than pretending the reply is a normal
  // message, because the indent already told you it isn't.
  if (target === undefined) {
    return (
      <p className="msg-reply meta">
        <span className="reply-mark">↩</span> an earlier message
      </p>
    );
  }
  const who = people.get(target.author_id)?.display_name ?? "someone";
  const excerpt =
    target.deleted_at !== null ? "deleted" : shorten(plainText(target.body), REPLY_EXCERPT_CHARS);
  return (
    <button type="button" className="msg-reply meta" onClick={() => onJump(target.id)}>
      <span className="reply-mark">↩</span>
      <span className="reply-author">{who}</span>
      <span className="reply-excerpt">{excerpt}</span>
    </button>
  );
}

function shorten(text: string, limit: number): string {
  return text.length <= limit ? text : `${text.slice(0, limit).trimEnd()}…`;
}

/**
 * The marks on a message.
 *
 * Each one's `--weight` runs 0 to 1 and CSS turns it into size and density. No
 * numeral appears: the tally lives in the hover title and the accessible label,
 * which is where SPEC §4.8 puts it.
 */
function Reactions({
  message,
  me,
  people,
  onReact,
}: {
  message: Message;
  me: User | null;
  people: Map<string, User>;
  onReact: (message: Message, key: string) => void;
}) {
  return (
    <div className="reactions">
      {message.reactions.map((group) => {
        // A key this build has never heard of is skipped rather than guessed
        // at, so a newer server adding a thirteenth mark doesn't draw a blank.
        const reaction = reactionOf(group.key);
        if (!reaction) return null;
        const names = group.user_ids.map((id) => people.get(id)?.display_name ?? "someone");
        const mine = me !== null && group.user_ids.includes(me.id);
        const counted = `${group.count} ${group.count === 1 ? "person" : "people"}`;
        return (
          <button
            key={group.key}
            type="button"
            className="reaction"
            data-mine={mine ? "true" : undefined}
            style={{ "--weight": reactionWeight(group.count) }}
            title={reactionTitle(names, reaction.label)}
            aria-pressed={mine}
            aria-label={`${reaction.label}, ${counted}`}
            onClick={() => onReact(message, group.key)}
          >
            <span aria-hidden="true">{reaction.glyph}</span>
          </button>
        );
      })}
    </div>
  );
}

/** Editing in place. Enter saves, Escape gives up, the text survives a refusal. */
function EditBox({
  api,
  message,
  onDone,
}: {
  api: AuthedApi;
  message: Message;
  onDone: () => void;
}) {
  const [draft, setDraft] = useState(message.body);
  const [problem, setProblem] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const box = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    const element = box.current;
    if (!element) return;
    element.focus();
    // The cursor belongs at the end of what is already there, not the start:
    // most edits are a fix at the end or an afterthought.
    element.setSelectionRange(element.value.length, element.value.length);
  }, []);

  useAutoGrow(box, draft);

  const save = async (): Promise<void> => {
    const body = draft.trim();
    if (saving) return;
    if (body === message.body.trim()) {
      onDone();
      return;
    }
    if (body.length === 0) {
      setProblem("An empty message is a delete, and that is a different button.");
      return;
    }
    setSaving(true);
    try {
      await editMessage(api, message, body);
      onDone();
    } catch (error) {
      setProblem(error instanceof ApiError ? error.message : "Couldn't reach the server.");
    } finally {
      setSaving(false);
    }
  };

  return (
    <form
      className="msg-edit"
      onSubmit={(event) => {
        event.preventDefault();
        void save();
      }}
    >
      <textarea
        ref={box}
        className="composer-input"
        rows={1}
        value={draft}
        maxLength={MAX_MESSAGE_CHARS}
        aria-label="edit this message"
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            onDone();
            return;
          }
          if (event.key === "Enter" && !event.shiftKey) {
            event.preventDefault();
            void save();
          }
        }}
      />
      {problem ? <p className="composer-problem meta">{problem}</p> : null}
      <p className="msg-edit-hint meta">enter saves · escape cancels</p>
    </form>
  );
}

// ---------------------------------------------------------------------------
// The composer
// ---------------------------------------------------------------------------

function Composer({
  api,
  room,
  replyTo,
  onClearReply,
  onEditLast,
}: {
  api: AuthedApi;
  room: Room;
  replyTo: Message | null;
  onClearReply: () => void;
  onEditLast: () => void;
}) {
  const [draft, setDraft] = useState("");
  const [problem, setProblem] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const box = useRef<HTMLTextAreaElement | null>(null);

  // Switching rooms should not carry a half-typed line into the new one.
  useEffect(() => {
    setDraft("");
    setProblem(null);
  }, [room.id]);

  // Choosing to reply is choosing to type, so put the cursor where the typing
  // goes.
  useEffect(() => {
    if (replyTo) box.current?.focus();
  }, [replyTo]);

  useAutoGrow(box, draft);

  const submit = async (): Promise<void> => {
    const body = draft.trim();
    if (body.length === 0 || sending) return;
    setSending(true);
    try {
      await sendMessage(api, room.id, body, replyTo?.id ?? null);
      setDraft("");
      setProblem(null);
      onClearReply();
    } catch (error) {
      // The composer is the only thing holding what they typed, so it is the
      // only thing that can tell them it did not go — and it keeps the text.
      setProblem(error instanceof ApiError ? error.message : "Couldn't reach the server.");
    } finally {
      setSending(false);
    }
  };

  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>): void => {
    if (event.key === "Escape" && replyTo) {
      event.preventDefault();
      onClearReply();
      return;
    }
    // Up arrow in an empty box edits the last thing you said. It is the habit
    // from every other chat client, and it is the only way to reach `edit`
    // without a mouse.
    if (event.key === "ArrowUp" && draft === "") {
      event.preventDefault();
      onEditLast();
      return;
    }
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void submit();
    }
  };

  const left = MAX_MESSAGE_CHARS - draft.length;

  return (
    <form
      className="composer"
      onSubmit={(event: FormEvent) => {
        event.preventDefault();
        void submit();
      }}
    >
      {replyTo ? (
        <p className="composer-reply meta">
          <span className="reply-mark">↩</span>
          <span className="reply-excerpt">{shorten(plainText(replyTo.body), 90)}</span>
          <button type="button" className="composer-reply-drop" onClick={onClearReply}>
            don’t reply
          </button>
        </p>
      ) : null}
      {problem ? <p className="composer-problem meta">{problem}</p> : null}
      <div className="composer-row">
        <textarea
          ref={box}
          className="composer-input"
          rows={1}
          value={draft}
          maxLength={MAX_MESSAGE_CHARS}
          onChange={(event) => {
            setDraft(event.target.value);
            if (event.target.value !== "") startedTyping(room.id);
          }}
          onKeyDown={onKeyDown}
          placeholder={replyTo ? "say something back" : `say something in #${room.slug}`}
          aria-label={`message #${room.slug}`}
          autoComplete="off"
        />
        <button className="composer-send" type="submit" disabled={draft.trim().length === 0}>
          send
        </button>
      </div>
      {/* Only near the ceiling. A counter that is always on is a scold. */}
      {left <= 200 ? (
        <p className="composer-room meta">{left} characters left</p>
      ) : null}
    </form>
  );
}

/**
 * Grow a textarea to fit what is in it, up to the height CSS allows.
 *
 * Resetting to `auto` first is the whole trick: `scrollHeight` is the content's
 * height *or* the box's, whichever is larger, so measuring without collapsing
 * it first means the box can grow and never shrink.
 */
function useAutoGrow(box: RefObject<HTMLTextAreaElement | null>, value: string): void {
  useLayoutEffect(() => {
    const element = box.current;
    if (!element) return;
    element.style.height = "auto";
    element.style.height = `${element.scrollHeight}px`;
  }, [box, value]);
}

/**
 * Who is writing something, above the composer.
 *
 * Nobody sends a "stopped typing" — the signal goes stale instead — so this
 * checks the clock on its own rather than waiting for a frame that is never
 * coming. Two seconds is under the six the signal lives for, so the line
 * disappears within a beat of the person stopping.
 */
function Typing({ roomId, people }: { roomId: string; people: Map<string, User> }) {
  const gateway = useGateway();
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 2000);
    return () => window.clearInterval(timer);
  }, []);

  const names = typistsIn(gateway, roomId, now).map(
    (id) => people.get(id)?.display_name ?? "someone",
  );
  // The line holds its space whether or not anyone is typing. A row that
  // appears and disappears would shove the composer up and down while you are
  // aiming at it.
  return (
    <p className="typing meta" aria-live="polite">
      {names.length === 0 ? "" : `${listOf(names)} ${names.length === 1 ? "is" : "are"} typing…`}
    </p>
  );
}

function listOf(names: readonly string[]): string {
  if (names.length === 1) return names[0] ?? "";
  if (names.length > 3) return `${names.length} people`;
  const last = names[names.length - 1] ?? "";
  return `${names.slice(0, -1).join(", ")} and ${last}`;
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
