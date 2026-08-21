/**
 * The gateway store: the frontend's one piece of shared state.
 *
 * AGENTS allows local component state plus exactly one gateway store, and this
 * is it — subscribe-and-notify instead of a state library. React reads it
 * through `useSyncExternalStore`, so every component that cares re-renders on a
 * change and nothing else does.
 *
 * The connection itself is not here. It lives in the Tauri core
 * (ARCHITECTURE §1), which owns reconnecting, resume and sequence numbers and
 * sends two events up: a connection status, and each sequenced server frame.
 * This file folds those frames into a snapshot the UI can render, answers when
 * the core says it needs a fresh access token, and holds each open room's
 * history — because history arrives two ways, as pages over REST and as frames
 * over the socket, and the two have to be stitched together in one place.
 *
 * Running `pnpm dev` in a plain browser, with no Tauri underneath, leaves the
 * status on `offline` — the same honest degrading as session storage.
 */
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useSyncExternalStore } from "react";

import type { ClientFrame } from "../generated/ClientFrame";
import type { CreateMessageRequest } from "../generated/CreateMessageRequest";
import type { EditMessageRequest } from "../generated/EditMessageRequest";
import type { Message } from "../generated/Message";
import type { MessageId } from "../generated/MessageId";
import type { NotifyRule } from "../generated/NotifyRule";
import type { PresenceEntry } from "../generated/PresenceEntry";
import type { PresenceState } from "../generated/PresenceState";
import type { ReactionGroup } from "../generated/ReactionGroup";
import type { Room } from "../generated/Room";
import type { RoomId } from "../generated/RoomId";
import type { ServerFrame } from "../generated/ServerFrame";
import type { UpdateReadMarkerRequest } from "../generated/UpdateReadMarkerRequest";
import type { User } from "../generated/User";
import type { UserId } from "../generated/UserId";
import { considerFrame } from "../notify/notify";
import type { AuthedApi } from "./api";

/**
 * Mirrors `Status` in `src-tauri/src/gateway.rs`. Allowed to be hand-written:
 * AGENTS rule 7 covers types that cross the *wire*, and this one never leaves
 * the machine. A Rust test pins the `kind` spellings so the halves can't drift.
 */
export type GatewayStatus =
  | { kind: "offline" }
  | { kind: "connecting" }
  | { kind: "connected"; tls: boolean }
  | { kind: "identifying" }
  | { kind: "resuming" }
  | { kind: "ready"; latency_ms: number }
  | { kind: "waiting"; retry_in_ms: number; reason: string }
  | { kind: "needs_token" };

/**
 * One room's loaded history.
 *
 * A room only appears in the store once it has been opened. Message frames for
 * a room nobody has opened are dropped rather than kept, because the page fetch
 * that opens it would fetch them anyway, and half a room's history is worse
 * than none: the gaps are invisible.
 */
export interface RoomStream {
  /** Oldest first. Ids are UUIDv7, so "in id order" is "in time order". */
  messages: Message[];
  /** True once the oldest message held is the oldest the server has. */
  atStart: boolean;
  /** A page is in flight; stops the same backfill firing twice. */
  loading: boolean;
}

