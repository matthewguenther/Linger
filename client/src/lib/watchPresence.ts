/**
 * Watch the window, send `room.focus`, `idle` and `away`.
 *
 * The clocks and the deciding live in `presence.ts`. This file is the part
 * that cannot be tested without a window: it listens for input and focus,
 * holds the last thing we sent, and talks to the gateway.
 */
import type { PresenceState } from "../generated/PresenceState";
import type { RoomId } from "../generated/RoomId";
import { send, sendOthers } from "./gateway";
import { readFocused, setLooking, stopLooking } from "./looking";
import { decide, frameFor, nextCheckAt, type PresenceAction, type PresenceClock } from "./presence";

interface Sent {
  roomId: RoomId | null;
  state: PresenceState;
  awayMessage: string | null;
}

function applySent(sent: Sent, action: PresenceAction): Sent {
  if (action.kind === "focus") {
    return {
      roomId: action.roomId,
      state: action.roomId === null ? "around" : "in_room",
      // The server clears the away message on any `room.focus`, so our record
      // of what is on the wire has to clear with it.
      awayMessage: null,
    };
  }
  if (action.kind === "away") {
    return { roomId: sent.roomId, state: "away", awayMessage: action.message };
  }
  return { roomId: sent.roomId, state: action.state, awayMessage: null };
}

let live = false;
let openRoomId: RoomId | null = null;
let sentRoomId: RoomId | null = null;
let sentState: PresenceState = "around";
let sentAwayMessage: string | null = null;
let wantAway: string | null = null;
let focused = false;
let lastInputAt = 0;
let blurredAt = 0;
let timer: number | null = null;
let running = false;
let pending = false;
let attached = false;
let unlisten: (() => void) | null = null;

function clock(): PresenceClock {
  return {
    focused,
    lastInputAt,
    blurredAt,
    roomId: openRoomId,
    sentRoomId,
    sentState,
    wantAway,
    sentAwayMessage,
  };
}

function schedule(): void {
  if (typeof window === "undefined") return;
  if (timer !== null) {
    window.clearTimeout(timer);
    timer = null;
  }
  if (!live) return;
  const at = nextCheckAt(clock(), Date.now());
  if (at === null) return;
  timer = window.setTimeout(() => {
    timer = null;
    void tick();
  }, Math.max(0, at - Date.now()));
}

/**
 * Work out what to send, and send it. One of these runs at a time.
 *
 * The guard is a plain boolean, and it has to be. An earlier version held the
 * in-flight promise instead, which looked equivalent and was not: when
 * `decide` has nothing to say, the whole body runs to completion *before* the
 * promise is assigned, so the assignment overwrote the `null` the body had
 * just cleared. Every later tick then saw a non-null promise, set `pending`,
 * and returned — and presence stopped sending anything at all for the rest of
 * the session. Nothing surfaced it: the frames simply stopped, and the last
 * ones sent stayed true on the server.
 *
 * A boolean set before the first statement cannot be beaten by its own body.
 * Anything that arrives mid-flight sets `pending` and the loop goes round
 * again, so a tick is never dropped.
 */
async function tick(): Promise<void> {
  if (running) {
    pending = true;
    return;
  }
  running = true;
  try {
    do {
      pending = false;
      if (!live) break;
      // One action per pass so a refused send cannot skip the next one.
      // `decide` is cheap; looping until it returns null is how "leave, then
      // go idle" happens as two frames rather than a fight.
      for (;;) {
        const action = decide(clock(), Date.now());
        if (action === null) break;
        const frame = frameFor(action);
        const ok = await send(frame);
        if (!ok) break;
        // Idle and away belong to the person, not to one server. Room focus
        // stays on the server we are looking at; the others already got
        // `room.focus(null)` when we switched away.
        if (action.kind !== "focus") void sendOthers(frame);
        const next = applySent(
          { roomId: sentRoomId, state: sentState, awayMessage: sentAwayMessage },
          action,
        );
        sentRoomId = next.roomId;
        sentState = next.state;
        sentAwayMessage = next.awayMessage;
      }
      schedule();
    } while (pending && live);
  } finally {
    running = false;
  }
}

