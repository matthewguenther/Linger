/**
 * "Always notify me when [person] posts" — per person, per room (SPEC §4.2).
 *
 * This is the whole notification settings surface, and it is deliberately the
 * only one. There is no keyword list, no "highlight words", no digest, no
 * schedule. The claim in the spec is that the setting people actually want is
 * *this* person, and that a keyword system is what you build when you have too
 * many people to name.
 *
 * The data model is two fields, so the UI is two questions: a person, and
 * either everywhere or a room. A rule with no room means everywhere, which is
 * why turning that on hides the per-room switches — they would be answering a
 * question that has already been answered.
 */
import { useState } from "react";

import type { NotifyRule } from "../generated/NotifyRule";
import type { Room } from "../generated/Room";
import type { User } from "../generated/User";
import { ApiError, type AuthedApi } from "../lib/api";
import { setNotifyRule, useGateway } from "../lib/gateway";
import { nameProps } from "../lib/names";
import "./notify.css";

export default function NotifyRules({ api, rooms }: { api: AuthedApi; rooms: Room[] }) {
  const gateway = useGateway(api.baseUrl);
  const me = gateway.me;
  const [open, setOpen] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  const others = gateway.users
    .filter((person) => person.id !== me?.id)
    .sort((a, b) => a.display_name.localeCompare(b.display_name));

  const change = (rule: NotifyRule, on: boolean): void => {
    setProblem(null);
    void setNotifyRule(api, rule, on).catch((error: unknown) => {
      setProblem(error instanceof ApiError ? error.message : "Couldn't reach the server.");
    });
  };

  if (others.length === 0) {
    return <p className="placeholder">nobody else here yet</p>;
  }

  return (
    <div className="notify">
      {problem ? <p className="notify-problem meta">{problem}</p> : null}
      <ul className="notify-list">
        {others.map((person) => (
          <PersonRules
            key={person.id}
            person={person}
            rooms={rooms}
            rules={gateway.notifyRules}
            open={open === person.id}
            onToggleOpen={() => setOpen((held) => (held === person.id ? null : person.id))}
            onChange={change}
          />
        ))}
      </ul>
      <p className="notify-note meta">
        Somebody naming you always reaches you. Nothing else does, and there is no
        @everyone to turn on.
      </p>
    </div>
  );
}

function PersonRules({
  person,
  rooms,
  rules,
  open,
  onToggleOpen,
  onChange,
}: {
  person: User;
  rooms: Room[];
  rules: NotifyRule[];
  open: boolean;
  onToggleOpen: () => void;
  onChange: (rule: NotifyRule, on: boolean) => void;
}) {
  const mine = rules.filter((rule) => rule.target_user_id === person.id);
  const everywhere = mine.some((rule) => rule.room_id === null);
  const named = rooms.filter((room) => mine.some((rule) => rule.room_id === room.id));

  const summary = everywhere
    ? "everywhere"
    : named.length === 0
      ? "—"
      : named.map((room) => `#${room.slug}`).join(" ");

  return (
    <li className="notify-person">
      <button
        type="button"
        className="notify-name"
        aria-expanded={open}
        onClick={onToggleOpen}
      >
        <span {...nameProps(person)}>{person.display_name}</span>
        <span className="notify-summary meta">{summary}</span>
      </button>

      {open ? (
        <div className="notify-choices">
          <Choice
            label="everywhere"
            on={everywhere}
            onChange={(next) => onChange({ target_user_id: person.id, room_id: null }, next)}
          />
          {everywhere
            ? null
            : rooms.map((room) => (
                <Choice
                  key={room.id}
                  label={`#${room.slug}`}
                  on={mine.some((rule) => rule.room_id === room.id)}
                  onChange={(next) =>
                    onChange({ target_user_id: person.id, room_id: room.id }, next)
                  }
                />
              ))}
        </div>
      ) : null}
    </li>
  );
}

/** A plain checkbox. No switch, no animation — this is a settings list. */
function Choice({
  label,
  on,
  onChange,
}: {
  label: string;
  on: boolean;
  onChange: (on: boolean) => void;
}) {
  return (
    <label className="notify-choice">
      <input type="checkbox" checked={on} onChange={(event) => onChange(event.target.checked)} />
      <span>{label}</span>
    </label>
  );
}