export interface GatewayState {
  status: GatewayStatus;
  /** Who the server says we are. Null until the first `ready`. */
  me: User | null;
  users: User[];
  rooms: Room[];
  presence: PresenceEntry[];
  /** Who is in each room, keyed by room id. */
  occupancy: Record<string, UserId[]>;
  /** Loaded history, keyed by room id. */
  streams: Record<string, RoomStream>;
  /**
   * Who is typing where: room id → user id → when they last said so. Kept as a
   * moment rather than a boolean because nobody sends a "stopped" — the signal
   * simply goes stale, and `typistsIn` is what decides when.
   */
  typing: Record<string, Record<string, number>>;
  /**
   * Room id → the newest message you have read, as far as the server knows.
   *
   * This is a *position*, never a quantity. Nothing in this file, and nothing
   * downstream of it, subtracts two of these to get a number — SPEC §4.2 is the
   * whole point of the app and a count is the thing it refuses to have.
   */
  read: Record<string, MessageId>;
  /**
   * Room id → the newest message that exists. Seeded from `ready` and kept up
   * to date by `message.create`, because the server computes a room's
   * `last_message_id` when it is asked for and never pushes a new one.
   *
   * Compared against `read`, this is the whole of the label-weight change: a
   * room either has something in it you have not seen, or it does not.
   */
  newest: Record<string, MessageId>;
  /**
   * Room id → the message the "you left off here" line sits after.
   *
   * Pinned the first time you open the room in this run of the app and then
   * left alone, even as you read past it. That is SPEC §4.2's "persists until
   * scrolled past and stays visible for the rest of the session": a line that
   * moved while you were reading would be a line you could never use to find
   * your way back to where you started.
   */
  leftOff: Record<string, MessageId>;
  /** "Always notify me when this person posts" (SPEC §4.2). */
  notifyRules: NotifyRule[];
  /**
   * User id → when this client watched their connection go away.
   *
   * The roster says how long somebody has been gone, and the server is no help
   * for the people who left while we were watching: it writes `last_seen_at` on
   * disconnect and never pushes the new value. Our own observation is the
   * fresher of the two, so it is kept. A fresh `ready` throws it away, because
   * the users it carries are freshly read from the database.
   */
  offlineAt: Record<string, number>;
}

const EMPTY: GatewayState = {
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
};

let state: GatewayState = EMPTY;
const listeners = new Set<() => void>();

function publish(next: GatewayState): void {
  state = next;
  for (const notify of listeners) notify();
}

function subscribe(notify: () => void): () => void {
  listeners.add(notify);
  return () => {
    listeners.delete(notify);
  };
}

/** Subscribe a component to the gateway. */
export function useGateway(): GatewayState {
  return useSyncExternalStore(subscribe, () => state);
}

// ---------------------------------------------------------------------------
// Folding frames into the snapshot
// ---------------------------------------------------------------------------

/** Replace an entry keyed by id, or append it if it's new. */
function upsert<T>(list: T[], item: T, sameAs: (candidate: T) => boolean): T[] {
  const at = list.findIndex(sameAs);
  if (at < 0) return [...list, item];
  const next = [...list];
  next[at] = item;
  return next;
}

/** The newest message in each room, as `ready` describes them. */
function newestFrom(rooms: Room[]): Record<string, MessageId> {
  const newest: Record<string, MessageId> = {};
  for (const room of rooms) {
    if (room.last_message_id !== null) newest[room.id] = room.last_message_id;
  }
  return newest;
}

function occupancyFrom(presence: PresenceEntry[]): Record<string, UserId[]> {
  const rooms: Record<string, UserId[]> = {};
  for (const entry of presence) {
    if (entry.room_id === null) continue;
    (rooms[entry.room_id] ??= []).push(entry.user_id);
  }
  return rooms;
}

/**
 * Keep `offlineAt` in step with one presence change. Pure, and separate,
 * because "when did they go" is the only thing on a roster card the server
 * does not tell us outright.
 */
function markOffline(
  held: Record<string, number>,
  userId: UserId,
  was: PresenceState,
  now: PresenceState,
): Record<string, number> {
  if (now === "offline") {
    if (was === "offline") return held;
    return { ...held, [userId]: Date.now() };
  }
  if (held[userId] === undefined) return held;
  const next = { ...held };
  delete next[userId];
  return next;
}

/**
 * How many messages one page of history holds. The server clamps `limit` to
 * 100 (PROTOCOL §4), so this is as much as one round trip can ever bring.
 */
const PAGE_SIZE = 100;

function byId(a: Message, b: Message): number {
  if (a.id < b.id) return -1;
  return a.id > b.id ? 1 : 0;
}

/** Insert or replace one message, keeping the list in id order. */
function mergeMessage(list: Message[], message: Message): Message[] {
  const at = list.findIndex((held) => held.id === message.id);
  if (at >= 0) {
    const next = [...list];
    next[at] = message;
    return next;
  }
  const newest = list[list.length - 1];
  // Nearly always: something newer than everything we hold.
  if (newest === undefined || newest.id < message.id) return [...list, message];
  return [...list, message].sort(byId);
}

