/**
 * The Tauri side of session storage.
 *
 * These types mirror `src-tauri/src/secrets.rs` by hand. That is allowed:
 * AGENTS rule 7 covers types crossing the *wire*, and none of these ever leave
 * the machine. A Rust test in that file pins the `kind` spellings so the two
 * halves can't drift silently.
 *
 * Running `pnpm dev` in a plain browser (no Tauri) is a supported way to work on
 * the UI, and it behaves exactly like a computer with no keyring: nothing is
 * remembered, and the app says so. The in-memory list still lets you add a
 * second server for the rest of this run.
 */
import { invoke, isTauri } from "@tauri-apps/api/core";

export interface StoredSession {
  /** Origin of the server, e.g. `https://linger.example`. No trailing slash. */
  base_url: string;
  refresh_token: string;
}

export type SessionLoad =
  | { kind: "found"; sessions: StoredSession[]; active: string | null }
  | { kind: "empty" }
  | { kind: "unavailable"; reason: string };

export type SessionWrite = { kind: "done" } | { kind: "unavailable"; reason: string };

const NO_TAURI = "Running in a browser, so nothing is saved between reloads.";

/** In-memory stand-in used when there is no keyring (or no Tauri). */
let memory: { sessions: StoredSession[]; active: string | null } | null = null;

export async function loadSessions(): Promise<SessionLoad> {
  if (!isTauri()) {
    if (memory === null || memory.sessions.length === 0) {
      return { kind: "unavailable", reason: NO_TAURI };
    }
    return { kind: "found", sessions: memory.sessions, active: memory.active };
  }
  const result: SessionLoad = await invoke("session_load");
  return result;
}

export async function saveSession(session: StoredSession): Promise<SessionWrite> {
  if (!isTauri()) {
    const list = memory ?? { sessions: [], active: null };
    const at = list.sessions.findIndex((held) => held.base_url === session.base_url);
    if (at >= 0) list.sessions[at] = session;
    else list.sessions.push(session);
    list.active = session.base_url;
    memory = list;
    return { kind: "unavailable", reason: NO_TAURI };
  }
  const result: SessionWrite = await invoke("session_save", { session });
  return result;
}

/** Forget one server, or every server when `baseUrl` is omitted. */
export async function clearSession(baseUrl?: string): Promise<SessionWrite> {
  if (!isTauri()) {
    if (memory === null) return { kind: "unavailable", reason: NO_TAURI };
    if (baseUrl === undefined) {
      memory = null;
    } else {
      memory.sessions = memory.sessions.filter((held) => held.base_url !== baseUrl);
      if (memory.active === baseUrl) {
        memory.active = memory.sessions[0]?.base_url ?? null;
      }
      if (memory.sessions.length === 0) memory = null;
    }
    return { kind: "unavailable", reason: NO_TAURI };
  }
  const result: SessionWrite = await invoke("session_clear", {
    baseUrl: baseUrl ?? null,
  });
  return result;
}

export async function setActiveSession(baseUrl: string): Promise<SessionWrite> {
  if (!isTauri()) {
    if (memory && memory.sessions.some((held) => held.base_url === baseUrl)) {
      memory.active = baseUrl;
    }
    return { kind: "unavailable", reason: NO_TAURI };
  }
  const result: SessionWrite = await invoke("session_set_active", { baseUrl });
  return result;
}
