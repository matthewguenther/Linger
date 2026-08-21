/**
 * The twelve reactions (SPEC §4.8).
 *
 * The keys are the wire contract and live in `linger-core::REACTIONS`; the
 * server rejects anything else. What each key *looks like* is a client concern
 * (PROTOCOL §4), which is what this file is: key → glyph, and key → the word a
 * screen reader says.
 *
 * The list is written out here rather than generated, because `ts-rs` exports
 * types and this is a value. A test in `reactions.test.ts` reads the Rust source
 * and fails if the two ever disagree, so the copy can't drift silently. A key
 * we don't recognise still renders — as its own name — because a newer server
 * adding one should not leave a hole in a message.
 *
 * **No numbers.** Six people hitting the same reaction make a denser, larger
 * mark, not `fire 6`. `weightOf` is that mark's step. The count exists on the
 * wire for accessibility labels and hover, and that is the only place it goes.
 */

export interface Reaction {
  key: string;
  glyph: string;
  /** The spoken name, used in `aria-label` and the hover title. */
  label: string;
}

/** In `linger-core::REACTIONS` order, which is the order the picker shows. */
export const REACTIONS: readonly Reaction[] = [
  { key: "heart", glyph: "❤️", label: "heart" },
  { key: "laugh", glyph: "😂", label: "laugh" },
  { key: "wow", glyph: "😮", label: "wow" },
  { key: "cry", glyph: "😢", label: "cry" },
  { key: "fire", glyph: "🔥", label: "fire" },
  { key: "skull", glyph: "💀", label: "skull" },
  { key: "up", glyph: "👍", label: "thumbs up" },
  { key: "down", glyph: "👎", label: "thumbs down" },
  { key: "eyes", glyph: "👀", label: "eyes" },
  { key: "clap", glyph: "👏", label: "clap" },
  { key: "hundred", glyph: "💯", label: "a hundred" },
  { key: "sparkles", glyph: "✨", label: "sparkles" },
];

const BY_KEY = new Map(REACTIONS.map((reaction) => [reaction.key, reaction]));

/** What to draw for a key, including one this build has never heard of. */
export function reactionFor(key: string): Reaction {
  return BY_KEY.get(key) ?? { key, glyph: key, label: key };
}

/**
 * How heavy a reaction's mark is: 0 for one person, 4 for a room full.
 *
 * The steps are uneven on purpose. The difference between one person and two is
 * the one worth seeing at a glance; past about eight, more people can only make
 * it a little heavier, or a busy message would be all mark and no message.
 */
export function weightOf(count: number): number {
  if (count <= 1) return 0;
  if (count === 2) return 1;
  if (count <= 4) return 2;
  if (count <= 7) return 3;
  return 4;
}