/** Fold a fetched page (the server sends newest-first) into what we hold. */
function mergePage(list: Message[], page: Message[]): Message[] {
  if (page.length === 0) return list;
  const older = [...page].sort(byId);
  const first = list[0];
  const last = older[older.length - 1];
  // Backfill asks for everything `before` the oldest message we hold, so the
  // ordinary case is a block that belongs entirely above the list. The slow
  // path is for the one race there is: a message arriving over the socket
  // while the first page of the same room is still on the wire.
  if (first === undefined || last === undefined || last.id < first.id) {
    return [...older, ...list];
  }
  const byKey = new Map(list.map((held) => [held.id, held]));
  for (const message of older) byKey.set(message.id, message);
  return [...byKey.values()].sort(byId);
}

/**
 * Replace one reaction group in place, so the server's canonical key order
 * survives. A count of zero is the last person taking theirs back.
 */
function applyReaction(groups: ReactionGroup[], update: ReactionGroup): ReactionGroup[] {
  const at = groups.findIndex((group) => group.key === update.key);
  if (update.count === 0) return at < 0 ? groups : groups.filter((_, index) => index !== at);
  if (at < 0) return [...groups, update];
  const next = [...groups];
  next[at] = update;
  return next;
}

/**
 * Rewrite one room's stream inside a snapshot. Returning the stream unchanged
 * returns the snapshot unchanged, so a frame that touches nothing re-renders
 * nothing.
 */
function withStream(
  current: GatewayState,
  roomId: RoomId,
  change: (stream: RoomStream) => RoomStream,
): GatewayState {
  const stream = current.streams[roomId];
  if (!stream) return current;
  const next = change(stream);
  if (next === stream) return current;
  return { ...current, streams: { ...current.streams, [roomId]: next } };
}

/** Forget that somebody was typing in a room. */
function stoppedTyping(current: GatewayState, roomId: RoomId, userId: UserId): GatewayState {
  const room = current.typing[roomId];
  if (!room || room[userId] === undefined) return current;
  const next = { ...room };
  delete next[userId];
  return { ...current, typing: { ...current.typing, [roomId]: next } };
}