function syncFocus(): void {
  const next = readFocused();
  if (next === focused) return;
  const now = Date.now();
  if (next) lastInputAt = now;
  else blurredAt = now;
  focused = next;
  setLooking(next);
}

function onInput(): void {
  lastInputAt = Date.now();
  syncFocus();
  void tick();
}

function onFocusChange(): void {
  syncFocus();
  void tick();
}

/** The room on screen. Call when it changes; no-ops if it has not. */
export function setPresenceRoom(roomId: RoomId | null): void {
  if (openRoomId === roomId) return;
  openRoomId = roomId;
  void tick();
}

/**
 * Go away, or come back (SPEC §4.6). `null` is coming back.
 *
 * Only the wire half. The copy that outlives the session is saved on the
 * status by the editor, which is also what stamps `away_since` — the server
 * owns that value and only `PATCH /me` sets it.
 */
export function setAway(message: string | null): void {
  const next = message === null || message.trim() === "" ? message : message.trim();
  if (next === wantAway) return;
  wantAway = next;
  void tick();
}

/** Whether this client currently believes it is away. */
export function isAway(): boolean {
  return wantAway !== null;
}

/**
 * True once the gateway has said `ready`. A fresh `ready` is a new session
 * as far as the server is concerned — we are `around` with no room, and the
 * next tick re-announces if we should still be in one.
 *
 * Being away survives the reconnect. The server forgot — its presence map is
 * per-connection — so `sentAwayMessage` clears and the next tick says it
 * again. A dropped socket is not somebody coming back to their desk.
 */
export function setPresenceLive(next: boolean): void {
  if (next === live) return;
  live = next;
  if (next) {
    sentRoomId = null;
    sentState = "around";
    sentAwayMessage = null;
    void tick();
    return;
  }
  if (timer !== null && typeof window !== "undefined") {
    window.clearTimeout(timer);
    timer = null;
  }
}

/**
 * Start watching the window. Returns a stop function so a signed-out
 * session does not keep sending frames for a connection that is gone.
 */
export function startPresence(): () => void {
  if (attached) return stopPresence;
  attached = true;
  const now = Date.now();
  lastInputAt = now;
  focused = readFocused();
  setLooking(focused);
  blurredAt = focused ? 0 : now;

  if (typeof window === "undefined") return stopPresence;

  const onPointer = (): void => onInput();
  const onKey = (): void => onInput();
  const onWheel = (): void => onInput();
  window.addEventListener("pointerdown", onPointer);
  window.addEventListener("pointermove", onPointer);
  window.addEventListener("keydown", onKey);
  window.addEventListener("wheel", onWheel, { passive: true });
  window.addEventListener("focus", onFocusChange);
  window.addEventListener("blur", onFocusChange);
  document.addEventListener("visibilitychange", onFocusChange);

  unlisten = () => {
    window.removeEventListener("pointerdown", onPointer);
    window.removeEventListener("pointermove", onPointer);
    window.removeEventListener("keydown", onKey);
    window.removeEventListener("wheel", onWheel);
    window.removeEventListener("focus", onFocusChange);
    window.removeEventListener("blur", onFocusChange);
    document.removeEventListener("visibilitychange", onFocusChange);
  };

  void tick();
  return stopPresence;
}

function stopPresence(): void {
  if (!attached) return;
  attached = false;
  live = false;
  openRoomId = null;
  sentRoomId = null;
  sentState = "around";
  sentAwayMessage = null;
  wantAway = null;
  if (timer !== null && typeof window !== "undefined") {
    window.clearTimeout(timer);
    timer = null;
  }
  unlisten?.();
  unlisten = null;
  stopLooking();
}
