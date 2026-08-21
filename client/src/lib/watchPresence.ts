/**
 * Watch the window, send `room.focus` and `idle`.
 *
 * The clocks and the deciding live in `presence.ts`. This file is the part
 * that cannot be tested without a window: it listens for input and focus,
 * holds the last thing we sent, and talks to the gateway.
 */
import type { PresenceState } from "../generated/PresenceState";
import type { RoomId } from "../generated/RoomId";
import { send } from "./gateway";
import { readFocused, setLooking, stopLooking } from "./looking";
import { decide, frameFor, nextCheckAt, type PresenceAction, type PresenceClock } from "./presence";

function applySent(
  sent: { roomId: RoomId | null; state: PresenceState },
  action: PresenceAction,
): { roomId: RoomId | null; state: PresenceState } {
  if (action.kind === "focus") {
    return {
      roomId: action.roomId,
      state: action.roomId === null ? "around" : "in_room",
    };
  }
  return { roomId: sent.roomId, state: action.state };
}

let live = false;
let openRoomId: RoomId | null = null;
let sentRoomId: RoomId | null = null;
let sentState: PresenceState = "around";
let focused = false;
let lastInputAt = 0;
let blurredAt = 0;
let timer: number | null = null;
let ticking: Promise<void> | null = null;
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

async function tick(): Promise<void> {
  if (ticking) {
    pending = true;
    return ticking;
  }
  ticking = run();
  return ticking;
}

async function run(): Promise<void> {
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
        const ok = await send(frameFor(action));
        if (!ok) break;
        const next = applySent({ roomId: sentRoomId, state: sentState }, action);
        sentRoomId = next.roomId;
        sentState = next.state;
      }
      schedule();
    } while (pending);
  } finally {
    ticking = null;
    if (pending && live) void tick();
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
 * True once the gateway has said `ready`. A fresh `ready` is a new session
 * as far as the server is concerned — we are `around` with no room, and the
 * next tick re-announces if we should still be in one.
 */
export function setPresenceLive(next: boolean): void {
  if (next === live) return;
  live = next;
  if (next) {
    sentRoomId = null;
    sentState = "around";
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
  if (timer !== null && typeof window !== "undefined") {
    window.clearTimeout(timer);
    timer = null;
  }
  unlisten?.();
  unlisten = null;
  stopLooking();
}
