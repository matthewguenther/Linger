/**
 * The one window-width decision the app makes.
 *
 * The three panels have real minimum widths (SPEC §5.5): a 200px rail, a
 * stream that stops shrinking at 420px, and a 240px roster. Under about 880px
 * they stop fitting side by side, and the roster becomes a horizontal strip
 * above the composer instead — **never a hamburger, never hidden** (SPEC §3).
 * It is the panel the product is about; it does not get folded away.
 *
 * The number lives here rather than in a stylesheet because the decision is
 * *where the roster is rendered*, not how it is painted, and only one of those
 * is something CSS can do. The frame carries the answer as `data-narrow` so the
 * stylesheet can follow along without a second copy of the breakpoint.
 */
import { useSyncExternalStore } from "react";

/** 200 rail + 420 stream + 240 roster, with a little slack. */
export const NARROW_MAX_PX = 880;

const QUERY = `(max-width: ${NARROW_MAX_PX}px)`;

let media: MediaQueryList | null = null;

function query(): MediaQueryList | null {
  if (typeof window === "undefined" || !window.matchMedia) return null;
  media ??= window.matchMedia(QUERY);
  return media;
}

function subscribe(changed: () => void): () => void {
  const held = query();
  if (!held) return () => undefined;
  held.addEventListener("change", changed);
  return () => held.removeEventListener("change", changed);
}

/** True when the window is too narrow for the roster to be a column. */
export function useNarrow(): boolean {
  return useSyncExternalStore(
    subscribe,
    () => query()?.matches ?? false,
    () => false,
  );
}
