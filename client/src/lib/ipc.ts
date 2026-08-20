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
 * remembered, and the app says so.
 */
import { invoke, isTauri } from "@tauri-apps/api/core";

export interface StoredSession {
  /** Origin of the server, e.g. `https://linger.example`. No trailing slash. */
  base_url: string;
  refresh_token: string;
}

export type SessionLoad =
  | { kind: "found"; session: StoredSession }
  | { kind: "empty" }
  | { kind: "unavailable"; reason: string };

export type SessionWrite = { kind: "done" } | { kind: "unavailable"; reason: string };

const NO_TAURI = "Running in a browser, so nothing is saved between reloads.";

export async function loadSession(): Promise<SessionLoad> {
  if (!isTauri()) return { kind: "unavailable", reason: NO_TAURI };
  const result: SessionLoad = await invoke("session_load");
  return result;
}

export async function saveSession(session: StoredSession): Promise<SessionWrite> {
  if (!isTauri()) return { kind: "unavailable", reason: NO_TAURI };
  const result: SessionWrite = await invoke("session_save", { session });
  return result;
}

export async function clearSession(): Promise<SessionWrite> {
  if (!isTauri()) return { kind: "unavailable", reason: NO_TAURI };
  const result: SessionWrite = await invoke("session_clear");
  return result;
}
