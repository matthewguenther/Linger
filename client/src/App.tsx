/**
 * The top of the client: sign-in first, the Console frame once you're in.
 *
 * The frame is real now. The account is T-301, the live connection is T-302,
 * and the message stream is T-303 — pick a room in the rail and you are reading
 * and writing in it. The roster on the right is still a plain list of names;
 * the card stack that makes an empty server feel occupied is T-401.
 *
 * The rail is where SPEC §4.2's other half lives: a room holding something you
 * have not seen changes *weight*, and nothing else. No number, no dot, no
 * color. It is one line of CSS and it is the whole feature.
 */
import { useEffect, useState } from "react";

import AuthScreens from "./auth/AuthScreens";
import type { RoomId } from "./generated/RoomId";
import type { ServerInfo } from "./generated/ServerInfo";
import type { User } from "./generated/User";
import type { AuthedApi } from "./lib/api";
import { applyDensity, type Density, loadDensity } from "./lib/density";
import {
  connect,
  disconnect,
  hasNewActivity,
  loadNotifyRules,
  loadReadMarkers,
  statusText,
  useGateway,
} from "./lib/gateway";
import { hostOf } from "./lib/link";
import { useSession } from "./lib/session";
import NotifyRules from "./notify/NotifyRules";
import { resetNotifications, setViewing } from "./notify/notify";
import Stream from "./stream/Stream";
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
  const [openRoomId, setOpenRoomId] = useState<RoomId | null>(null);
  const [density, setDensity] = useState<Density>(loadDensity);
  const [notifying, setNotifying] = useState(false);

  useEffect(() => {
    applyDensity(density);
  }, [density]);

  useEffect(() => {
    void connect(api);
    return () => {
      resetNotifications();
      void disconnect();
    };
  }, [api]);

  // Where you had got to, and who you asked to hear from. Both are small, both
  // are needed before the first frame is judged worth interrupting anyone for,
  // and neither is worth a screen of its own if it fails.
  useEffect(() => {
    void loadReadMarkers(api);
    void loadNotifyRules(api).catch(() => undefined);
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

  // Land in the first room, and don't hold a room that was archived or that
  // this account can no longer see.
  const open = rooms.find((room) => room.id === openRoomId) ?? rooms[0] ?? null;

  // Nothing interrupts you about the room you are already reading.
  useEffect(() => {
    setViewing(open?.id ?? null);
  }, [open?.id]);

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
                <li key={room.id}>
                  <button
                    type="button"
                    className="room-item"
                    aria-current={room.id === open?.id ? "true" : undefined}
                    // The entire "there is something here" signal (SPEC §4.2).
                    // A boolean, on purpose: there is nothing to count and no
                    // endpoint that would answer if there were.
                    data-new={hasNewActivity(gateway, room.id) ? "true" : undefined}
                    onClick={() => setOpenRoomId(room.id)}
                  >
                    #{room.slug}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>
      </aside>

      {open === null ? (
        <main className="stream">
          <header className="stream-header">
            <span className="room-name">welcome</span>
          </header>
          <div className="stream-body">
            <p className="placeholder">
              {gateway.status.kind === "ready"
                ? "This server has no rooms yet."
                : "Connecting to the server…"}
            </p>
          </div>
        </main>
      ) : (
        <Stream
          api={api}
          room={open}
          users={gateway.users}
          density={density}
          onDensityChange={setDensity}
        />
      )}

      {/* The right-hand column has two modes. The roster is what it is for;
          the notify rules borrow it because they are a list of people too, and
          a settings screen for one setting would be a screen too many. */}
      <aside className="roster">
        <div className="roster-head">
          <h2 className="panel-label">{notifying ? "notify me when" : "who’s around"}</h2>
          <button
            type="button"
            className="roster-switch meta"
            aria-expanded={notifying}
            onClick={() => setNotifying((held) => !held)}
          >
            {notifying ? "done" : "notify"}
          </button>
        </div>
        {notifying ? (
          <NotifyRules api={api} rooms={rooms} />
        ) : around.length === 0 ? (
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
