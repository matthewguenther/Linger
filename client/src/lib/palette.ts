/**
 * The sixteen palette keys, in picker order.
 *
 * `linger-core::PALETTE` is the one definition of the palette (SPEC §5.4) and
 * the server validates every key it is sent against it, because client-side
 * validation alone is a defect (AGENTS rule 8). This list is a mirror kept for
 * the pickers to iterate over, the same way `Stream.tsx` mirrors
 * `MAX_MESSAGE_CHARS`: the server stays the authority, and a key this build has
 * never heard of is still refused there.
 *
 * It holds *keys*, never colors. What "azure" looks like is a CSS custom
 * property M7 generates from the Rust table, so nothing in the frontend has
 * ever seen a hex value (AGENTS rule 12).
 */
export const PALETTE_KEYS = [
  "ember",
  "rust",
  "amber",
  "brass",
  "lime",
  "fern",
  "mint",
  "teal",
  "cyan",
  "sky",
  "azure",
  "indigo",
  "violet",
  "orchid",
  "rose",
  "slate",
] as const;

export type PaletteKey = (typeof PALETTE_KEYS)[number];

export function isPaletteKey(value: string): value is PaletteKey {
  return PALETTE_KEYS.some((key) => key === value);
}

/**
 * The CSS variable a palette key names, with a fallback for right now.
 *
 * `--name-*` does not exist until M7 emits `palette.generated.css` from
 * `linger-core::palette::css_variables`. Until it does, every one of these
 * resolves to the fallback — which is why nothing in this app carries meaning
 * in color alone.
 */
export function colorVar(key: string, fallback: string): string {
  return isPaletteKey(key) ? `var(--name-${key}, ${fallback})` : fallback;
}