/** Apply one server frame. */
function apply(current: GatewayState, frame: ServerFrame): GatewayState {
  switch (frame.op) {
    case "ready":
      // A fresh `ready` replaces everything. It arrives after a re-identify,
      // which is exactly when our copy is the thing that can't be trusted.
      // Loaded history goes with it: re-identifying means the resume window
      // lapsed, so there may be messages we never saw, and a hole in the middle
      // of a room is invisible in a way an empty room is not. The open room
      // refetches, which costs one request and a scroll position.
      return {
        ...current,
        me: frame.d.user,
        users: frame.d.users,
        rooms: frame.d.rooms,
        presence: frame.d.presence,
        occupancy: occupancyFrom(frame.d.presence),
        streams: {},
        typing: {},
        offlineAt: {},
        // `read` and `leftOff` survive: one is a copy of something the server
        // is holding for us, and the other is where this session started, which
        // a reconnect does not change.
        newest: newestFrom(frame.d.rooms),
      };
    case "presence.update": {
      const entry = frame.d;
      const was = current.presence.find((p) => p.user_id === entry.user_id)?.state ?? "offline";
      return {
        ...current,
        presence: upsert(current.presence, entry, (p) => p.user_id === entry.user_id),
        // Note the moment somebody drops off, and forget it when they come
        // back. Only the *transition* sets it: a second offline frame for
        // somebody already gone must not restart their clock.
        offlineAt: markOffline(current.offlineAt, entry.user_id, was, entry.state),
      };
    }
    case "user.update": {
      const user = frame.d;
      return {
        ...current,
        me: current.me?.id === user.id ? user : current.me,
        users: upsert(current.users, user, (u) => u.id === user.id),
      };
    }
    case "room.create":
    case "room.update": {
      const room = frame.d;
      return { ...current, rooms: upsert(current.rooms, room, (r) => r.id === room.id) };
    }
    case "room.occupancy":
      return {
        ...current,
        occupancy: { ...current.occupancy, [frame.d.room_id]: frame.d.user_ids },
      };
    case "message.create":
    case "message.update": {
      const message = frame.d;
      let next = withStream(current, message.room_id, (stream) => ({
        ...stream,
        messages: mergeMessage(stream.messages, message),
      }));
      // Rooms nobody has opened have no stream to fold this into, and they are
      // exactly the rooms whose label has to change weight. So the newest id is
      // tracked separately from the history.
      const held = next.newest[message.room_id];
      if (frame.op === "message.create" && (held === undefined || held < message.id)) {
        next = { ...next, newest: { ...next.newest, [message.room_id]: message.id } };
      }
      // Saying the thing is how you stop typing it. Without this the line
      // hangs around for a few seconds after the message it was announcing has
      // already arrived, which reads as a second message that never comes.
      return frame.op === "message.create"
        ? stoppedTyping(next, message.room_id, message.author_id)
        : next;
    }
    case "message.delete": {
      const { id, room_id } = frame.d;
      // A delete is a tombstone, not a removal, so reply chains survive
      // (PROTOCOL §4). The frame carries no timestamp and nothing reads the
      // value — only whether there is one.
      return withStream(current, room_id, (stream) => ({
        ...stream,
        messages: stream.messages.map((held) =>
          held.id === id ? { ...held, body: "", deleted_at: held.deleted_at ?? Date.now() } : held,
        ),
      }));
    }
    case "typing": {
      const { room_id, user_id } = frame.d;
      return {
        ...current,
        typing: {
          ...current.typing,
          [room_id]: { ...current.typing[room_id], [user_id]: Date.now() },
        },
      };
    }
    case "reaction.update": {
      // The only frame that names a message without naming its room, so the
      // rooms we hold get searched. There are a handful of them, not hundreds.
      const { message_id, key, count, user_ids } = frame.d;
      const update: ReactionGroup = { key, count, user_ids };
      let next = current;
      for (const roomId of Object.keys(current.streams)) {
        next = withStream(next, roomId, (stream) =>
          stream.messages.some((held) => held.id === message_id)
            ? {
                ...stream,
                messages: stream.messages.map((held) =>
                  held.id === message_id
                    ? { ...held, reactions: applyReaction(held.reactions, update) }
                    : held,
                ),
              }
            : stream,
        );
      }
      return next;
    }
    default:
      return current;
  }
}

// ---------------------------------------------------------------------------
// Talking to the Tauri core
// ---------------------------------------------------------------------------

/** The signed-in connection this store is currently following. */
let connected: AuthedApi | null = null;
let unlisteners: UnlistenFn[] = [];
/**
 * The token we last handed down. Cleared once the handshake works, so a second
 * request for a token means the last one was refused rather than expired — the
 * case where only forcing a refresh gets us back on.
 */
let givenToken: string | null = null;
let supplying: Promise<void> | null = null;

async function supplyToken(): Promise<void> {
  if (supplying) return supplying;
  const api = connected;
  if (!api) return;
  supplying = (async () => {
    try {
      const fresh = await api.accessToken(givenToken !== null);
      givenToken = fresh.token;
      await invoke("gateway_token", {
        token: fresh.token,
        expiresAtMs: fresh.expiresAt,
      });
    } catch {
      // A refused refresh already ends the session through `AuthedApi`; a
      // network failure is temporary and the core will ask again.
    } finally {
      supplying = null;
    }
  })();
  return supplying;
}

