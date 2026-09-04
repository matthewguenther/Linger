/**
 * The store's part in voice (SPEC §4.14, T-1404): folding the server's
 * `voice.state`, holding our own seat, and talking to the core.
 *
 * Same shape as `gateway.test.ts` — the core is mocked down to the events
 * the store listens for and the commands it sends — and kept apart from it
 * because voice has its own three events and its own failure modes.
 */
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ReadyData } from "../generated/ReadyData";
import type { ServerFrame } from "../generated/ServerFrame";
import type { User } from "../generated/User";
import type { AuthedApi } from "./api";

const invoked: { cmd: string; args: Record<string, unknown> }[] = [];
/** Commands that should reject, to see what the store does with a refusal. */
const failing = new Set<string>();

vi.mock("@tauri-apps/api/core", () => ({
  isTauri: () => true,
  invoke: async (cmd: string, args: Record<string, unknown>): Promise<boolean> => {
    invoked.push({ cmd, args });
    if (failing.has(cmd)) throw new Error(`no ${cmd} today`);
    return true;
  },
}));

type Handler = (event: { payload: unknown }) => void;
const handlers = new Map<string, Handler>();

vi.mock("@tauri-apps/api/event", () => ({
  listen: async (name: string, handler: Handler): Promise<() => void> => {
    handlers.set(name, handler);
    return () => handlers.delete(name);
  },
}));

vi.mock("../notify/notify", () => ({ considerFrame: () => undefined }));
vi.mock("./sound", () => ({ playKnock: () => false }));

const {
  connect,
  disconnect,
  joinVoice,
  leaveVoice,
  serverState,
  setVoiceMuted,
  setVoiceVolume,
  voicePeersIn,
} = await import("./gateway");

const HOME = "https://home.example";
const WORK = "https://work.example";

function fakeApi(baseUrl: string): AuthedApi {
  const stub = {
    baseUrl,
    accessToken: async () => ({ token: `token-${baseUrl}`, expiresAt: 0 }),
    get: async (path: string) => {
      throw new Error(`unexpected GET ${path}`);
    },
  };
  return stub as unknown as AuthedApi;
}

function person(id: string, name: string): User {
  return {
    id,
    username: name,
    display_name: name,
    is_host: false,
    style: {
      font_key: "inter",
      weight: 400,
      italic: false,
      fill: { kind: "solid", color: "azure" },
      effect: "none",
      msg_font_key: null,
    },
    status: null,
    entrance_sound: null,
    last_seen_at: null,
  };
}

const MATT = person("u-matt", "Matt");

function ready(data: Partial<ReadyData> = {}): ServerFrame {
  return {
    s: 1,
    op: "ready",
    d: {
      session_id: "s-me",
      user: MATT,
      users: [MATT],
      rooms: [],
      dms: [],
      presence: [],
      ...data,
    },
  };
}

function voiceState(roomId: string, seats: [string, string][]): ServerFrame {
  return {
    s: 2,
    op: "voice.state",
    d: {
      room_id: roomId,
      peers: seats.map(([session_id, user_id]) => ({ session_id, user_id })),
    },
  };
}

function arrive(server: string, frame: ServerFrame): void {
  handlers.get("gateway:frame")?.({ payload: { server, frame } });
}

function coreEvent(name: string, payload: unknown): void {
  handlers.get(name)?.({ payload });
}

const DEFAULTS = { input: null, output: null };

async function seated(server: string): Promise<void> {
  await connect(fakeApi(server));
  arrive(server, ready());
  await joinVoice(fakeApi(server), "r-garage", DEFAULTS, false);
}

