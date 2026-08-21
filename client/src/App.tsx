/**
 * The top of the client: sign-in first, the Console frame once you're in.
 *
 * The frame is [rail | stream | roster] over a permanent status bar, and the
 * roster is the point of it (SPEC §3): people get the right-hand panel, not a
 * gutter. On a window too narrow for three columns the roster moves into the
 * stream column as a horizontal strip above the composer — it is never hidden
 * and it never becomes a menu, so `Stream` takes it as a slot.
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
import { useNarrow } from "./lib/layout";
import { hostOf } from "./lib/link";
import { personStyle } from "./lib/names";
import { occupancyLine, occupantsOf, STACK_VISIBLE } from "./lib/occupancy";
import { useSession } from "./lib/session";
import { setPresenceLive, setPresenceRoom, startPresence } from "./lib/watchPresence";
import { resetNotifications, setViewing } from "./notify/notify";
import Roster from "./roster/Roster";
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
  const narrow = useNarrow();

  useEffect(() => {
    applyDensity(density);
  }, [density]);

  useEffect(() => {
    void connect(api);
    const stop = startPresence();
    return () => {
      stop();
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
  // Land in the first room, and don't hold a room that was archived or that
  // this account can no longer see.
  const open = rooms.find((room) => room.id === openRoomId) ?? rooms[0] ?? null;

  // Nothing interrupts you about the room you are already reading.
  useEffect(() => {
    setViewing(open?.id ?? null);
    setPresenceRoom(open?.id ?? null);
  }, [open?.id]);

  // A fresh `ready` is a new session on the server: we are `around` until
  // this clock re-announces the room. Anything short of ready is not a
  // connection worth sending presence on.
  useEffect(() => {
    setPresenceLive(gateway.status.kind === "ready");
  }, [gateway.status.kind]);

  const status = statusText(gateway.status);
  const statusDetail = gateway.status.kind === "waiting" ? gateway.status.reason : undefined;

  // One roster, in one of two places. Rendering it twice and hiding one would
  // mean two of everything it holds — two open cards, two scroll positions.
  const roster = <Roster api={api} rooms={rooms} layout={narrow ? "strip" : "column"} />;

  return (
    <div className="frame" data-narrow={narrow ? "true" : undefined}>
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
                    <span className="room-slug">#{room.slug}</span>
                    <RoomStack
                      people={occupantsOf(
                        room.id,
                        gateway.occupancy,
                        gateway.presence,
                        gateway.users,
                      )}
                    />
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
          {narrow ? roster : null}
        </main>
      ) : (
        <Stream
          api={api}
          room={open}
          users={gateway.users}
          density={density}
          onDensityChange={setDensity}
          roster={narrow ? roster : undefined}
        />
      )}

      {narrow ? null : roster}

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

/**
 * The small stack of who is in a room, on the rail (SPEC §4.1).
 *
 * Dots, not faces: there are no avatars in this app, and the rail is not
 * allowed colored icon squares either. Five is as many as the column will
 * hold without crowding the name; the rest live in the accessible label,
 * never as a "+N".
 */
function RoomStack({ people }: { people: User[] }) {
  if (people.length === 0) return null;
  const visible = people.slice(0, STACK_VISIBLE);
  const label = `${occupancyLine(people)} in the room`;
  return (
    <span className="room-stack" aria-label={label} title={occupancyLine(people)}>
      {visible.map((person) => (
        <span
          key={person.id}
          className="room-stack-dot"
          style={personStyle(person)}
          aria-hidden="true"
        />
      ))}
    </span>
  );
}