/** Open the connection and start following it. Replaces any previous one. */
export async function connect(api: AuthedApi): Promise<void> {
  await disconnect();
  connected = api;
  if (!isTauri()) return;

  const following = api;
  const attach = async (): Promise<void> => {
    const stop = [
      await listen<GatewayStatus>("gateway:status", (event) => {
        if (connected !== following) return;
        // Nothing else knows the token was accepted, so this is where we learn
        // it, and where a later request means "refused", not "expired".
        if (event.payload.kind === "ready") givenToken = null;
        if (event.payload.kind === "needs_token") void supplyToken();
        publish({ ...state, status: event.payload });
      }),
      await listen<ServerFrame>("gateway:frame", (event) => {
        if (connected !== following) return;
        publish(apply(state, event.payload));
        // After the fold, never before: whether a message is worth interrupting
        // somebody for depends on who they are and what rules they have, and
        // the snapshot is where both of those live.
        considerFrame(event.payload, state);
      }),
    ];
    // A disconnect that landed while we were attaching: don't leak listeners.
    if (connected !== following) {
      for (const off of stop) off();
      return;
    }
    unlisteners = stop;
  };
  await attach();

  const token = await api.accessToken();
  givenToken = token.token;
  await invoke("gateway_connect", {
    baseUrl: api.baseUrl,
    token: token.token,
    expiresAtMs: token.expiresAt,
  });
}

/** Close the connection and forget everything it told us. */
export async function disconnect(): Promise<void> {
  connected = null;
  givenToken = null;
  for (const off of unlisteners) off();
  unlisteners = [];
  publish(EMPTY);
  if (isTauri()) await invoke("gateway_disconnect");
}

/**
 * Send one frame upstream. `false` means there was no live connection, which
 * callers treat as "say it again after the next `ready`" rather than an error.
 */
export async function send(frame: ClientFrame): Promise<boolean> {
  if (!isTauri()) return false;
  const sent: boolean = await invoke("gateway_send", { frame });
  return sent;
}

// ---------------------------------------------------------------------------
// Room history
// ---------------------------------------------------------------------------

/** Put one room's stream into the snapshot, creating it if it is new. */
function putStream(roomId: RoomId, stream: RoomStream): void {
  publish({ ...state, streams: { ...state.streams, [roomId]: stream } });
}

async function fetchPage(api: AuthedApi, roomId: RoomId, before: MessageId | null): Promise<void> {
  const room = encodeURIComponent(roomId);
  const range = before === null ? "" : `&before=${encodeURIComponent(before)}`;
  let page: Message[];
  try {
    page = await api.get<Message[]>(`/rooms/${room}/messages?limit=${PAGE_SIZE}${range}`);
  } catch {
    // A page that didn't arrive doesn't need a state of its own. The status bar
    // is already saying what is wrong, and the next scroll asks again.
    const stream = state.streams[roomId];
    if (connected === api && stream) putStream(roomId, { ...stream, loading: false });
    return;
  }
  const stream = state.streams[roomId];
  if (connected !== api || !stream) return;
  putStream(roomId, {
    messages: mergePage(stream.messages, page),
    // A short page means the range scan ran out of room: this is the beginning.
    atStart: page.length < PAGE_SIZE,
    loading: false,
  });
}

/**
 * Open a room: take a place in the store first, then fetch the newest page.
 *
 * That order is the point. From the moment the room is in `streams`, live
 * frames for it are kept, so a message posted while the first page is still on
 * the wire is not lost — the two get folded together whichever way they land.
 *
 * Opening a room that is already loaded does nothing, which is what keeps its
 * scrollback when you switch away and come back.
 */
export async function openRoom(api: AuthedApi, roomId: RoomId): Promise<void> {
  if (connected !== api || state.streams[roomId]) return;
  putStream(roomId, { messages: [], atStart: false, loading: true });
  await fetchPage(api, roomId, null);
}

/**
 * How far back "since you were gone" is willing to reach. Ten pages is a
 * thousand messages; past that the line is somewhere you are not going to
 * scroll to anyway, and the alternative is a loop that pulls a year of history
 * because somebody was on holiday.
 */
const MAX_CATCHUP_PAGES = 10;

/**
 * Load older pages until `id` is inside the loaded range.
 *
 * This is what makes "since you were gone" work when you have been away longer
 * than one page: the "you left off here" line can only be drawn once the client
 * holds the message on *both* sides of it, or it would be marking the top of a
 * page rather than the place you stopped.
 */
