/**
 * A textarea that is exactly as tall as what is in it.
 *
 * `rows` counts the newlines somebody typed, which is the wrong number the
 * moment a long line wraps. The browser already knows the right one — it is
 * `scrollHeight` — but only once the box has been allowed to shrink first, so
 * this sets the height to `auto` and reads it back before setting it for real.
 */
import { type RefObject, useLayoutEffect } from "react";

/** Past this the box stops growing and scrolls, so it can never eat the room. */
const CEILING_PX = 220;

export function useAutoGrow(field: RefObject<HTMLTextAreaElement | null>, value: string): void {
  useLayoutEffect(() => {
    const element = field.current;
    if (!element) return;
    element.style.height = "auto";
    element.style.height = `${Math.min(element.scrollHeight, CEILING_PX)}px`;
  }, [field, value]);
}
