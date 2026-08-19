/**
 * The M0 shell: proves the Console layout (SPEC §3, §5), the token system, and
 * the ts-rs type pipeline end to end. M3 replaces the placeholder panes with the
 * real message stream, rail, and roster — the frame itself is the design.
 */
import type { PresenceState } from "./generated/PresenceState";
import "./app.css";

// Wire types come only from linger-core via ts-rs (client/src/generated/).
const presence: PresenceState = "offline";

export default function App() {
  return (
    <div className="frame">
      <aside className="rail">
        <section className="rail-section">
          <h2 className="panel-label">stoops</h2>
          <p className="placeholder">no stoop yet</p>
        </section>
        <section className="rail-section rail-rooms">
          <h2 className="panel-label">rooms</h2>
          <p className="placeholder">—</p>
        </section>
      </aside>

      <main className="stream">
        <header className="stream-header">
          <span className="room-name">welcome</span>
          <span className="meta">nobody sitting in</span>
        </header>
        <div className="stream-body">
          <p className="placeholder">
            This is a stoop with the lights off. M1–M3 turn them on.
          </p>
        </div>
        <footer className="composer">
          <input
            className="composer-input"
            placeholder="say something"
            disabled
            aria-label="message composer (disabled until connected)"
          />
        </footer>
      </main>

      <aside className="roster">
        <h2 className="panel-label">who’s around</h2>
        <p className="placeholder">nobody yet</p>
      </aside>

      <footer className="status-bar meta">
        <span>{presence}</span>
        <span>linger-desktop/0.1.0</span>
      </footer>
    </div>
  );
}
