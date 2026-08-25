/**
 * The top of the client: sign-in first, the Console frame once you're in.
 *
 * The frame is [rail | stream | roster] over a permanent status bar, and the
 * roster is the point of it (SPEC §3): people get the right-hand panel, not a
 * gutter. On a window too narrow for three columns the roster moves into the
 * stream column as a horizontal strip above the composer — it is never hidden
 * and it never becomes a menu, so `Stream` takes it as a slot.
 *
 * The rail starts with the server list (SPEC §3, T-412): a live dot per server,
 * a mark when one is holding something you have not read, and `+ add` for the
 * next one. You can be signed into several at once. Each has its own connection,
 * its own people and its own rooms, and switching between them takes the stream,
 * the roster and your presence with it — you are only ever standing in one room.
 *
 * Below that, the rail is where the host's own controls hang: `+ room` beside
 * the room list and `manage` beside the server's name. They are *absent* for
 * everybody else rather than greyed out — a disabled control is a permission
 * matrix drawn in CSS, and this product refuses to have one. Host or member is
 * decided per server: you can host one and be a guest on the next.
 *
 * `you` is the other door: display name, password, density, sign out. It is
 * drawn for everybody, because those are yours, not the host's. The panel
 * takes the stream column the same way `manage` does (T-411).
 *
 * The rail is where SPEC §4.2's other half lives: a room holding something you
 * have not seen changes *weight*, and nothing else. No number, no dot, no
 * color. It is one line of CSS and it is the whole feature.
 */
import { type CSSProperties, useCallback, useEffect, useState } from "react";

import AuthScreens from "./auth/AuthScreens";
import type { AuthResponse } from "./generated/AuthResponse";
import type { RoomId } from "./generated/RoomId";
import type { ServerInfo } from "./generated/ServerInfo";
import type { User } from "./generated/User";
import HostPanel, { type HostSection } from "./host/HostPanel";
import { applyDensity, type Density, loadDensity } from "./lib/density";
import SettingsPanel from "./settings/SettingsPanel";
import { noRoomsBody, noRoomsRail } from "./settings/copy";
import {
  anyNewActivity,
  connect,
  disconnect,
  type GatewayState,
  hasNewActivity,
  loadNotifyRules,
  loadReadMarkers,
  statusText,
  useGateway,
  useServers,
} from "./lib/gateway";
import { useNarrow } from "./lib/layout";
import { hostOf } from "./lib/link";
import { personStyle } from "./lib/names";
import { occupancyLine, occupantsOf, STACK_VISIBLE } from "./lib/occupancy";
import { colorVar } from "./lib/palette";
import { type ServerSession, useSessions } from "./lib/session";
import { dropPresence, setPresenceLive, setPresenceRoom, startPresence } from "./lib/watchPresence";
import { forgetNotifications, resetNotifications, setViewing } from "./notify/notify";
import Roster from "./roster/Roster";
import Stream from "./stream/Stream";
import "./app.css";

export default function App() {
  const sessions = useSessions();

  if (sessions.state.status === "restoring") {
    return (
      <div className="auth">
        <p className="meta">signing you back in…</p>
      </div>
    );
  }

  // Destructured rather than length-checked so the type says what the frame
  // relies on: there is always a server to be looking at.
  const [first, ...rest] = sessions.state.servers;
  if (first === undefined) {
    return (
      <AuthScreens
        notice={sessions.notice}
        keyringNotice={sessions.keyringNotice}
        onAuthenticated={sessions.addServer}
      />
    );
  }

  return (
    <Console
      servers={[first, ...rest]}
      keyringNotice={sessions.keyringNotice}
      onSignOut={sessions.signOut}
      onAddServer={sessions.addServer}
    />
  );
}

/**
 * One server's connection, as a component.
 *
 * Connecting in an effect keyed on the sign-in is what makes adding and
 * removing a server clean: React mounts one of these per server and unmounts
 * it when the server goes, so the socket, the pending notifications and the
 * presence record all die with it. Doing the same thing in a loop over the
 * list would reconnect every server whenever any one of them changed.
 *
 * It draws nothing. The server's name is the one thing `ready` does not carry,
 * so it is fetched here and handed up for the rail to draw.
 */
