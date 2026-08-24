/**
 * The colors a person carries, wherever their name is drawn.
 *
 * Every name in the app — in the stream, in the roster, in a status card —
 * takes its color from the same two custom properties, so a person reads as the
 * same person in all three places. Both point at a *palette variable* rather
 * than a color: M6 generates those variables from `linger-core::PALETTE`, the
 * one place the sixteen colors are defined (SPEC §5.4), so nothing here ever
 * learns what "azure" looks like and everything picks up real colors the moment
 * that stylesheet exists.
 *
 * Weight, italic and the palette key are all this does. Gradients, shimmer and
 * glow are T-601's, and they attach to the same elements.
 */
import type { CSSProperties } from "react";

import type { User } from "../generated/User";

/**
 * The two colors: the 3px rule beside somebody's messages, and their name.
 */
export function personStyle(person: User | undefined): CSSProperties {
  const key = paletteKey(person);
  if (key === null) {
    return { "--person-rule": "var(--hairline-strong)", "--person-name": "var(--text-primary)" };
  }
  return {
    "--person-rule": `var(--name-${key}, var(--hairline-strong))`,
    "--person-name": `var(--name-${key}, var(--text-primary))`,
  };
}

/** Weight and slant, which are the person's own and not the surface's. */
export function nameStyle(person: User | undefined): CSSProperties {
  return {
    fontWeight: person?.style.weight ?? 500,
    fontStyle: person?.style.italic === true ? "italic" : "normal",
  };
}

export function paletteKey(person: User | undefined): string | null {
  if (!person) return null;
  // A gradient name takes its rule from the first of its two colors. Painting
  // the gradient itself is M6's, along with the palette these keys name.
  const fill = person.style.fill;
  const key = fill.kind === "solid" ? fill.color : fill.from;
  // The server validates keys against linger-core::PALETTE (AGENTS rule 8);
  // this is the second lock on the door, because the key is about to become
  // part of a CSS variable name and user content is hostile (ARCHITECTURE §7).
  return /^[a-z]{2,16}$/.test(key) ? key : null;
}
