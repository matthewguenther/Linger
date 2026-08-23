/**
 * Comfortable / Compact / IRC (SPEC §4.7, §5.6).
 *
 * The same control lives in the room header (you change how you are reading
 * while you are reading) and in settings (so you can find it without a room).
 * One component, so the labels cannot drift.
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
