/**
 * The voice surface (SPEC §4.14, T-1404): one line under the room header.
 *
 * Voice happens *in a room*, so the surface lives in the room rather than
 * in a panel of its own: a way in, a way out, who is talking, and the two
 * things that are yours alone — mute and how loud each person is. There is
 * no call to answer and nothing rings; the line is empty until somebody is
 * in voice here, and then it says who.
 *
 * Console rules apply (SPEC §5): no bubbles, no glow, no animated rings.
 * Somebody talking is their name drawn a little brighter, the way a live
 * status is drawn anywhere else in the app.
 */
import { useEffect, useState } from "react";

import type { Room } from "../generated/Room";
import type { User } from "../generated/User";
import type { AuthedApi } from "../lib/api";
import {
  joinVoice,
  leaveVoice,
  setVoiceMuted,
  setVoiceVolume,
  useGateway,
  voicePeersIn,
} from "../lib/gateway";
import { nameProps } from "../lib/names";
import {
  clampVolume,
  loadVoicePrefs,
  microphoneLine,
  PUSH_TO_TALK_KEY,
  seatsOf,
  volumeLabel,
} from "./voice";
import "./voice.css";

export default function VoiceBar({
  api,
  room,
  users,
}: {
  api: AuthedApi;
  room: Room;
  users: User[];
}) {
  const gateway = useGateway(api.baseUrl);
  const peers = voicePeersIn(gateway, room.id);
  const mine = gateway.myVoice;
  const seatedHere = mine !== null && mine.roomId === room.id;
  const seatedElsewhere = mine !== null && mine.roomId !== room.id;
  const [joining, setJoining] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  // Read once per join rather than subscribed: a device changed in settings
  // applies to the next join, which is the honest promise and the simple one.
  const join = async (): Promise<void> => {
    const prefs = loadVoicePrefs();
    setJoining(true);
    setProblem(null);
    try {
      await joinVoice(api, room.id, prefs.devices, prefs.pushToTalk);
    } catch (error) {
      setProblem(error instanceof Error ? error.message : "Couldn't join voice.");
    } finally {
      setJoining(false);
    }
  };

  // Push-to-talk: hold the key, the microphone opens; let go, it closes.
  // Attached only while seated with push-to-talk on, so an idle window has
  // no key listener at all.
  const pushToTalk = seatedHere && loadVoicePrefs().pushToTalk;
  useEffect(() => {
    if (!pushToTalk) return;
    const server = api.baseUrl;
    const down = (event: KeyboardEvent): void => {
      if (event.key === PUSH_TO_TALK_KEY && !event.repeat) setVoiceMuted(server, false);
    };
    const up = (event: KeyboardEvent): void => {
      if (event.key === PUSH_TO_TALK_KEY) setVoiceMuted(server, true);
    };
    // Losing the window mid-word must not leave the microphone open.
    const blur = (): void => setVoiceMuted(server, true);
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    window.addEventListener("blur", blur);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
      window.removeEventListener("blur", blur);
    };
  }, [pushToTalk, api.baseUrl]);

  // Nothing to say: nobody here is in voice and neither are you.
  if (peers.length === 0 && !seatedHere && problem === null) {
    return (
      <div className="voice-bar" data-empty="true">
        <button
          type="button"
          className="voice-action meta"
          disabled={joining}
          onClick={() => void join()}
        >
          {joining ? "joining voice…" : seatedElsewhere ? "move voice here" : "join voice"}
        </button>
      </div>
    );
  }

  const seats = seatsOf(peers, users, gateway.sessionId);
  const line = seatedHere ? microphoneLine(mine.audio, pushToTalk, mine.muted) : null;

  return (
    <div className="voice-bar" aria-label="voice">
      <span className="voice-label meta">voice</span>
      <ul className="voice-seats">
        {seats.map((seat) => {
          const talking = seat.isMe
            ? (seatedHere && mine.talking)
            : (seatedHere && (mine.speaking[seat.sessionId] ?? false));
          const link = seatedHere && !seat.isMe ? mine.peers[seat.sessionId] : undefined;
          return (
            <li
              key={seat.sessionId}
              className="voice-seat"
              data-talking={talking ? "true" : undefined}
              data-link={link}
            >
              <span {...nameProps(seat.user, "voice-name")}>{seat.name}</span>
              {seat.isMe ? <span className="meta">you</span> : null}
              {/* A screen reader gets the word; sighted people get the weight. */}
              {talking ? <span className="sr-only">talking</span> : null}
              {link === "connecting" || link === "new" ? (
                <span className="meta">connecting…</span>
              ) : link === "failed" || link === "disconnected" ? (
                <span className="meta">can't reach</span>
              ) : null}
              {seatedHere && !seat.isMe ? (
                <Volume
                  value={mine.volumes[seat.sessionId] ?? 1}
                  name={seat.name}
                  onChange={(volume) => setVoiceVolume(api.baseUrl, seat.sessionId, volume)}
                />
              ) : null}
            </li>
          );
        })}
      </ul>
      {seatedHere ? (
        <>
          {line === null ? null : <span className="voice-line meta">{line}</span>}
          {pushToTalk ? null : (
            <button
              type="button"
              className="voice-action meta"
              aria-pressed={mine.muted}
              onClick={() => setVoiceMuted(api.baseUrl, !mine.muted)}
            >
              {mine.muted ? "muted" : "mute"}
            </button>
          )}
          <button
            type="button"
            className="voice-action meta"
            onClick={() => void leaveVoice(api.baseUrl)}
          >
            leave voice
          </button>
        </>
      ) : (
        <button
          type="button"
          className="voice-action meta"
          disabled={joining}
          onClick={() => void join()}
        >
          {joining ? "joining voice…" : seatedElsewhere ? "move voice here" : "join voice"}
        </button>
      )}
      {problem === null ? null : <span className="voice-problem meta">{problem}</span>}
    </div>
  );
}

/**
 * How loud one person is, for you. A plain range: 0 is silent, the middle is
 * as sent, the top is twice that. The number beside it is metadata, so mono.
 */
function Volume({
  value,
  name,
  onChange,
}: {
  value: number;
  name: string;
  onChange: (volume: number) => void;
}) {
  return (
    <label className="voice-volume">
      <span className="sr-only">volume for {name}</span>
      <input
        type="range"
        min={0}
        max={2}
        step={0.05}
        value={value}
        onChange={(event) => onChange(clampVolume(Number(event.target.value)))}
      />
      <span className="meta">{volumeLabel(value)}</span>
    </label>
  );
}
