/**
 * Comfortable / Compact / IRC (SPEC §4.7, §5.6).
 *
 * It lives in settings and nowhere else (T-904, 2026-08-31). It used to sit in
 * the room header as well, on the theory that you change how you are reading
 * while you are reading — but nobody does. Somebody picks a density in their
 * first week and then keeps it, so a permanent control was three words of
 * chrome above every conversation, paid for on every screen, for a choice made
 * once.
 */
import { DENSITIES, type Density } from "./density";

export default function DensityPicker({
  density,
  onChange,
}: {
  density: Density;
  onChange: (density: Density) => void;
}) {
  return (
    <div className="density" role="group" aria-label="density">
      {DENSITIES.map((mode) => (
        <button
          key={mode}
          type="button"
          className="density-option meta"
          aria-pressed={mode === density}
          onClick={() => onChange(mode)}
        >
          {mode}
        </button>
      ))}
    </div>
  );
}