function ServerLink({
  session,
  onInfo,
}: {
  session: ServerSession;
  onInfo: (baseUrl: string, info: ServerInfo | null) => void;
}) {
  const { api, baseUrl } = session;

  useEffect(() => {
    void connect(api);
    return () => {
      forgetNotifications(baseUrl);
      dropPresence(baseUrl);
      void disconnect(baseUrl);
    };
  }, [api, baseUrl]);

  // Where you had got to, and who you asked to hear from. Both are small, both
  // are needed before the first frame is judged worth interrupting anyone for,
  // and neither is worth a screen of its own if it fails.
  useEffect(() => {
    void loadReadMarkers(api);
    void loadNotifyRules(api).catch(() => undefined);
  }, [api]);

  useEffect(() => {
    const abort = new AbortController();
    // A failure isn't worth a screen of its own: the rail falls back to the
    // hostname.
    void api
      .serverInfo(abort.signal)
      .then((info) => onInfo(baseUrl, info))
      .catch(() => undefined);
    return () => {
      abort.abort();
      onInfo(baseUrl, null);
    };
  }, [api, baseUrl, onInfo]);

  return null;
}

function Console({
  servers,
  keyringNotice,
  onSignOut,
  onAddServer,
}: {
  /** At least one, always. The frame has nothing to draw without it. */
  servers: [ServerSession, ...ServerSession[]];
  keyringNotice: string | null;
  onSignOut: (baseUrl: string) => Promise<void>;
  onAddServer: (baseUrl: string, auth: AuthResponse) => Promise<void>;
}) {
  const all = useServers();
  const [activeUrl, setActiveUrl] = useState(servers[0].baseUrl);
  // Each server's own name, once it has answered. Keyed by base URL like
  // everything else about a server.
  const [info, setInfo] = useState<Record<string, ServerInfo>>({});
  // Which room you were reading on each server, so switching back returns you
  // to where you were rather than to the top of its list.
  const [openRoomIds, setOpenRoomIds] = useState<Record<string, RoomId>>({});
  const [density, setDensity] = useState<Density>(loadDensity);
  // Which host surface is open over the stream, if any (T-410).
  const [hostSection, setHostSection] = useState<HostSection | null>(null);
  // The member's own settings (T-411). Mutually exclusive with the host panel:
  // both take the stream column, and two overlays is a modal stack.
  const [settingsOpen, setSettingsOpen] = useState(false);
  // The paste box, reached from inside the app rather than only before
  // sign-in (T-301's screen, T-412's door to it).
  const [addingServer, setAddingServer] = useState(false);
  const narrow = useNarrow();

  // A server that has gone — signed out of, or refused — must not leave the
  // frame pointing at nothing.
  const active = servers.find((server) => server.baseUrl === activeUrl) ?? servers[0];
  const api = active.api;
  const server = info[active.baseUrl] ?? null;
  const gateway = useGateway(active.baseUrl);

  const closePanels = (): void => {
    setHostSection(null);
    setSettingsOpen(false);
    setAddingServer(false);
  };
  const openSettings = (): void => {
    closePanels();
    setSettingsOpen(true);
  };
  const openHost = (section: HostSection): void => {
    closePanels();
    setHostSection(section);
  };
  const openAdd = (): void => {
    closePanels();
    setAddingServer(true);
  };

  const noteInfo = useCallback((baseUrl: string, next: ServerInfo | null): void => {
    setInfo((held) => {
      if (next === null) {
        if (!(baseUrl in held)) return held;
        const without = { ...held };
        delete without[baseUrl];
        return without;
      }
      return { ...held, [baseUrl]: next };
    });
  }, []);

  useEffect(() => {
    applyDensity(density);
  }, [density]);

  // One watcher for the window, however many servers there are. The per-server
  // records live inside it and are added and dropped by `ServerLink`.
  //
  // Nothing here closes a connection. Each `ServerLink` owns exactly one and
  // closes it when it unmounts, and reaching around them from up here to close
  // all of them is how the frame listener ends up racing a connect that has not
  // finished — a live socket with nobody following it (see `lib/gateway.ts`).
  useEffect(() => {
    const stop = startPresence();
    return () => {
      stop();
      resetNotifications();
    };
  }, []);

  const rooms = [...gateway.rooms]
    .filter((room) => room.archived_at === null)
    .sort((a, b) => a.position - b.position);
  // Land in the first room, and don't hold a room that was archived or that
  // this account can no longer see.
  const open = rooms.find((room) => room.id === openRoomIds[active.baseUrl]) ?? rooms[0] ?? null;

  // Nothing interrupts you about the room you are already reading, and you are
  // only ever standing in one room — switching servers takes you out of the
  // last one.
  useEffect(() => {
    setViewing(open === null ? null : { server: active.baseUrl, roomId: open.id });
    setPresenceRoom(active.baseUrl, open?.id ?? null);
  }, [active.baseUrl, open?.id]);

  // A fresh `ready` is a new session on the server: we are `around` until
  // this clock re-announces the room. Anything short of ready is not a
  // connection worth sending presence on.
  useEffect(() => {
    setPresenceLive(active.baseUrl, gateway.status.kind === "ready");
  }, [active.baseUrl, gateway.status.kind]);

  const status = statusText(gateway.status);
  const statusDetail = gateway.status.kind === "waiting" ? gateway.status.reason : undefined;

  // One roster, in one of two places. Rendering it twice and hiding one would
  // mean two of everything it holds — two open cards, two scroll positions.
  const roster = <Roster api={api} rooms={rooms} layout={narrow ? "strip" : "column"} />;

  // `ready` is the fresher answer about who we are; the stored session is what
  // we have before it arrives. Neither is the lock — every host endpoint checks
  // for itself — so this only decides whether the controls are drawn at all.
  const isHost = gateway.me?.is_host ?? active.user.is_host;
  const host = isHost ? hostSection : null;
  // `ready` is the live copy of who we are; the stored session is the fallback
  // until it arrives. The status bar and settings both prefer the live one so
  // a display-name save shows up without a reload.
  const you = gateway.me ?? active.user;

  // The server's accent, if the host picked one (SPEC §5.3, `PATCH /server`).
  // It names a palette key, and the variable that key points at is generated
  // from `linger-core::PALETTE` in M6 — until then every key falls back to the
  // built-in accent and this line quietly does nothing.
  const frameStyle: CSSProperties = {
    "--accent": colorVar(server?.accent_key ?? "", "var(--accent-default)"),
  };

  return (
    <div className="frame" data-narrow={narrow ? "true" : undefined} style={frameStyle}>
      {servers.map((session) => (
        <ServerLink key={session.baseUrl} session={session} onInfo={noteInfo} />
      ))}

      <aside className="rail">
        <section className="rail-section">
          <div className="rail-head">
            <h2 className="panel-label">servers</h2>
            <button
              type="button"
              className="rail-action meta"
              aria-pressed={addingServer}
              onClick={() => (addingServer ? setAddingServer(false) : openAdd())}
            >
              + add
            </button>
          </div>
          <ul className="server-list">
            {servers.map((session) => (
              <li key={session.baseUrl}>
                <ServerRow
                  name={info[session.baseUrl]?.name ?? hostOf(session.baseUrl)}
                  state={all[session.baseUrl]}
                  current={session.baseUrl === active.baseUrl}
                  onOpen={() => {
                    setActiveUrl(session.baseUrl);
                    closePanels();
                  }}
                />
              </li>
            ))}
          </ul>
        </section>
        <section className="rail-section">
          <div className="rail-head">
            <h2 className="panel-label">server</h2>
            <div className="rail-actions">
              {isHost ? (
                <button
                  type="button"
                  className="rail-action meta"
                  aria-pressed={host !== null}
                  onClick={() =>
                    host === null ? openHost("server") : setHostSection(null)
                  }
                >
                  manage
                </button>
              ) : null}
              <button
                type="button"
                className="rail-action meta"
                aria-pressed={settingsOpen}
                aria-label="your settings"
                onClick={() => (settingsOpen ? setSettingsOpen(false) : openSettings())}
              >
                you
              </button>
            </div>
          </div>
          <p className="rail-server">{server?.name ?? hostOf(active.baseUrl)}</p>
        </section>
        <section className="rail-section rail-rooms">
          <div className="rail-head">
            <h2 className="panel-label">rooms</h2>
            {isHost ? (
              <button
                type="button"
                className="rail-action meta"
                onClick={() => openHost("rooms")}
              >
                + room
              </button>
            ) : null}
          </div>
          {rooms.length === 0 ? (
            <p className="placeholder">{noRoomsRail()}</p>
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
                    onClick={() => {
                      setOpenRoomIds((held) => ({ ...held, [active.baseUrl]: room.id }));
                      // You clicked a room to read it, so the host panel gets
                      // out of the way rather than sitting over the stream.
                      closePanels();
                    }}
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

      {addingServer ? (
        <main className="stream">
          <header className="stream-header">
            <span className="room-name">add a server</span>
            <button
              type="button"
              className="rail-action meta"
              onClick={() => setAddingServer(false)}
            >
              close
            </button>
          </header>
          <AuthScreens
            inline
            notice={null}
            keyringNotice={keyringNotice}
            onAuthenticated={async (baseUrl, auth) => {
              await onAddServer(baseUrl, auth);
              setActiveUrl(baseUrl);
              setAddingServer(false);
            }}
          />
        </main>
      ) : settingsOpen ? (
        <SettingsPanel
          api={api}
          user={you}
          density={density}
          onDensityChange={setDensity}
          onSignOut={() => onSignOut(active.baseUrl)}
          onReauthenticated={(auth) => onAddServer(api.baseUrl, auth)}
          onClose={() => setSettingsOpen(false)}
          roster={narrow ? roster : undefined}
        />
      ) : host !== null ? (
        <HostPanel
          api={api}
          rooms={rooms}
          server={server}
          section={host}
          onSection={setHostSection}
          onServerChange={(next) => noteInfo(active.baseUrl, next)}
          onClose={() => setHostSection(null)}
          roster={narrow ? roster : undefined}
        />
      ) : open === null ? (
        <main className="stream">
          <header className="stream-header">
            <span className="room-name">
              {gateway.status.kind === "ready" ? "no rooms yet" : "welcome"}
            </span>
          </header>
          <div className="stream-body">
            <p className="placeholder">
              {noRoomsBody(gateway.status.kind === "ready", isHost)}
            </p>
            {/* The host is the one person who can fix this, so they get the
                way out rather than a sentence about it. */}
            {isHost && gateway.status.kind === "ready" ? (
              <p className="placeholder">
                <button
                  type="button"
                  className="rail-action meta"
                  onClick={() => openHost("rooms")}
                >
                  make the first room
                </button>
              </p>
            ) : null}
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
          <button
            className="status-action"
            type="button"
            aria-pressed={settingsOpen}
            aria-label="your settings"
            onClick={() => (settingsOpen ? setSettingsOpen(false) : openSettings())}
          >
            {you.display_name}
          </button>
          {keyringNotice ? <span className="status-warn">not remembered</span> : null}
        </span>
      </footer>
    </div>
  );
}

/**
 * One server in the rail: a dot, a name, and nothing else (SPEC §3).
 *
 * The dot is the connection — filled once that server has said `ready`, hollow
 * while it is retrying. A server holding something you have not read gets the
 * same weight change the room list uses: heavier text, never a badge and never
 * a count (SPEC §4.2, AGENTS rule 3). The accessible name says it in words,
 * because a font weight is not something a screen reader can read out.
 */
function ServerRow({
  name,
  state,
  current,
  onOpen,
}: {
  name: string;
  state: GatewayState | undefined;
  current: boolean;
  onOpen: () => void;
}) {
  const live = state?.status.kind === "ready";
  const waiting = state !== undefined && anyNewActivity(state);
  const label = [name, live ? "connected" : "not connected", waiting ? "something new" : null]
    .filter((part) => part !== null)
    .join(", ");
  return (
    <button
      type="button"
      className="server-item"
      aria-current={current ? "true" : undefined}
      aria-label={label}
      data-live={live ? "true" : undefined}
      data-new={waiting ? "true" : undefined}
      onClick={onOpen}
    >
      <span className="server-dot" aria-hidden="true" />
      <span className="server-name">{name}</span>
    </button>
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
