/**
 * One message: what it says, who said it, what people did about it.
 *
 * The actions — react, reply, edit, delete — sit in a strip pinned to the top
 * right of the row. It is always in the DOM and invisible until the pointer is
 * over the message or the keyboard is inside it, which is what makes it
 * reachable by tab without being a wall of buttons at rest. Because it is
 * positioned out of flow, showing it never changes the row's height, and the
 * virtualizer never has to re-measure anything to draw it.
 *
 * The strip has three states and swaps them in place rather than opening a
 * popover: the buttons, the twelve reactions, or "delete this?". A popover
 * would have to be positioned against a scroller, clipped at its edges, and
 * layered over the rows after it — three problems in exchange for nothing, when
 * the strip is right there and already the width of twelve glyphs.
 */
import { type FormEvent, type KeyboardEvent, useEffect, useRef, useState } from "react";

import type { Message } from "../generated/Message";
import type { ReactionGroup } from "../generated/ReactionGroup";
import type { User } from "../generated/User";
import type { UserId } from "../generated/UserId";
import { useAutoGrow } from "../lib/autogrow";
import { MAX_MESSAGE_CHARS } from "../lib/limits";
import MessageBody from "./Markdown";
import { plainText } from "./markdown";
import { reactionFor, REACTIONS, weightOf } from "./reactions";
import type { StreamRow } from "./rows";
import { ageOpacity, clockTime, fullTime } from "./time";

/** How much of the message being replied to fits on the line above a reply. */
const QUOTE_CHARS = 120;

export interface MessageActions {
  onReply: (message: Message) => void;
  onEditStart: (message: Message) => void;
  onEditCancel: () => void;
  onEditSave: (message: Message, body: string) => Promise<void>;
  onDelete: (message: Message) => void;
  onReact: (message: Message, key: string, on: boolean) => void;
  /** Move the view to the message this one is answering. */
  onJumpToParent: (parent: Message) => void;
}

interface MessageRowProps {
  row: Extract<StreamRow, { kind: "message" }>;
  author: User | undefined;
  /** Everyone the server has told us about, for reply quotes and reaction names. */
  people: Map<string, User>;
  me: User | null;
  /** The message this one replies to, if it is loaded. */
  parent: Message | undefined;
  now: number;
  irc: boolean;
  editing: boolean;
  actions: MessageActions;
}

export default function MessageRow({
  row,
  author,
  people,
  me,
  parent,
  now,
  irc,
  editing,
  actions,
}: MessageRowProps) {
  const { message, head } = row;
  const name = author?.display_name ?? "someone";
  const deleted = message.deleted_at !== null;

  const nameStyle = {
    fontWeight: author?.style.weight ?? 500,
    fontStyle: author?.style.italic === true ? "italic" : "normal",
  } as const;

  const time = (
    <time
      className="msg-time meta"
      dateTime={new Date(message.created_at).toISOString()}
      title={fullTime(message.created_at)}
    >
      {clockTime(message.created_at, irc)}
    </time>
  );

  const body = deleted ? (
    <span className="msg-gone">deleted</span>
  ) : editing ? (
    <Editor message={message} actions={actions} />
  ) : (
    <MessageBody
      source={message.body}
      trailing={message.edited_at === null ? null : <span className="msg-edited meta">edited</span>}
    />
  );

  const reactions =
    message.reactions.length === 0 ? null : (
      <Reactions
        groups={message.reactions}
        people={people}
        me={me}
        onToggle={(key, on) => actions.onReact(message, key, on)}
      />
    );

  const strip = deleted ? null : (
    <ActionStrip message={message} me={me} author={author} editing={editing} actions={actions} />
  );

  // One line per message: a fixed-width timestamp gutter, then the aligned nick
  // column mIRC had, then the text (SPEC §5.6). Reactions ride along at the end
  // of the line rather than claiming a second one.
  if (irc) {
    return (
      <>
        <div className="msg-block">
          <div className="msg-body">
            {time}
            <span className="irc-name" style={nameStyle}>
              {name}
            </span>
            <div className="irc-text">
              {parent ? <ReplyQuote parent={parent} people={people} actions={actions} /> : null}
              {body}
              {reactions}
            </div>
          </div>
        </div>
        {strip}
      </>
    );
  }

  return (
    <>
      {head ? (
        <p className="msg-head">
          <span className="msg-author" style={nameStyle}>
            {name}
          </span>
          {time}
        </p>
      ) : null}
      <div className="msg-block">
        {parent ? <ReplyQuote parent={parent} people={people} actions={actions} /> : null}
        {/* Aging is one custom property, computed from the timestamp and applied
            to the body only — never the name, never the time (SPEC §5.6), and
            never a reaction, which is somebody else's and is not aging. */}
        <div className="msg-body" style={{ "--age": ageOpacity(message.created_at, now) }}>
          {body}
        </div>
        {reactions}
      </div>
      {strip}
    </>
  );
}

