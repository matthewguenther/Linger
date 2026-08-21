/**
 * Who's around: the card stack, and the thesis of the whole app (SPEC §3).
 *
 * Discord puts people in a gutter. Linger puts them on the right-hand panel and
 * gives each of them a card: their name in their own styling, a presence dot,
 * the room they are in, what they are doing if they chose to share it, and their
 * status underneath. People who are gone stay on the stack with when they were
 * last here and whatever they left as an away message. An empty server should
 * still read as a house with the lights on, and this panel is the only thing
 * that can do that.
 *
 * Three rules it follows.
 *
 * **Nothing here is a count** (SPEC §4.2). Cards are cards; there is no "3
 * online", and the durations are how long ago, never how many.
 *
 * **The activity line is a registry label, never a window title** (SPEC §4.3).
 * The wire type has no field for one, so this file could not render one if it
 * wanted to — which is the point of putting the rule in the type.
 *
 * **On a narrow window it becomes a horizontal strip above the composer**, and
 * it is never hidden. Same component, same cards, laid out along instead of
 * down; `lib/layout.ts` owns the one number that decides which.
 */
import { useMemo, useState } from "react";

import type { Room } from "../generated/Room";
import type { AuthedApi } from "../lib/api";
import { useNow } from "../lib/clock";
import { useGateway } from "../lib/gateway";
import { nameStyle, personStyle } from "../lib/names";
import NotifyRules from "../notify/NotifyRules";
import {
  activityMark,
  buildRoster,
  hasStatus,
  type RosterEntry,
  shortAgo,
  stateWord,
} from "./roster";
import "./roster.css";

/** Down the right-hand side, or along the top of the composer. */
export type RosterLayout = "column" | "strip";

export default function Roster({
  api,
  rooms,
  layout,
}: {
  api: AuthedApi;
  rooms: Room[];
  layout: RosterLayout;
}) {
  const gateway = useGateway();
  const now = useNow();
  // The panel has two modes: who is around, and who you want to hear from.
  // The rules are a list of people too, and a settings screen for one setting
  // would be a screen too many.
  const [notifying, setNotifying] = useState(false);
  // One card open at a time. Two open cards is a list of statuses, which is a
  // different panel and a worse one.
  const [open, setOpen] = useState<string | null>(null);

  const { users, presence, me, offlineAt } = gateway;
  const allRooms = gateway.rooms;
  const entries = useMemo(
    () =>
      buildRoster({
        users,
        presence,
        // Every room, not just the unarchived ones: if somebody is standing in
        // a room that was archived under them, the card should say where.
        rooms: allRooms,
        meId: me?.id ?? null,
        offlineAt,
        now,
      }),
    [users, presence, allRooms, me?.id, offlineAt, now],
  );

  return (
    <aside className="roster" data-layout={layout}>
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
        <div className="roster-notify">
          <NotifyRules api={api} rooms={rooms} />
        </div>
      ) : entries.length === 0 ? (
        <p className="placeholder">nobody yet</p>
      ) : (
        <ul className="roster-list">
          {entries.map((entry) => (
            <PersonCard
              key={entry.user.id}
              entry={entry}
              now={now}
              open={open === entry.user.id}
              onToggle={() =>
                setOpen((held) => (held === entry.user.id ? null : entry.user.id))
              }
            />
          ))}
        </ul>
      )}
    </aside>
  );
}

/**
 * One person.
 *
 * The head is a button only when there is a status under it. An affordance that
 * does nothing is worse than no affordance, and most cards, most of the time,
 * have nothing to open.
 */
