/**
 * The composer.
 *
 * It is a textarea, not a text field, because messages have more than one line
 * in them: Enter sends, Shift-Enter is a new line, and the box grows to what
 * you have written and stops at ten lines so it can never eat the room.
 *
 * Two things sit above it. A reply banner, when you are answering somebody —
 * visible the whole time you are typing, because a reply you forgot you were
 * writing lands in the wrong place. And one line that says either what went
 * wrong with the last send or who else is mid-sentence. That line keeps its
 * height whether or not it has anything to say, so nothing below the stream
 * ever jumps.
 */
import { type FormEvent, type KeyboardEvent, useEffect, useRef, useState } from "react";

import type { Message } from "../generated/Message";
import type { Room } from "../generated/Room";
import type { User } from "../generated/User";
import { ApiError, type AuthedApi } from "../lib/api";
import { useAutoGrow } from "../lib/autogrow";
import { sendMessage, startTyping } from "../lib/gateway";
import { MAX_MESSAGE_CHARS } from "../lib/limits";
import { plainText } from "./markdown";

/** Show what is left only once it is close enough to matter. */
const COUNTDOWN_FROM = 200;

/** How much of the message you are answering fits in the banner. */
const QUOTE_CHARS = 90;

interface ComposerProps {
  api: AuthedApi;
  room: Room;
  /** The message being answered, if any. */
  replyTo: Message | null;
  replyToAuthor: User | undefined;
  onClearReply: () => void;
  /** Display names of everyone else typing in this room, right now. */
  typing: string[];
}

export default function Composer({
  api,
  room,
  replyTo,
  replyToAuthor,
  onClearReply,
  typing,
}: ComposerProps) {
  const [draft, setDraft] = useState("");
  const [problem, setProblem] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const field = useRef<HTMLTextAreaElement | null>(null);
  useAutoGrow(field, draft);

  // Switching rooms should not carry a half-typed line into the new one.
  useEffect(() => {
    setDraft("");
    setProblem(null);
  }, [room.id]);

  // Choosing to reply puts the cursor where you are about to type.
  useEffect(() => {
    if (replyTo !== null) field.current?.focus();
  }, [replyTo]);

  const submit = async (event: FormEvent): Promise<void> => {
    event.preventDefault();
    const body = draft.trim();
    if (body.length === 0 || sending) return;
    setSending(true);
    try {
      await sendMessage(api, room.id, body, replyTo?.id ?? null);
      setDraft("");
      setProblem(null);
      onClearReply();
    } catch (error) {
      // The composer is the only thing holding what they typed, so it is the
      // only thing that can tell them it did not go — and it keeps the text.
      setProblem(error instanceof ApiError ? error.message : "Couldn't reach the server.");
    } finally {
      setSending(false);
    }
  };

  const keys = (event: KeyboardEvent<HTMLTextAreaElement>): void => {
    if (event.key === "Escape" && replyTo !== null) {
      event.preventDefault();
      onClearReply();
      return;
    }
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void submit(event);
    }
  };

  const typed = (value: string): void => {
    setDraft(value);
    // Say it on the way in, not on a timer: the store throttles this to the one
    // frame every four seconds the server will take (PROTOCOL §8).
    if (value.length > 0) void startTyping(room.id);
  };

  const left = MAX_MESSAGE_CHARS - draft.length;

  return (
    <form className="composer" onSubmit={(event) => void submit(event)}>
      {replyTo === null ? null : (
        <div className="composer-reply">
          <span className="composer-reply-mark meta" aria-hidden="true">
            ↳
          </span>
          <span className="composer-reply-text">
            replying to {replyToAuthor?.display_name ?? "someone"}:{" "}
            {plainText(replyTo.body).slice(0, QUOTE_CHARS) || "a message"}
          </span>
          <button
            type="button"
            className="composer-reply-drop meta"
            onClick={onClearReply}
            aria-label="stop replying"
          >
            ×
          </button>
        </div>
      )}

      {/* Only the problem is announced. A typing line that came and went in a
          screen reader's ear every few seconds would be the noisiest thing in
          the app, and it is the least important thing on the screen. */}
      <p className="composer-line meta">
        {problem === null ? typingLine(typing) : <span role="alert">{problem}</span>}
      </p>

      <div className="composer-row">
        <textarea
          ref={field}
          className="composer-input"
          value={draft}
          rows={1}
          maxLength={MAX_MESSAGE_CHARS}
          onChange={(event) => typed(event.target.value)}
          onKeyDown={keys}
          placeholder={`say something in #${room.slug}`}
          aria-label={`message #${room.slug}`}
          autoComplete="off"
        />
        {left <= COUNTDOWN_FROM ? <span className="composer-left meta">{left}</span> : null}
        <button className="composer-send" type="submit" disabled={draft.trim().length === 0}>
          send
        </button>
      </div>
    </form>
  );
}

/**
 * Who is typing, in words. Past two people it stops naming them: the point is
 * that the room is busy, and a list of five names is a different sentence.
 */
function typingLine(names: string[]): string {
  if (names.length === 0) return "";
  if (names.length === 1) return `${names[0]} is typing…`;
  if (names.length === 2) return `${names[0]} and ${names[1]} are typing…`;
  return "several people are typing…";
}
