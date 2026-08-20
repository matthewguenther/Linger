/**
 * The top of the client: sign-in first, the Console frame once you're in.
 *
 * The frame is still mostly placeholders — T-302 brings the live gateway and
 * T-303 the real message stream. What's real here is the account: who you are,
 * which server you're on, and staying signed in across restarts (T-301).
 */
import { useEffect, useState } from "react";

import AuthScreens from "./auth/AuthScreens";
import type { PresenceState } from "./generated/PresenceState";
import type { ServerInfo } from "./generated/ServerInfo";
import type { User } from "./generated/User";
import type { AuthedApi } from "./lib/api";
import { hostOf } from "./lib/link";
import { useSession } from "./lib/session";
import "./app.css";

export default function App() {
  const session = useSession();

  if (session.state.status === "restoring") {
    return (
      <div className="auth">
        <p className="meta">signing you back in…</p>
      </div>
    );
  }

  if (session.state.status === "signed_out") {
    return (
      <AuthScreens
        notice={session.notice}
        keyringNotice={session.keyringNotice}
        onAuthenticated={session.signIn}
      />
    );
  }

  return (
    <Console
      api={session.state.api}
      user={session.state.user}
      keyringNotice={session.keyringNotice}
      onSignOut={session.signOut}
    />
  );
}

function Console({
  api,
  user,
  keyringNotice,
  onSignOut,
}: {
  api: AuthedApi;
  user: User;
  keyringNotice: string | null;
  onSignOut: () => Promise<void>;
}) {
  const [server, setServer] = useState<ServerInfo | null>(null);

  useEffect(() => {
    const abort = new AbortController();
    // A failure here is not worth a screen of its own: the rail falls back to
    // the hostname, and the gateway (T-302) will report a real connection state.
    void api
      .serverInfo(abort.signal)
      .then(setServer)
      .catch(() => undefined);
    return () => abort.abort();
  }, [api]);

  // Real presence arrives with the gateway in T-302.
  const presence: PresenceState = "offline";

  return (
    <div className="frame">
      <aside className="rail">
        <section className="rail-section">
          <h2 className="panel-label">server</h2>
          <p className="rail-server">{server?.name ?? hostOf(api.baseUrl)}</p>
        </section>
        <section className="rail-section rail-rooms">
          <h2 className="panel-label">rooms</h2>
          <p className="placeholder">—</p>
        </section>
      </aside>

      <main className="stream">
        <header className="stream-header">
          <span className="room-name">welcome</span>
          <span className="meta">nobody in the room</span>
        </header>
        <div className="stream-body">
          <p className="placeholder">
            You're signed in. The stream itself lands in T-303.
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
        <span>
          {presence} · {user.display_name}
        </span>
        <span className="status-right">
          {keyringNotice ? <span className="status-warn">not remembered</span> : null}
          <button className="status-action" type="button" onClick={() => void onSignOut()}>
            sign out
          </button>
        </span>
      </footer>
    </div>
  );
}