export async function loadUntil(api: AuthedApi, roomId: RoomId, id: MessageId): Promise<void> {
  for (let page = 0; page < MAX_CATCHUP_PAGES; page += 1) {
    const stream = state.streams[roomId];
    if (connected !== api || !stream || stream.atStart) return;
    const oldest = stream.messages[0];
    if (oldest !== undefined && oldest.id <= id) return;
    await loadOlder(api, roomId);
    // `loadOlder` declines while a page is already in flight. Nothing changed,
    // so asking again in a tight loop would only spin.
    if (state.streams[roomId] === stream) return;
  }
}

/** Fetch the page before the oldest message held. Safe to call repeatedly. */
export async function loadOlder(api: AuthedApi, roomId: RoomId): Promise<void> {
  const stream = state.streams[roomId];
  if (connected !== api || !stream || stream.loading || stream.atStart) return;
  const oldest = stream.messages[0];
  putStream(roomId, { ...stream, loading: true });
  await fetchPage(api, roomId, oldest?.id ?? null);
}

/**
 * Post a message.
 *
 * The gateway sends the same message back as a frame, and merging is by id, so
 * folding the POST's own answer in here can't double it up. Doing both means
 * the message appears even when the socket is down and REST is fine.
 *
 * Failures are thrown, not swallowed: the composer is the only thing that knows
 * what the person was typing, so it is the only thing that can tell them.
 */
export async function sendMessage(
  api: AuthedApi,
  roomId: RoomId,
  body: string,
  replyTo: MessageId | null = null,
): Promise<void> {
  const request: CreateMessageRequest = {
    body,
    reply_to: replyTo,
    attachment_ids: null,
  };
  const path = `/rooms/${encodeURIComponent(roomId)}/messages`;
  const message = await api.post<Message>(path, request);
  const stream = state.streams[roomId];
  if (connected !== api || !stream) return;
  putStream(roomId, { ...stream, messages: mergeMessage(stream.messages, message) });
}

/**
 * Change what a message says. Author only; the server enforces that and
 * refuses anyone else (PROTOCOL §4).
 *
 * Like sending, the answer is folded in directly as well as arriving as a
 * frame, so an edit lands even with the socket down. Failures are thrown for
 * the same reason: the edit box is holding the new text.
 */
export async function editMessage(
  api: AuthedApi,
  message: Message,
  body: string,
): Promise<void> {
  const request: EditMessageRequest = { body };
  const edited = await api.patch<Message>(`/messages/${encodeURIComponent(message.id)}`, request);
  const stream = state.streams[edited.room_id];
  if (connected !== api || !stream) return;
  putStream(edited.room_id, { ...stream, messages: mergeMessage(stream.messages, edited) });
}

/**
 * Delete a message. Author or host (PROTOCOL §4).
 *
 * The row stays: a delete is a tombstone, so reply chains still point at
 * something. The server answers 204 and tells us nothing, so the local copy is
 * marked the same way the `message.delete` frame marks it.
 */
export async function deleteMessage(api: AuthedApi, message: Message): Promise<void> {
  await api.delete(`/messages/${encodeURIComponent(message.id)}`);
  if (connected !== api) return;
  publish(apply(state, { op: "message.delete", d: { id: message.id, room_id: message.room_id } }));
}

/**
 * Add or take back one reaction, whichever the person does not already have.
 *
 * The server answers 204 with no body, so unlike an edit there is nothing to
 * fold in — it sends the new group as a `reaction.update` frame instead. That
 * frame is the truth, and it usually beats the HTTP response back. What happens
 * here first is a guess at the same answer so the mark moves under the cursor
 * rather than a round trip later; if the guess is wrong, the frame corrects it.
 */
