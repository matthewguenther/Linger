/**
 * Numbers the server also knows.
 *
 * These mirror `linger-core::limits`, which is the source of truth — the server
 * enforces all of them and the client only uses them to be a better citizen: to
 * stop a message at the length the server would refuse anyway, and to avoid
 * spending a rate-limit slot it knows isn't there.
 *
 * `ts-rs` exports types, not constants, so these are written out by hand. A
 * test in `limits.test.ts` reads the Rust source and fails if they drift.
 */

/** Message body cap in characters, after trimming (PROTOCOL §4). */
export const MAX_MESSAGE_CHARS = 8000;

/**
 * The server accepts one `typing.start` per room per 4 seconds. We leave half a
 * second of headroom so a burst of keystrokes never spends a refusal.
 */
export const TYPING_INTERVAL_MS = 4500;

/**
 * How long someone stays "typing" after their last frame. Longer than the
 * 4s heartbeat between frames, so a steady typist doesn't flicker, and short
 * enough that somebody who wandered off stops claiming to be mid-sentence.
 */
export const TYPING_TTL_MS = 7000;
