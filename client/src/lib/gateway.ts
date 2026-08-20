/**
 * The gateway store: the frontend's one piece of shared state.
 *
 * AGENTS allows local component state plus exactly one gateway store, and this
 * is it — about eighty lines of subscribe-and-notify instead of a state
 * library. React reads it through `useSyncExternalStore`, so every component
 * that cares re-renders on a change and nothing else does.
 *
 * The connection itself is not here. It lives in the Tauri core
 * (ARCHITECTURE §1), which owns reconnecting, resume and sequence numbers and
 * sends two events up: a connection status, and each sequenced server frame.
 * This file's whole job is to fold those frames into a snapshot the UI can
 * render, and to answer when the core says it needs a fresh access token.
 *
 * Running `pnpm dev` in a plain browser, with no Tauri underneath, leaves the
 * status on `offline` — the same honest degrading as session storage.
 */
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useSyncExternalStore } from "react";

import type { ClientFrame } from "../generated/ClientFrame";
import type { PresenceEntry } from "../generated/PresenceEntry";
import type { Room } from "../generated/Room";
import type { ServerFrame } from "../generated/ServerFrame";
import type { User } from "../generated/User";
import type { UserId } from "../generated/UserId";
import type { AuthedApi } from "./api";

/**
 * Mirrors `Status` in `src-tauri/src/gateway.rs`. Allowed to be hand-written:
 * AGENTS rule 7 covers types that cross the *wire*, and this one never leaves
 * the machine. A Rust test pins the `kind` spellings so the halves can't drift.
 */
export type GatewayStatus =
  | { kind: "offline" }
  | { kind: "connecting" }
  | { kind: "connected"; tls: boolean }
  | { kind: "identifying" }
  | { kind: "resuming" }
  | { kind: "ready"; latency_ms: number }
  | { kind: "waiting"; retry_in_ms: number; reason: string }
  | { kind: "needs_token" };

export interface GatewayState {
  status: GatewayStatus;
  /** Who the server says we are. Null until the first `ready`. */
  me: User | null;
  users: User[];
  rooms: Room[];
  presence: PresenceEntry[];
  /** Who is in each room, keyed by room id. */
  occupancy: Record<string, UserId[]>;
}

const EMPTY: GatewayState = {
  status: { kind: "offline" },
  me: null,
  users: [],
  rooms: [],
  presence: [],
  occupancy: {},
};

let state: GatewayState = EMPTY;
const listeners = new Set<() => void>();

function publish(next: GatewayState): void {
  state = next;
  for (const notify of listeners) notify();
}

function subscribe(notify: () => void): () => void {
  listeners.add(notify);
  return () => {
    listeners.delete(notify);
  };
}

/** Subscribe a component to the gateway. */
export function useGateway(): GatewayState {
  return useSyncExternalStore(subscribe, () => state);
}

// ---------------------------------------------------------------------------
// Folding frames into the snapshot
// ---------------------------------------------------------------------------

/** Replace an entry keyed by id, or append it if it's new. */
function upsert<T>(list: T[], item: T, sameAs: (candidate: T) => boolean): T[] {
  const at = list.findIndex(sameAs);
  if (at < 0) return [...list, item];
  const next = [...list];
  next[at] = item;
  return next;
}

function occupancyFrom(presence: PresenceEntry[]): Record<string, UserId[]> {
  const rooms: Record<string, UserId[]> = {};
  for (const entry of presence) {
    if (entry.room_id === null) continue;
    (rooms[entry.room_id] ??= []).push(entry.user_id);
  }
  return rooms;
}

/**
 * Apply one server frame. Message events land in T-303 — they arrive here
 * already and are ignored on purpose rather than half-stored.
 */
