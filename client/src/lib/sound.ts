/**
 * The sound player. There is exactly one, and this is it.
 *
 * Linger makes a noise in two places: a knock (SPEC §4.9, T-1102) and, when
 * T-901 lands, entrance sounds (SPEC §4.1). This file is deliberately the
 * smaller half of that — the *gate* and the playback, with no library, no
 * asset pipeline and no per-user state. **T-901 extends this file rather than
 * writing a second one.** What it adds is a source of bundled `.opus` files,
 * a per-listener 5-minute cooldown, and per-user mute; what it inherits is
 * everything below.
 *
 * The gate is SPEC §4.1's, and it applies to knocks too:
 *
 * - **Global mute.** Off by default; one switch in settings.
 * - **Quiet hours, 22:00–08:00 in the listener's own time, default on.** The
 *   listener's clock, never the sender's — 2am for you is what matters, and
 *   somebody knocking from another timezone does not get to decide that.
 *
 * Both are the reader's preference about their own machine, so they live in
 * local storage beside density and theme rather than in the gateway store.
 *
 * The knock itself is synthesized rather than played from a file. Two soft
 * taps out of an oscillator is about twenty lines and no bytes, and it means
 * this task does not have to reach into T-903's curation to ship a sound.
 */

/** Quiet hours run from 22:00 to 08:00, listener-local (SPEC §4.1). */
export const QUIET_FROM_HOUR = 22;
export const QUIET_UNTIL_HOUR = 8;

const MUTE_KEY = "linger.sound.muted";
const QUIET_KEY = "linger.sound.quietHours";

/** What the listener has decided about noise on this computer. */
export interface SoundPrefs {
  /** Nothing makes a sound. Off by default. */
  muted: boolean;
  /** Nothing makes a sound between 22:00 and 08:00. **On** by default. */
  quietHours: boolean;
}

/** Whether `at` falls inside quiet hours. Wraps midnight, hence the `||`. */
export function inQuietHours(at: Date): boolean {
  const hour = at.getHours();
  return hour >= QUIET_FROM_HOUR || hour < QUIET_UNTIL_HOUR;
}

/**
 * Whether a sound may be played right now. Pure, so the rule can be tested
 * without a clock, an audio device or a browser.
 */
export function soundAllowed(prefs: SoundPrefs, at: Date): boolean {
  if (prefs.muted) return false;
  return !(prefs.quietHours && inQuietHours(at));
}

/**
 * The saved preferences, or the defaults. Quiet hours default **on**: a
 * product that wakes people up at 3am has to be opted into, not out of.
 */
export function loadSoundPrefs(): SoundPrefs {
  try {
    return {
      muted: window.localStorage.getItem(MUTE_KEY) === "true",
      quietHours: window.localStorage.getItem(QUIET_KEY) !== "false",
    };
  } catch {
    return { muted: false, quietHours: true };
  }
}

/** Remember them for next launch. Storage can be switched off; that is fine. */
export function saveSoundPrefs(prefs: SoundPrefs): void {
  try {
    window.localStorage.setItem(MUTE_KEY, String(prefs.muted));
    window.localStorage.setItem(QUIET_KEY, String(prefs.quietHours));
  } catch {
    // The setting still holds for this session.
  }
}

/**
 * One `AudioContext` for the app, made on the first sound and kept.
 *
 * Contexts are a limited resource in every engine — making one per knock is
 * how a browser eventually refuses to make any. `null` means there is no audio
 * here at all (a test runner, a headless session), which is not an error.
 */
let context: AudioContext | null = null;

function audio(): AudioContext | null {
  if (typeof window === "undefined" || typeof window.AudioContext !== "function") return null;
  context ??= new window.AudioContext();
  return context;
}

/**
 * One knuckle on a door: a low sine struck hard and damped fast, through a
 * lowpass so it reads as wood rather than as a beep.
 */
function tap(ctx: AudioContext, at: number, gain: number): void {
  const osc = ctx.createOscillator();
  const level = ctx.createGain();
  const wood = ctx.createBiquadFilter();

  osc.type = "sine";
  osc.frequency.setValueAtTime(180, at);
  // The pitch drop is most of what makes it sound struck rather than played.
  osc.frequency.exponentialRampToValueAtTime(90, at + 0.08);

  wood.type = "lowpass";
  wood.frequency.setValueAtTime(700, at);

  level.gain.setValueAtTime(gain, at);
  level.gain.exponentialRampToValueAtTime(0.0001, at + 0.09);

  osc.connect(wood).connect(level).connect(ctx.destination);
  osc.start(at);
  osc.stop(at + 0.1);
}

/**
 * Somebody knocked. Two taps, quietly, if the listener is letting sounds
 * through right now.
 *
 * Returns whether it made a noise, which is what the caller needs in order to
 * be honest in a test. Never throws: a machine with no audio device still gets
 * the card.
 */
export function playKnock(now: Date = new Date()): boolean {
  if (!soundAllowed(loadSoundPrefs(), now)) return false;
  const ctx = audio();
  if (ctx === null) return false;
  try {
    // A context can start suspended until the page has been interacted with.
    // Nothing to wait for — the taps are scheduled against its clock either
    // way, and by the time anybody knocks the app has been clicked.
    void ctx.resume();
    const at = ctx.currentTime + 0.01;
    tap(ctx, at, 0.16);
    tap(ctx, at + 0.14, 0.12);
    return true;
  } catch {
    return false;
  }
}
