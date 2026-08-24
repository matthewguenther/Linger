/**
 * The top of the client: sign-in first, the Console frame once you're in.
 *
 * The frame is [rail | stream | roster] over a permanent status bar, and the
 * roster is the point of it (SPEC §3): people get the right-hand panel, not a
 * gutter. On a window too narrow for three columns the roster moves into the
 * stream column as a horizontal strip above the composer — it is never hidden
 * and it never becomes a menu, so `Stream` takes it as a slot.
 *
 * The rail starts with the server list (SPEC §3, T-412): a live dot per
 * server, weight when a background server has something you have not seen,
 * and `+ add` for the paste box that used to exist only before sign-in.
 *
 * Below that, the host's own controls hang: `+ room` beside the
 * room list and `manage` beside the rooms heading. They are *absent* for
 * everybody else rather than greyed out — a disabled control is a permission
 * matrix drawn in CSS, and this product refuses to have one.
 *
 * `you` is the other door: display name, password, density, sign out. It is
 * drawn for everybody, because those are yours, not the host's. The panel
 * takes the stream column the same way `manage` does (T-411).
 *
 * The rail is where SPEC §4.2's other half lives: a room holding something you
 * have not seen changes *weight*, and nothing else. No number, no dot, no
 * color. It is one line of CSS and it is the whole feature.
 */
import { type CSSProperties, useEffect, useRef, useState } from "react";

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
  connect,
  disconnect,
  hasNewActivity,
  loadNotifyRules,
  loadReadMarkers,
  setActiveServer,
  statusText,
  useGateway,
  useServerSummaries,
} from "./lib/gateway";
import { useNarrow } from "./lib/layout";
import { hostOf } from "./lib/link";
import { personStyle } from "./lib/names";
import { occupancyLine, occupantsOf, STACK_VISIBLE } from "./lib/occupancy";
import { colorVar } from "./lib/palette";
import { useSession, type ServerEntry } from "./lib/session";
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
      servers={session.state.servers}
      activeUrl={session.state.active}
      keyringNotice={session.keyringNotice}
      onActive={session.setActive}
      onSignOut={session.signOut}
      onSignIn={session.signIn}
    />
  );
}