describe("voice in the store", () => {
  beforeEach(async () => {
    await Promise.all([disconnect(HOME), disconnect(WORK)]);
    invoked.length = 0;
    failing.clear();
  });

  it("remembers the session id from ready, and who is in voice per room", async () => {
    await connect(fakeApi(HOME));
    arrive(HOME, ready());
    expect(serverState(HOME).sessionId).toBe("s-me");

    arrive(HOME, voiceState("r-garage", [["s-1", "u-amy"], ["s-2", "u-zed"]]));
    expect(voicePeersIn(serverState(HOME), "r-garage").map((p) => p.session_id)).toEqual([
      "s-1",
      "s-2",
    ]);
    expect(voicePeersIn(serverState(HOME), "r-else")).toEqual([]);

    // The whole list every time: a shorter one replaces, an empty one removes.
    arrive(HOME, voiceState("r-garage", [["s-2", "u-zed"]]));
    expect(voicePeersIn(serverState(HOME), "r-garage").map((p) => p.session_id)).toEqual(["s-2"]);
    arrive(HOME, voiceState("r-garage", []));
    expect(serverState(HOME).voice).toEqual({});
  });

  it("joins with the session id and the chosen devices, muting first for push-to-talk", async () => {
    await connect(fakeApi(HOME));
    arrive(HOME, ready());
    invoked.length = 0;

    await joinVoice(fakeApi(HOME), "r-garage", { input: "USB Mic", output: null }, true);

    const calls = invoked.filter((call) => call.cmd.startsWith("voice_"));
    expect(calls.map((call) => call.cmd)).toEqual(["voice_mute", "voice_join"]);
    expect(calls[0]?.args).toEqual({ baseUrl: HOME, muted: true });
    expect(calls[1]?.args).toEqual({
      baseUrl: HOME,
      sessionId: "s-me",
      roomId: "r-garage",
      input: "USB Mic",
      output: null,
    });
    expect(serverState(HOME).myVoice).toMatchObject({
      roomId: "r-garage",
      muted: true,
      audio: "opening",
    });
  });

  it("refuses to join before the session exists", async () => {
    await connect(fakeApi(HOME));
    await expect(joinVoice(fakeApi(HOME), "r-garage", DEFAULTS, false)).rejects.toThrow(
      /not connected/i,
    );
    expect(serverState(HOME).myVoice).toBeNull();
  });

  it("gives up a seat on one server before taking one on another", async () => {
    await seated(HOME);
    await connect(fakeApi(WORK));
    arrive(WORK, ready());
    invoked.length = 0;

    await joinVoice(fakeApi(WORK), "r-standup", DEFAULTS, false);

    const calls = invoked.filter((call) => call.cmd.startsWith("voice_"));
    expect(calls.map((call) => [call.cmd, call.args.baseUrl])).toEqual([
      ["voice_leave", HOME],
      ["voice_mute", WORK],
      ["voice_join", WORK],
    ]);
    expect(serverState(HOME).myVoice).toBeNull();
    expect(serverState(WORK).myVoice?.roomId).toBe("r-standup");
  });

  it("a refused join leaves no seat behind and says why", async () => {
    await connect(fakeApi(HOME));
    arrive(HOME, ready());
    failing.add("voice_join");

    await expect(joinVoice(fakeApi(HOME), "r-garage", DEFAULTS, false)).rejects.toThrow(
      /no voice_join today/,
    );
    expect(serverState(HOME).myVoice).toBeNull();
  });

  it("loses the seat, and tells the core, when the server's list no longer has us", async () => {
    await seated(HOME);
    arrive(HOME, voiceState("r-garage", [["s-me", "u-matt"], ["s-1", "u-amy"]]));
    expect(serverState(HOME).myVoice).not.toBeNull();
    invoked.length = 0;

    // Somebody else's room changing is not our business.
    arrive(HOME, voiceState("r-else", [["s-9", "u-zed"]]));
    expect(serverState(HOME).myVoice).not.toBeNull();

    // Our room, without us: the seat is gone and the microphone must go too.
    arrive(HOME, voiceState("r-garage", [["s-1", "u-amy"]]));
    expect(serverState(HOME).myVoice).toBeNull();
    expect(invoked.filter((call) => call.cmd === "voice_leave")).toHaveLength(1);
  });

  it("folds the core's own events into the seat, and ignores them without one", async () => {
    await seated(HOME);

    coreEvent("voice:peer", { server: HOME, peer: "s-1", state: "connected" });
    coreEvent("voice:audio", { server: HOME, state: "sending" });
    coreEvent("voice:speaking", { server: HOME, peer: "s-1", speaking: true });
    coreEvent("voice:speaking", { server: HOME, peer: null, speaking: true });

    expect(serverState(HOME).myVoice).toMatchObject({
      peers: { "s-1": "connected" },
      audio: "sending",
      speaking: { "s-1": true },
      talking: true,
    });

    await leaveVoice(HOME);
    expect(serverState(HOME).myVoice).toBeNull();
    coreEvent("voice:speaking", { server: HOME, peer: "s-1", speaking: false });
    expect(serverState(HOME).myVoice).toBeNull();
  });

  it("mute and volume are remembered and sent, and never leave this machine", async () => {
    await seated(HOME);
    invoked.length = 0;

    setVoiceMuted(HOME, true);
    setVoiceVolume(HOME, "s-1", 1.5);

    expect(serverState(HOME).myVoice).toMatchObject({ muted: true, volumes: { "s-1": 1.5 } });
    const calls = invoked.filter((call) => call.cmd.startsWith("voice_"));
    expect(calls.map((call) => call.cmd)).toEqual(["voice_mute", "voice_volume"]);
    // Nothing went to the gateway: mute and volume are not wire frames.
    expect(invoked.some((call) => call.cmd === "gateway_send")).toBe(false);
  });

  it("a fresh ready is a fresh session, so the seat and the lists go with it", async () => {
    await seated(HOME);
    arrive(HOME, voiceState("r-garage", [["s-me", "u-matt"]]));

    arrive(HOME, ready({ session_id: "s-me-2" }));

    expect(serverState(HOME).sessionId).toBe("s-me-2");
    expect(serverState(HOME).myVoice).toBeNull();
    expect(serverState(HOME).voice).toEqual({});
  });
});
