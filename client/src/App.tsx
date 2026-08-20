/**
 * The top of the client: sign-in first, the Console frame once you're in.
 *
 * The frame is live from the gateway now (T-302) but still thin: the message
 * stream is T-303 and the real roster is T-401. What's real here is the account
 * (T-301) and the connection — who's on the server, what rooms exist, who is
 * around, and a status bar that reports the connection honestly.
 */
import { useEffect, useState } from "react";

import AuthScreens from "./auth/AuthScreens";
import type { ServerInfo } from "./generated/ServerInfo";
import type { User } from "./generated/User";
import type { AuthedApi } from "./lib/api";
import { connect, disconnect, statusText, useGateway } from "./lib/gateway";
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
  const gateway = useGateway();
  const [server, setServer] = useState<ServerInfo | null>(null);

  useEffect(() => {
    void connect(api);
    return () => {
      void disconnect();
    };
  }, [api]);

  useEffect(() => {
    const abort = new AbortController();
    // The server's name is the one thing `ready` doesn't carry. A failure isn't
    // worth a screen of its own: the rail falls back to the hostname.
    void api
      .serverInfo(abort.signal)
      .then(setServer)
      .catch(() => undefined);
    return () => abort.abort();
  }, [api]);

  const rooms = [...gateway.rooms]
    .filter((room) => room.archived_at === null)
    .sort((a, b) => a.position - b.position);
  // Everyone the server has told us about, minus the people who aren't here.
  // T-401 turns this into the card stack; it is a list of names until then.
  const around = gateway.presence
    .filter((entry) => entry.state !== "offline")
    .map((entry) => gateway.users.find((person) => person.id === entry.user_id))
    .filter((person) => person !== undefined);

  const status = statusText(gateway.status);
  const statusDetail = gateway.status.kind === "waiting" ? gateway.status.reason : undefined;

  return (
    <div className="frame">
      <aside className="rail">
        <section className="rail-section">
          <h2 className="panel-label">server</h2>
          <p className="rail-server">{server?.name ?? hostOf(api.baseUrl)}</p>
        </section>
        <section className="rail-section rail-rooms">
          <h2 className="panel-label">rooms</h2>
          {rooms.length === 0 ? (
            <p className="placeholder">—</p>
          ) : (
            <ul className="room-list">
              {rooms.map((room) => (
                <li key={room.id} className="room-item">
                  #{room.slug}
                </li>
              ))}
            </ul>
          )}
        </section>
      </aside>

      <main className="stream">
        <header className="stream-header">
          <span className="room-name">welcome</span>
          <span className="meta">nobody in the room</span>
        </header>
        <div className="stream-body">
          <p className="placeholder">
            You're signed in and connected. The stream itself lands in T-303.
          </p>
        </div>
        <footer className="composer">
          <input
            className="composer-input"
            placeholder="say something"
            disabled
            aria-label="message composer (disabled until the stream lands)"
          />
        </footer>
      </main>

      <aside className="roster">
        <h2 className="panel-label">who’s around</h2>
        {around.length === 0 ? (
          <p className="placeholder">nobody yet</p>
        ) : (
          <ul className="roster-list">
            {around.map((person) => (
              <li key={person.id} className="roster-name">
                {person.display_name}
              </li>
            ))}
          </ul>
        )}
      </aside>

      <footer className="status-bar meta">
        <span title={statusDetail}>{status}</span>
        <span className="status-right">
          <span>{user.display_name}</span>
          {keyringNotice ? <span className="status-warn">not remembered</span> : null}
          <button className="status-action" type="button" onClick={() => void onSignOut()}>
            sign out
          </button>
        </span>
      </footer>
    </div>
  );
}