function Console({
  servers,
  activeUrl,
  keyringNotice,
  onActive,
  onSignOut,
  onSignIn,
}: {
  servers: ServerEntry[];
  activeUrl: string;
  keyringNotice: string | null;
  onActive: (baseUrl: string) => void;
  onSignOut: (baseUrl?: string) => Promise<void>;
  onSignIn: (baseUrl: string, auth: AuthResponse) => Promise<void>;
}) {
  const current = servers.find((entry) => entry.api.baseUrl === activeUrl) ?? servers[0] ?? null;
  const api = current?.api ?? null;
  const user = current?.user ?? null;

  const gateway = useGateway();
  const summaries = useServerSummaries();
  const [infoByUrl, setInfoByUrl] = useState<Record<string, ServerInfo>>({});
  const [openByUrl, setOpenByUrl] = useState<Record<string, RoomId | null>>({});
  const [density, setDensity] = useState<Density>(loadDensity);
  // Which host surface is open over the stream, if any (T-410).
  const [hostSection, setHostSection] = useState<HostSection | null>(null);
  // The member's own settings (T-411). Mutually exclusive with the host panel:
  // both take the stream column, and two overlays is a modal stack.
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [adding, setAdding] = useState(false);
  const narrow = useNarrow();
  const serverUrls = servers.map((entry) => entry.api.baseUrl).join("\n");
  const connectedUrls = useRef(new Set<string>());

  const openSettings = (): void => {
    setHostSection(null);
    setAdding(false);
    setSettingsOpen(true);
  };
  const openHost = (section: HostSection): void => {
    setSettingsOpen(false);
    setAdding(false);
    setHostSection(section);
  };
  const switchServer = (baseUrl: string): void => {
    if (api === null || baseUrl === api.baseUrl) return;
    setHostSection(null);
    setSettingsOpen(false);
    setAdding(false);
    setActiveServer(baseUrl);
    onActive(baseUrl);
  };

  useEffect(() => {
    applyDensity(density);
  }, [density]);

  useEffect(() => {
    const stop = startPresence();
    return () => {
      stop();
      resetNotifications();
      void disconnect();
    };
  }, []);

  useEffect(() => {
    const live = new Set(servers.map((entry) => entry.api.baseUrl));
    for (const entry of servers) void connect(entry.api);
    // Adding a second server must not tear down the first. Only a url that
    // left the list (signed out) is closed here. Unmount of the whole frame
    // still disconnects everyone, via the presence effect above.
    for (const url of connectedUrls.current) {
      if (!live.has(url)) void disconnect(url);
    }
    connectedUrls.current = live;
  }, [serverUrls, servers]);

  useEffect(() => {
    if (api === null) return;
    setActiveServer(api.baseUrl);
  }, [api]);

  // Where you had got to, and who you asked to hear from. Both are small, both
  // are needed before the first frame is judged worth interrupting anyone for,
  // and neither is worth a screen of its own if it fails.
  useEffect(() => {
    if (api === null) return;
    void loadReadMarkers(api);
    void loadNotifyRules(api).catch(() => undefined);
  }, [api]);

  useEffect(() => {
    const abort = new AbortController();
    // The server's name is the one thing `ready` doesn't carry. A failure isn't
    // worth a screen of its own: the rail falls back to the hostname.
    for (const entry of servers) {
      const url = entry.api.baseUrl;
      void entry.api
        .serverInfo(abort.signal)
        .then((info) => {
          setInfoByUrl((held) => ({ ...held, [url]: info }));
        })
        .catch(() => undefined);
    }
    return () => abort.abort();
  }, [api, serverUrls, servers]);

  const rooms = [...gateway.rooms]
    .filter((room) => room.archived_at === null)
    .sort((a, b) => a.position - b.position);
  const server = api !== null ? (infoByUrl[api.baseUrl] ?? null) : null;
  const openRoomId = api !== null ? (openByUrl[api.baseUrl] ?? null) : null;
  // Land in the first room, and don't hold a room that was archived or that
  // this account can no longer see.
  const open = rooms.find((room) => room.id === openRoomId) ?? rooms[0] ?? null;

  // Nothing interrupts you about the room you are already reading.
  useEffect(() => {
    if (api === null) return;
    setViewing(open ? { baseUrl: api.baseUrl, roomId: open.id } : null);
    setPresenceRoom(open?.id ?? null);
  }, [api, open?.id]);

  // A fresh `ready` is a new session on the server: we are `around` until
  // this clock re-announces the room. Anything short of ready is not a
  // connection worth sending presence on.
  useEffect(() => {
    setPresenceLive(gateway.status.kind === "ready");
  }, [gateway.status.kind]);

  if (current === null || api === null || user === null) return null;

  const status = statusText(gateway.status);
  const statusDetail = gateway.status.kind === "waiting" ? gateway.status.reason : undefined;

  // One roster, in one of two places. Rendering it twice and hiding one would
  // mean two of everything it holds — two open cards, two scroll positions.
  const roster = <Roster api={api} rooms={rooms} layout={narrow ? "strip" : "column"} />;

  // `ready` is the fresher answer about who we are; the stored session is what
  // we have before it arrives. Neither is the lock — every host endpoint checks
  // for itself — so this only decides whether the controls are drawn at all.
  const isHost = gateway.me?.is_host ?? user.is_host;
  const host = isHost ? hostSection : null;
  // `ready` is the live copy of who we are; the stored session is the fallback
  // until it arrives. The status bar and settings both prefer the live one so
  // a display-name save shows up without a reload.
  const you = gateway.me ?? user;

  // The server's accent, if the host picked one (SPEC §5.3, `PATCH /server`).
  // It names a palette key, and the variable that key points at is generated
  // from `linger-core::PALETTE` in M7 — until then every key falls back to the
  // built-in accent and this line quietly does nothing.
  const frameStyle: CSSProperties = {
    "--accent": colorVar(server?.accent_key ?? "", "var(--accent-default)"),
  };

  return (
    <div className="frame" data-narrow={narrow ? "true" : undefined} style={frameStyle}>
      <aside className="rail">
        <section className="rail-section">
          <div className="rail-head">
            <h2 className="panel-label">servers</h2>
            <div className="rail-actions">
              <button
                type="button"
                className="rail-action meta"
                aria-pressed={adding}
                onClick={() => {
                  setHostSection(null);
                  setSettingsOpen(false);
                  setAdding((held) => !held);
                }}
              >
                + add
              </button>
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
          <ul className="server-list">
            {servers.map((entry) => {
              const url = entry.api.baseUrl;
              const summary = summaries.find((item) => item.baseUrl === url);
              const live = summary?.status.kind === "ready";
              const selected = url === api.baseUrl;
              const label = infoByUrl[url]?.name ?? hostOf(url);
              return (
                <li key={url}>
                  <button
                    type="button"
                    className="server-item"
                    aria-current={selected ? "true" : undefined}
                    data-live={live ? "true" : undefined}
                    data-new={!selected && summary?.hasNew ? "true" : undefined}
                    onClick={() => switchServer(url)}
                  >
                    <span
                      className="server-dot"
                      data-live={live ? "true" : undefined}
                      aria-hidden="true"
                    />
                    <span className="server-name">{label}</span>
                    <span className="sr-only">
                      {live ? "connected" : "not connected"}
                      {!selected && summary?.hasNew ? ", something new" : ""}
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        </section>
        <section className="rail-section rail-rooms">
          <div className="rail-head">
            <h2 className="panel-label">rooms</h2>
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
                      setOpenByUrl((held) => ({ ...held, [api.baseUrl]: room.id }));
                      // You clicked a room to read it, so the overlay gets
                      // out of the way rather than sitting over the stream.
                      setHostSection(null);
                      setSettingsOpen(false);
                      setAdding(false);
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

      {adding ? (
        <main className="stream">
          <AuthScreens
            notice={null}
            keyringNotice={keyringNotice}
            onAuthenticated={async (baseUrl, auth) => {
              await onSignIn(baseUrl, auth);
              setAdding(false);
            }}
            onCancel={() => setAdding(false)}
          />
          {narrow ? roster : null}
        </main>
      ) : settingsOpen ? (
        <SettingsPanel
          api={api}
          user={you}
          density={density}
          onDensityChange={setDensity}
          onSignOut={() => onSignOut(api.baseUrl)}
          onReauthenticated={(auth) => onSignIn(api.baseUrl, auth)}
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
          onServerChange={(info) =>
            setInfoByUrl((held) => ({ ...held, [api.baseUrl]: info }))
          }
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
