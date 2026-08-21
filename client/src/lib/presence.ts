/**
 * Being in a room, and the two clocks that take you out of it.
 *
 * SPEC §4.1: focusing the app on a room means you are in the room. Other
 * people see this. Backgrounding the app, or sitting still for 90 seconds,
 * takes you out. SPEC §4.3: no input for ten minutes is `idle`.
 *
 * Pure, and takes the clock as arguments, so the 90-second and ten-minute
 * edges can be tested without waiting for either. The driver that reads the
 * window and sends the frames is `watchPresence.ts`.
 *
 * `away` is T-405's. This file will not send a state over the top of it.
 */
import type { ClientFrame } from "../generated/ClientFrame";
import type { PresenceState } from "../generated/PresenceState";
import type { RoomId } from "../generated/RoomId";

/** Backgrounding or sitting still for this long takes you out of the room. */
export const LEAVE_AFTER_MS = 90_000;

/** No input for this long is `idle` (SPEC §4.3). */
export const IDLE_AFTER_MS = 10 * 60_000;

/** Everything `decide` needs, and nothing it would have to go and fetch. */
export interface PresenceClock {
  /** The window has the user's attention right now. */
  focused: boolean;
  /** Last pointer, key, wheel, or focus. Not blur — that is `blurredAt`. */
  lastInputAt: number;
  /** When the window last lost focus. Ignored while `focused` is true. */
  blurredAt: number;
  /** The room on screen, if any. */
  roomId: RoomId | null;
  /** The last `room.focus` we actually got onto the wire. */
  sentRoomId: RoomId | null;
  /** The last presence state we sent, or that `ready` gave us. */
  sentState: PresenceState;
}

export type PresenceAction =
  | { kind: "focus"; roomId: RoomId | null }
  | { kind: "state"; state: "idle" | "around" };

/**
 * Whether the 90-second clocks still say this person is here.
 *
 * Two clocks, because the spec names two ways of leaving: sitting still, and
 * putting the app in the background. Either one expiring is enough.
 */
export function stillHere(clock: PresenceClock, now: number): boolean {
  if (now - clock.lastInputAt >= LEAVE_AFTER_MS) return false;
  if (!clock.focused && now - clock.blurredAt >= LEAVE_AFTER_MS) return false;
  return true;
}

/** The room we should be in, or `null` for not in one. */
export function wantedRoom(clock: PresenceClock, now: number): RoomId | null {
  if (clock.roomId === null || !stillHere(clock, now)) return null;
  return clock.roomId;
}

/**
 * The next frame to send, or `null` if the wire already matches.
 *
 * Room membership is `room.focus`. Idle is `presence.update`. The server
 * sets `in_room` / `around` from the former, so this never sends those as a
 * state of their own — doing so would either no-op or fight the room.
 *
 * `away` is left alone: T-405 owns that word, and overwriting it with `idle`
 * because somebody's hands were off the keyboard would be this file reaching
 * into a different task.
 */
export function decide(clock: PresenceClock, now: number): PresenceAction | null {
  if (clock.sentState === "away") return null;

  const want = wantedRoom(clock, now);
  if (want !== clock.sentRoomId) return { kind: "focus", roomId: want };

  const idle = now - clock.lastInputAt >= IDLE_AFTER_MS;
  if (idle && clock.sentState !== "idle") return { kind: "state", state: "idle" };
  if (!idle && clock.sentState === "idle") return { kind: "state", state: "around" };
  return null;
}

/**
 * When `decide` will next have something new to say, if nobody touches the
 * window. `null` means it is waiting on input, not on the clock.
 */
export function nextCheckAt(clock: PresenceClock, now: number): number | null {
  const times: number[] = [];
  const leaveByInput = clock.lastInputAt + LEAVE_AFTER_MS;
  if (leaveByInput > now) times.push(leaveByInput);
  if (!clock.focused) {
    const leaveByBlur = clock.blurredAt + LEAVE_AFTER_MS;
    if (leaveByBlur > now) times.push(leaveByBlur);
  }
  const idleAt = clock.lastInputAt + IDLE_AFTER_MS;
  if (idleAt > now) times.push(idleAt);
  if (times.length === 0) return null;
  return Math.min(...times);
}

export function frameFor(action: PresenceAction): ClientFrame {
  if (action.kind === "focus") {
    return { op: "room.focus", d: { room_id: action.roomId } };
  }
  return {
    op: "presence.update",
    d: { state: action.state, activity: null, away_message: null },
  };
}