/**
 * The line above a reply, naming what it answers.
 *
 * Only drawn when the message being answered is loaded. There is no endpoint
 * for one message, so the alternative to leaving it out would be a page fetch
 * per reply — and scrolling up far enough to see the original is the thing that
 * makes it appear anyway.
 */
function ReplyQuote({
  parent,
  people,
  actions,
}: {
  parent: Message;
  people: Map<string, User>;
  actions: MessageActions;
}) {
  const who = people.get(parent.author_id)?.display_name ?? "someone";
  const said =
    parent.deleted_at !== null ? "deleted" : plainText(parent.body).slice(0, QUOTE_CHARS);
  return (
    <button
      type="button"
      className="msg-reply meta"
      onClick={() => actions.onJumpToParent(parent)}
      title="go to that message"
    >
      <span className="msg-reply-mark" aria-hidden="true">
        ↳
      </span>
      <span className="msg-reply-who">{who}</span>
      <span className="msg-reply-said">{said}</span>
    </button>
  );
}

/** Editing in place, because the message you are changing is the context. */
function Editor({ message, actions }: { message: Message; actions: MessageActions }) {
  const [draft, setDraft] = useState(message.body);
  const [saving, setSaving] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const field = useRef<HTMLTextAreaElement | null>(null);
  useAutoGrow(field, draft);

  useEffect(() => {
    const element = field.current;
    if (!element) return;
    element.focus();
    // Land at the end of what is already there, which is where you were when
    // you decided the message needed fixing.
    element.setSelectionRange(element.value.length, element.value.length);
  }, []);

  const save = async (event: FormEvent): Promise<void> => {
    event.preventDefault();
    const body = draft.trim();
    if (saving) return;
    if (body.length === 0) {
      // An emptied message is a delete, and deleting is a different decision
      // with a different button. Say so rather than doing one for the other.
      setProblem("An empty message is a delete. Use delete.");
      return;
    }
    if (body === message.body.trim()) {
      actions.onEditCancel();
      return;
    }
    setSaving(true);
    try {
      await actions.onEditSave(message, body);
    } catch (error) {
      setProblem(error instanceof Error ? error.message : "That edit didn't go through.");
    } finally {
      setSaving(false);
    }
  };

  const keys = (event: KeyboardEvent<HTMLTextAreaElement>): void => {
    if (event.key === "Escape") {
      event.preventDefault();
      actions.onEditCancel();
      return;
    }
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void save(event);
    }
  };

  return (
    <form className="msg-editor" onSubmit={(event) => void save(event)}>
      <textarea
        ref={field}
        className="composer-input msg-editor-field"
        value={draft}
        maxLength={MAX_MESSAGE_CHARS}
        rows={1}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={keys}
        aria-label="edit this message"
      />
      <p className="msg-editor-hint meta">
        {problem ?? "enter saves · shift-enter is a new line · esc cancels"}
      </p>
    </form>
  );
}

type StripMode = "buttons" | "react" | "confirm";

