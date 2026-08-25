/**
 * The Tauri side of session storage.
 *
 * These types mirror `src-tauri/src/secrets.rs` by hand. That is allowed:
 * AGENTS rule 7 covers types crossing the *wire*, and none of these ever leave
 * the machine. A Rust test in that file pins the `kind` spellings so the two
 * halves can't drift silently.
 *
 * Since T-412 this is a list: one stored session per server, keyed by base URL.
 * Saving one replaces only that server's entry, and forgetting one leaves the
 * rest signed in.
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

export type SessionsLoad =
  | { kind: "found"; sessions: StoredSession[] }
  | { kind: "empty" }
  | { kind: "unavailable"; reason: string };

export type SessionWrite = { kind: "done" } | { kind: "unavailable"; reason: string };

const NO_TAURI = "Running in a browser, so nothing is saved between reloads.";

/** Every server we have a sign-in for, in the order they were added. */
export async function loadSessions(): Promise<SessionsLoad> {
  if (!isTauri()) return { kind: "unavailable", reason: NO_TAURI };
  const result: SessionsLoad = await invoke("sessions_load");
  return result;
}

export async function saveSession(session: StoredSession): Promise<SessionWrite> {
  if (!isTauri()) return { kind: "unavailable", reason: NO_TAURI };
  const result: SessionWrite = await invoke("session_save", { session });
  return result;
}

/** Sign out of one server. The others are untouched. */
export async function forgetSession(baseUrl: string): Promise<SessionWrite> {
  if (!isTauri()) return { kind: "unavailable", reason: NO_TAURI };
  const result: SessionWrite = await invoke("session_forget", { baseUrl });
  return result;
}
