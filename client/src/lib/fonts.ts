/**
 * The twelve curated faces, as CSS variable names.
 *
 * `linger-core::FONTS` is the one definition of the set (SPEC §5.7) and the
 * server rejects any `font_key` that is not in it, because client-side
 * validation alone is a defect (AGENTS rule 8). This list is a mirror for the
 * same reason `palette.ts` mirrors the palette keys: the pickers need something
 * to iterate over, and a key this build has never heard of still falls back to
 * the UI face rather than reaching for a font nobody bundled.
 *
 * What each key *looks like* is a client concern — the same way the glyph for a
 * reaction key is — so the family stacks live in `styles/tokens.css` as
 * `--font-<key>`. T-604 fetches, subsets and `@font-face`s the actual files; a
 * key whose face is not installed falls through its stack to the generic at the
 * end, which is why a name is never invisible while that task is open.
 */
export const FONT_KEYS = [
  "geist-sans",
  "geist-mono",
  "ibm-plex-sans",
  "ibm-plex-mono",
  "jetbrains-mono",
  "inter",
  "space-grotesk",
  "commit-mono",
  "newsreader",
  "instrument-serif",
  "departure-mono",
  "silkscreen",
] as const;

export type FontKey = (typeof FONT_KEYS)[number];

export function isFontKey(value: string): value is FontKey {
  return FONT_KEYS.some((key) => key === value);
}

/**
 * The CSS variable a font key names, or the fallback for a key we don't know.
 *
 * The key is about to become part of a variable name and user content is
 * hostile (ARCHITECTURE §7), so it is matched against the list rather than
 * interpolated on trust.
 */
export function fontVar(key: string | null | undefined, fallback: string): string {
  return typeof key === "string" && isFontKey(key) ? `var(--font-${key}, ${fallback})` : fallback;
}