function ActionStrip({
  message,
  me,
  author,
  editing,
  actions,
}: {
  message: Message;
  me: User | null;
  author: User | undefined;
  editing: boolean;
  actions: MessageActions;
}) {
  const [mode, setMode] = useState<StripMode>("buttons");
  const strip = useRef<HTMLDivElement | null>(null);
  const was = useRef<StripMode>("buttons");

  // The strip belongs to the message, so it goes back to its resting state when
  // the message stops being the one you are working on.
  useEffect(() => {
    if (editing) setMode("buttons");
  }, [editing]);

  // Changing mode replaces every button in the strip, which drops the keyboard
  // wherever it was standing. Put it on the first button of whatever the strip
  // just became — pressing "react" should land on a reaction, not on nothing.
  useEffect(() => {
    if (mode === was.current) return;
    was.current = mode;
    // "delete this?" lands on *no*. A confirmation whose default answer is yes
    // is not a confirmation.
    const target =
      mode === "confirm"
        ? strip.current?.querySelector<HTMLButtonElement>("[data-answer='no']")
        : strip.current?.querySelector("button");
    target?.focus();
  }, [mode]);

  const mine = me !== null && me.id === message.author_id;
  // Editing is the author's alone; deleting is the author's or the host's
  // (PROTOCOL §4). The server decides either way — this only stops us offering
  // a button that would come back refused.
  const canDelete = mine || me?.is_host === true;
  const whose = author?.display_name ?? "someone";

  if (editing) {
    return (
      <div className="msg-actions" data-open="true">
        <button type="button" className="msg-action meta" onClick={actions.onEditCancel}>
          cancel
        </button>
      </div>
    );
  }

  if (mode === "react") {
    const mineAlready = (key: string): boolean =>
      me !== null &&
      message.reactions.some((group) => group.key === key && group.user_ids.includes(me.id));

    return (
      <div
        className="msg-actions"
        data-open="true"
        role="group"
        aria-label="pick a reaction"
        ref={strip}
      >
        {REACTIONS.map((reaction) => (
          <button
            key={reaction.key}
            type="button"
            className="msg-action msg-pick"
            aria-label={reaction.label}
            aria-pressed={mineAlready(reaction.key)}
            onClick={() => {
              actions.onReact(message, reaction.key, !mineAlready(reaction.key));
              setMode("buttons");
            }}
          >
            <span aria-hidden="true">{reaction.glyph}</span>
          </button>
        ))}
      </div>
    );
  }

  if (mode === "confirm") {
    return (
      <div className="msg-actions" data-open="true" ref={strip}>
        <span className="msg-action-label meta">delete this?</span>
        <button
          type="button"
          className="msg-action meta"
          onClick={() => {
            setMode("buttons");
            actions.onDelete(message);
          }}
        >
          yes
        </button>
        <button
          type="button"
          className="msg-action meta"
          data-answer="no"
          onClick={() => setMode("buttons")}
        >
          no
        </button>
      </div>
    );
  }

  return (
    <div className="msg-actions" ref={strip}>
      <button
        type="button"
        className="msg-action meta"
        aria-label={`react to ${whose}'s message`}
        onClick={() => setMode("react")}
      >
        react
      </button>
      <button
        type="button"
        className="msg-action meta"
        aria-label={`reply to ${whose}`}
        onClick={() => actions.onReply(message)}
      >
        reply
      </button>
      {mine ? (
        <button
          type="button"
          className="msg-action meta"
          aria-label="edit this message"
          onClick={() => actions.onEditStart(message)}
        >
          edit
        </button>
      ) : null}
      {canDelete ? (
        <button
          type="button"
          className="msg-action meta"
          aria-label={`delete ${whose}'s message`}
          onClick={() => setMode("confirm")}
        >
          delete
        </button>
      ) : null}
    </div>
  );
}

/**
 * What a message collected (SPEC §4.8).
 *
 * The mark gets denser and larger with the crowd; the number is in the hover
 * title and the accessibility label and nowhere a person can read it as a
 * score. Hover names names, which is the thing you actually want to know.
 */
function Reactions({
  groups,
  people,
  me,
  onToggle,
}: {
  groups: ReactionGroup[];
  people: Map<string, User>;
  me: User | null;
  onToggle: (key: string, on: boolean) => void;
}) {
  return (
    <div className="reactions">
      {groups.map((group) => {
        const reaction = reactionFor(group.key);
        const mine = me !== null && group.user_ids.includes(me.id);
        const who = whoReacted(group.user_ids, people, me);
        return (
          <button
            key={group.key}
            type="button"
            className="reaction"
            data-mine={mine}
            style={{ "--weight": weightOf(group.count) }}
            title={`${reaction.label} — ${who}`}
            aria-label={`${reaction.label}, ${who}`}
            aria-pressed={mine}
            onClick={() => onToggle(group.key, !mine)}
          >
            <span aria-hidden="true">{reaction.glyph}</span>
          </button>
        );
      })}
    </div>
  );
}

/** "you and Callie", "Matt, Callie and 3 others" — for hover and screen readers. */
function whoReacted(userIds: UserId[], people: Map<string, User>, me: User | null): string {
  const names = userIds.map((id) =>
    id === me?.id ? "you" : (people.get(id)?.display_name ?? "someone"),
  );
  const shown = names.slice(0, 4);
  const rest = names.length - shown.length;
  if (rest > 0) return `${shown.join(", ")} and ${rest} others`;
  if (shown.length <= 1) return shown[0] ?? "nobody";
  return `${shown.slice(0, -1).join(", ")} and ${shown[shown.length - 1]}`;
}