function PersonCard({
  entry,
  now,
  open,
  onToggle,
}: {
  entry: RosterEntry;
  now: number;
  open: boolean;
  onToggle: () => void;
}) {
  const { user, state } = entry;
  const openable = hasStatus(entry);
  const head = (
    <>
      {/* The dot is decoration; the word beside it is what a screen reader
          reads, so presence is never carried by color alone. */}
      <span className="person-dot" aria-hidden="true" />
      <span className="person-name" style={nameStyle(user)}>
        {user.display_name}
      </span>
      <span className="sr-only">{stateWord(state)}</span>
      {entry.isMe ? <span className="person-you meta">you</span> : null}
    </>
  );

  return (
    <li
      className="person"
      data-state={state}
      data-open={open ? "true" : undefined}
      style={personStyle(user)}
    >
      {openable ? (
        <button type="button" className="person-head" aria-expanded={open} onClick={onToggle}>
          {head}
        </button>
      ) : (
        <p className="person-head">{head}</p>
      )}
      <PersonLines entry={entry} now={now} />
      {open ? <StatusCard entry={entry} /> : null}
    </li>
  );
}

/**
 * Where they are, what they are doing, and how long it has been.
 *
 * Order matters twice over: it is reading order, and on a narrow window the
 * strip shows only the first of these lines, so the most locating one has to
 * come first.
 */
function PersonLines({ entry, now }: { entry: RosterEntry; now: number }) {
  const { state, room, activity, awayMessage } = entry;

  // "around" says nothing here on purpose — the dot has already said it, and a
  // card that spends a line on "around" is a card that says less.
  const where =
    state === "in_room" ? (room === null ? "in a room" : `in #${room.slug}`) : null;
  const stateLine = state === "idle" || state === "away" ? stateWord(state) : null;

  const since = sinceOf(entry, now);
  const mark = activity === null ? null : activityMark(activity.kind);
  const away = awayMessage === null || awayMessage === "" ? null : awayMessage;

  return (
    <div className="person-lines">
      {where !== null || stateLine !== null ? (
        <p className="person-where">{where ?? stateLine}</p>
      ) : null}
      {activity !== null ? (
        <p className="person-activity">
          {mark === null ? null : <span className="activity-mark">{mark}</span>}
          {activity.label}
        </p>
      ) : null}
      {/* Gone, and what they left behind: SPEC §3 draws these on one line. */}
      {state === "offline" ? (
        since === null && away === null ? null : (
          <p className="person-gone">
            {since === null ? null : <span className="meta">{since}</span>}
            {away === null ? null : <span className="person-away">“{away}”</span>}
          </p>
        )
      ) : (
        <>
          {since === null ? null : <p className="person-since meta">{since}</p>}
          {away === null ? null : <p className="person-away">“{away}”</p>}
        </>
      )}
    </div>
  );
}

/**
 * The one duration on a card, and what it is counting.
 *
 * Gone: how long since we saw them. Away: since they said so. Here: how long
 * they have been in whatever they are in. There is never more than one, because
 * a card with three durations on it is a card nobody reads.
 */
function sinceOf(entry: RosterEntry, now: number): string | null {
  if (entry.state === "offline") {
    return entry.seenAt === null ? null : shortAgo(entry.seenAt, now);
  }
  const awaySince = entry.user.status?.away_since ?? null;
  if (entry.state === "away" && awaySince !== null) return shortAgo(awaySince, now);
  return entry.activity === null ? null : shortAgo(entry.activity.since, now);
}

/**
 * The status, opened (SPEC §4.6): a line in their own styling, and up to three
 * labeled fields. The status image is not here yet — it needs the media store
 * that M6 builds, and T-405 owns the editor that would set one.
 */
function StatusCard({ entry }: { entry: RosterEntry }) {
  const status = entry.user.status;
  if (!status) return null;
  const fields: [string, string | null][] = [
    ["reading", status.reading],
    ["listening to", status.listening],
    ["working on", status.working_on],
  ];
  const shown = fields.filter(([, value]) => value !== null && value !== "");
  // The away message supersedes the status line when it is set (SPEC §4.6),
  // and `PersonLines` has already drawn it.
  const line = entry.awayMessage !== null && entry.awayMessage !== "" ? null : status.line;

  return (
    <div className="person-status">
      {line === null || line === "" ? null : (
        <p className="status-line" style={nameStyle(entry.user)}>
          {line}
        </p>
      )}
      {shown.length === 0 ? null : (
        <dl className="status-fields">
          {shown.map(([label, value]) => (
            <div className="status-field" key={label}>
              <dt className="meta">{label}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
      )}
    </div>
  );
}