function apply(current: GatewayState, frame: ServerFrame): GatewayState {
  switch (frame.op) {
    case "ready":
      // A fresh `ready` replaces everything. It arrives after a re-identify,
      // which is exactly when our copy is the thing that can't be trusted.
      return {
        ...current,
        me: frame.d.user,
        users: frame.d.users,
        rooms: frame.d.rooms,
        presence: frame.d.presence,
        occupancy: occupancyFrom(frame.d.presence),
      };
    case "presence.update": {
      const entry = frame.d;
      return {
        ...current,
        presence: upsert(current.presence, entry, (p) => p.user_id === entry.user_id),
      };
    }
    case "user.update": {
      const user = frame.d;
      return {
        ...current,
        me: current.me?.id === user.id ? user : current.me,
        users: upsert(current.users, user, (u) => u.id === user.id),
      };
    }
    case "room.create":
    case "room.update": {
      const room = frame.d;
      return { ...current, rooms: upsert(current.rooms, room, (r) => r.id === room.id) };
    }
    case "room.occupancy":
      return {
        ...current,
        occupancy: { ...current.occupancy, [frame.d.room_id]: frame.d.user_ids },
      };
    default:
      return current;
  }
}

// ---------------------------------------------------------------------------
// Talking to the Tauri core
// ---------------------------------------------------------------------------

/** The signed-in connection this store is currently following. */
let connected: AuthedApi | null = null;
let unlisteners: UnlistenFn[] = [];
/**
 * The token we last handed down. Cleared once the handshake works, so a second
 * request for a token means the last one was refused rather than expired — the
 * case where only forcing a refresh gets us back on.
 */
let givenToken: string | null = null;
let supplying: Promise<void> | null = null;

async function supplyToken(): Promise<void> {
  if (supplying) return supplying;
  const api = connected;
  if (!api) return;
  supplying = (async () => {
    try {
      const fresh = await api.accessToken(givenToken !== null);
      givenToken = fresh.token;
      await invoke("gateway_token", {
        token: fresh.token,
        expiresAtMs: fresh.expiresAt,
      });
    } catch {
      // A refused refresh already ends the session through `AuthedApi`; a
      // network failure is temporary and the core will ask again.
    } finally {
      supplying = null;
    }
  })();
  return supplying;
}

/** Open the connection and start following it. Replaces any previous one. */
export async function connect(api: AuthedApi): Promise<void> {
  await disconnect();
  connected = api;
  if (!isTauri()) return;

  const following = api;
  const attach = async (): Promise<void> => {
    const stop = [
      await listen<GatewayStatus>("gateway:status", (event) => {
        if (connected !== following) return;
        // Nothing else knows the token was accepted, so this is where we learn
        // it, and where a later request means "refused", not "expired".
        if (event.payload.kind === "ready") givenToken = null;
        if (event.payload.kind === "needs_token") void supplyToken();
        publish({ ...state, status: event.payload });
      }),
      await listen<ServerFrame>("gateway:frame", (event) => {
        if (connected !== following) return;
        publish(apply(state, event.payload));
      }),
    ];
    // A disconnect that landed while we were attaching: don't leak listeners.
    if (connected !== following) {
      for (const off of stop) off();
      return;
    }
    unlisteners = stop;
  };
  await attach();

  const token = await api.accessToken();
  givenToken = token.token;
  await invoke("gateway_connect", {
    baseUrl: api.baseUrl,
    token: token.token,
    expiresAtMs: token.expiresAt,
  });
}

/** Close the connection and forget everything it told us. */
export async function disconnect(): Promise<void> {
  connected = null;
  givenToken = null;
  for (const off of unlisteners) off();
  unlisteners = [];
  publish(EMPTY);
  if (isTauri()) await invoke("gateway_disconnect");
}

/**
 * Send one frame upstream. `false` means there was no live connection, which
 * callers treat as "say it again after the next `ready`" rather than an error.
 */
export async function send(frame: ClientFrame): Promise<boolean> {
  if (!isTauri()) return false;
  const sent: boolean = await invoke("gateway_send", { frame });
  return sent;
}

/** The status bar line (SPEC §5.6): protocol text, never a spinner. */
export function statusText(status: GatewayStatus): string {
  switch (status.kind) {
    case "offline":
      return "offline";
    case "connecting":
      return "connecting…";
    case "connected":
      // Only claim TLS when there was a TLS handshake. A server on your own
      // machine over plain http gets the honest word instead.
      return status.tls ? "tls ok…" : "socket ok…";
    case "identifying":
      return "identify…";
    case "resuming":
      return "resume…";
    case "ready":
      return `ready (${status.latency_ms}ms)`;
    case "waiting":
      return `retry in ${Math.max(1, Math.round(status.retry_in_ms / 1000))}s…`;
    case "needs_token":
      return "renewing…";
  }
}
