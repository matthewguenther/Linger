/**
 * Density: Comfortable, Compact, or IRC (SPEC §4.7, §5.6).
 *
 * The mode is one attribute on the root element and every difference between
 * the three is a custom property in `tokens.css` — no second stylesheet, no
 * component that knows which mode it is in. The one exception is grouping,
 * which IRC mode turns off, and that is a decision the row builder makes rather
 * than something CSS can express.
 *
 * This is a preference, not shared state, so it lives in local component state
 * and here — not in the gateway store (AGENTS: local state plus one store).
 */
export type Density = "comfortable" | "compact" | "irc";

export const DENSITIES: readonly Density[] = ["comfortable", "compact", "irc"];

const KEY = "linger.density";

function isDensity(value: string | null): value is Density {
  return value !== null && DENSITIES.some((mode) => mode === value);
}

/** The saved mode, or Comfortable. Reading storage can throw; that is fine. */
export function loadDensity(): Density {
  try {
    const saved = window.localStorage.getItem(KEY);
    return isDensity(saved) ? saved : "comfortable";
  } catch {
    return "comfortable";
  }
}

/** Apply the mode to the document and remember it for next launch. */
export function applyDensity(density: Density): void {
  document.documentElement.dataset.density = density;
  try {
    window.localStorage.setItem(KEY, density);
  } catch {
    // A browser with storage switched off still gets the mode for this session.
  }
}