export async function toggleReaction(
  api: AuthedApi,
  message: Message,
  key: string,
): Promise<void> {
  const me = state.me;
  if (!me) return;
  const held = message.reactions.find((group) => group.key === key);
  const mine = held?.user_ids.includes(me.id) ?? false;

  const others = held?.user_ids.filter((id) => id !== me.id) ?? [];
  const guess = mine ? others : [...others, me.id];
  const guessed = (user_ids: UserId[]): void => {
    publish(
      apply(state, {
        op: "reaction.update",
        d: { message_id: message.id, key, count: user_ids.length, user_ids },
      }),
    );
  };
  guessed(guess);

  const path = `/messages/${encodeURIComponent(message.id)}/reactions/${encodeURIComponent(key)}`;
  try {
    if (mine) await api.delete(path);
    else await api.put(path);
  } catch (error) {
    // Put it back. With the socket up a frame would have corrected this
    // anyway, but a refusal while the socket is down would otherwise leave a
    // mark on screen that the server never accepted.
    if (connected === api) guessed(held?.user_ids ?? []);
    throw error;
  }
}

// ---------------------------------------------------------------------------
// Read markers, and where you left off
// ---------------------------------------------------------------------------

/**
 * You walked into a room: pin the "you left off here" line where you were.
 *
 * Call this once per entry and never per render. Inside a visit the line has to
 * hold still while you read past it, or it is not a mark you can find your way
 * back to. Walking out and back in is a different matter — that is a new visit,
 * and pinning it again is the only way the line means anything the second time.
 */
export function enterRoom(roomId: RoomId): void {
  const marker = state.read[roomId];
  if (marker === undefined || state.leftOff[roomId] === marker) return;
  publish({ ...state, leftOff: { ...state.leftOff, [roomId]: marker } });
}

/**
 * Fetch where you had got to in every room.
 *
 * `GET /read` answers with positions, not counts, and there is no endpoint that
 * would answer with a count (PROTOCOL §4). A failure is quiet: the worst case
 * is a session with no "you left off here" line, which is the app being less
 * helpful, not the app being wrong.
 */
export async function loadReadMarkers(api: AuthedApi): Promise<void> {
  let map: Record<string, MessageId>;
  try {
    map = await api.get<Record<string, MessageId>>("/read");
  } catch {
    return;
  }
  if (connected !== api) return;
  // Pin from the server's copy rather than the merged one. This answer and the
  // gateway's `ready` race on every cold start, so by the time it lands the open
  // room may already have been marked read — and the whole point of the line is
  // where you were *before* this session began.
  const leftOff = { ...state.leftOff };
  for (const [roomId, id] of Object.entries(map)) {
    leftOff[roomId] ??= id;
  }
  publish({ ...state, read: { ...map, ...state.read }, leftOff });
}

/** PROTOCOL §4: at most one read-marker write per five seconds per room. */
const READ_DEBOUNCE_MS = 5_000;

const readSentAt: Record<string, number> = {};
const readTimers: Record<string, number> = {};

function flushRead(api: AuthedApi, roomId: RoomId): void {
  const last_read_id = state.read[roomId];
  if (connected !== api || last_read_id === undefined) return;
  readSentAt[roomId] = Date.now();
  const body: UpdateReadMarkerRequest = { last_read_id };
  // Nothing shows a failure and nothing retries. The marker is a convenience,
  // the next `GET /read` is the truth, and a red line in the UI because a
  // bookmark did not save would be worse than the bookmark not saving.
  void api.put(`/rooms/${encodeURIComponent(roomId)}/read`, body).catch(() => undefined);
}

/**
 * Say you have read up to here.
 *
 * Moves only forward, and never moves the "you left off here" line — that one
 * was pinned when you walked in and stays where it is for the session.
 */
export function markRead(api: AuthedApi, roomId: RoomId, messageId: MessageId): void {
  const held = state.read[roomId];
  if (held !== undefined && held >= messageId) return;
  publish({ ...state, read: { ...state.read, [roomId]: messageId } });

  if (readTimers[roomId] !== undefined) return;
  const since = Date.now() - (readSentAt[roomId] ?? 0);
  if (since >= READ_DEBOUNCE_MS) {
    flushRead(api, roomId);
    return;
  }
  readTimers[roomId] = window.setTimeout(() => {
    delete readTimers[roomId];
    flushRead(api, roomId);
  }, READ_DEBOUNCE_MS - since);
}

