/**
 * Whether the window has the user's attention right now.
 *
 * Split out of the presence driver so the notifier can ask without pulling
 * the gateway in a circle (`gateway` → `notify` → here, never back).
 *
 * This is the answer T-305's read-marker and the notifier both need: a room
 * sitting open on a second monitor while you type somewhere else has not
 * been read, and a mention that landed on a screen you are looking at is
 * not worth a notification. It is *not* the 90-second occupancy clock —
 * sitting still while reading is still reading.
 */

let watching = false;
let focused = false;

export function readFocused(): boolean {
  if (typeof document === "undefined") return false;
  return document.hasFocus() && document.visibilityState === "visible";
}

export function isLooking(): boolean {
  return watching ? focused : readFocused();
}

/** The presence driver is the only writer. */
export function setLooking(next: boolean): void {
  watching = true;
  focused = next;
}

export function stopLooking(): void {
  watching = false;
  focused = false;
}