/** True when a room holds something you have not seen (SPEC §4.2: weight, not a number). */
export function hasNewActivity(current: GatewayState, roomId: RoomId): boolean {
  const newest = current.newest[roomId];
  if (newest === undefined) return false;
  const read = current.read[roomId];
  return read === undefined || read < newest;
}

// ---------------------------------------------------------------------------
// Notify rules
// ---------------------------------------------------------------------------

function sameRule(a: NotifyRule, b: NotifyRule): boolean {
  return a.target_user_id === b.target_user_id && a.room_id === b.room_id;
}

/** "Always notify me when this person posts" — the whole list (SPEC §4.2). */
export async function loadNotifyRules(api: AuthedApi): Promise<void> {
  const rules = await api.get<NotifyRule[]>("/me/notify-rules");
  if (connected !== api) return;
  publish({ ...state, notifyRules: rules });
}

/**
 * Turn one rule on or off. The local copy moves first so the switch answers
 * under the finger; a refusal puts it back and is thrown for the panel to show.
 */
export async function setNotifyRule(
  api: AuthedApi,
  rule: NotifyRule,
  on: boolean,
): Promise<void> {
  const before = state.notifyRules;
  const without = before.filter((held) => !sameRule(held, rule));
  publish({ ...state, notifyRules: on ? [...without, rule] : without });
  try {
    if (on) await api.put("/me/notify-rules", rule);
    else await api.delete("/me/notify-rules", rule);
  } catch (error) {
    if (connected === api) publish({ ...state, notifyRules: before });
    throw error;
  }
}

// ---------------------------------------------------------------------------
// Typing
// ---------------------------------------------------------------------------

/**
 * How long a `typing.start` counts for. The server lets a client send one per
 * four seconds per room (`RATE_TYPING_PER_ROOM`), so this is that window plus
 * enough slack for a slow frame — long enough that someone typing steadily
 * never flickers, short enough that someone who walked away disappears.
 */
const TYPING_TTL_MS = 6_000;

/** Who is typing in a room right now, excluding you. Ids, oldest first. */
export function typistsIn(current: GatewayState, roomId: RoomId, now: number): UserId[] {
  const room = current.typing[roomId];
  if (!room) return [];
  return Object.entries(room)
    .filter(([userId, at]) => now - at < TYPING_TTL_MS && userId !== current.me?.id)
    .sort(([, a], [, b]) => a - b)
    .map(([userId]) => userId);
}

/** When we last told each room we were writing something. */
const typingSentAt: Record<string, number> = {};

/**
 * Tell the room you are writing something. Throttled here as well as on the
 * server, because being rate-limited is a refusal and there is no reason to
 * collect one on every keystroke. The server's window is four seconds
 * (`RATE_TYPING_PER_ROOM`), so this stays just inside it.
 */
export function startedTyping(roomId: RoomId): void {
  const now = Date.now();
  if (now - (typingSentAt[roomId] ?? 0) < TYPING_TTL_MS - 2_000) return;
  typingSentAt[roomId] = now;
  void send({ op: "typing.start", d: { room_id: roomId } });
}

/** The status bar line (SPEC §5.6): protocol text, never a spinner. */
export function statusText(status: GatewayStatus): string {
  switch (status.kind) {
    case "offline":
      return "offline";
    case "connecting":
      return "connecting…";
    case "connected":
      // Only claim TLS when there was a TLS handshake. A server on your own
      // machine over plain http gets the honest word instead.
      return status.tls ? "tls ok…" : "socket ok…";
    case "identifying":
      return "identify…";
    case "resuming":
      return "resume…";
    case "ready":
      return `ready (${status.latency_ms}ms)`;
    case "waiting":
      return `retry in ${Math.max(1, Math.round(status.retry_in_ms / 1000))}s…`;
    case "needs_token":
      return "renewing…";
  }
}
