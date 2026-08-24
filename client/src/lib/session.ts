/**
 * Who is signed in, and staying signed in across restarts.
 *
 * T-412: this is a list. Each server has its own tokens, its own account, its
 * own connection. Signing out of one must not touch the others.
 *
 * The shape of it: each refresh token lives in the OS keyring, and on every
 * start the client trades each one for a fresh access token and asks that
 * server who that is. Nothing about the account is cached locally, so a name
 * or style changed on another device is right the moment the app opens.
 *
 * AGENTS allows local state plus one gateway store. This is the local half; the
 * gateway store is `./gateway.ts`. The split is deliberate — who you are outlives
 * any one connection, and the connection is not this file's business.
 */
import { useCallback, useEffect, useRef, useState } from "react";

import type { AuthResponse } from "../generated/AuthResponse";
import type { User } from "../generated/User";
import { ApiError, AuthedApi, expiryOf, PublicApi, type Tokens } from "./api";
import { clearSession, loadSessions, saveSession, setActiveSession } from "./ipc";

export interface ServerEntry {
  api: AuthedApi;
  user: User;
}

export type SessionState =
  | { status: "restoring" }
  | { status: "signed_out" }
  | { status: "signed_in"; servers: ServerEntry[]; active: string };

export interface Session {
  state: SessionState;
  /**
   * Set when this computer can't remember a sign-in — no wallet, a locked one,
   * or the UI running in a plain browser. Screens show it so the person finds
   * out before they're surprised by it, not after.
   */
  keyringNotice: string | null;
  /** Why the last sign-in ended, when it ended on its own. */
  notice: string | null;
  signIn: (baseUrl: string, auth: AuthResponse) => Promise<void>;
  signOut: (baseUrl?: string) => Promise<void>;
  setActive: (baseUrl: string) => void;
}

/**
 * Restoring must happen exactly once per launch. Refresh tokens rotate and
 * spending one twice revokes the whole family (PROTOCOL §2), so React's
 * StrictMode double-mount in development would otherwise sign the user out —
 * a bug that would only ever appear on a dev machine. Keeping the promise at
 * module scope makes the second mount await the first attempt instead of
 * starting its own.
 */
let restoreOnce: Promise<Restored> | null = null;

interface Restored {
  entries: Array<{ baseUrl: string; tokens: Tokens }>;
  active: string | null;
  keyringNotice: string | null;
  notice: string | null;
}

async function restore(): Promise<Restored> {
  const loaded = await loadSessions();
  if (loaded.kind === "unavailable") {
    return { entries: [], active: null, keyringNotice: loaded.reason, notice: null };
  }
  if (loaded.kind === "empty") {
    return { entries: [], active: null, keyringNotice: null, notice: null };
  }

  const entries: Array<{ baseUrl: string; tokens: Tokens }> = [];
  let notice: string | null = null;
  for (const session of loaded.sessions) {
    try {
      const fresh = await new PublicApi(session.base_url).refresh({
        refresh_token: session.refresh_token,
      });
      await saveSession({
        base_url: session.base_url,
        refresh_token: fresh.refresh_token,
      });
      entries.push({
        baseUrl: session.base_url,
        tokens: {
          accessToken: fresh.access_token,
          refreshToken: fresh.refresh_token,
          expiresAt: expiryOf(fresh.expires_in),
        },
      });
    } catch (error) {
      // A refused token is genuinely dead, so forget it. Anything else — the
      // server is down, the laptop is on a train — leaves it in place, because
      // it will very likely work on the next launch. Other servers are not
      // dropped because one of them is unreachable.
      const refused =
        error instanceof ApiError &&
        (error.code === "UNAUTHENTICATED" || error.code === "FORBIDDEN");
      if (refused) await clearSession(session.base_url);
      notice =
        error instanceof Error
          ? refused
            ? "A saved sign-in is no longer valid. Please sign in again."
            : error.message
          : "Couldn't restore a saved sign-in.";
    }
  }

  const active =
    (loaded.active !== null && entries.some((entry) => entry.baseUrl === loaded.active)
      ? loaded.active
      : null) ??
    entries[0]?.baseUrl ??
    null;

  if (active !== null) await setActiveSession(active);

  return { entries, active, keyringNotice: null, notice };
}

export function useSession(): Session {
  const [state, setState] = useState<SessionState>({ status: "restoring" });
  const [keyringNotice, setKeyringNotice] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const mounted = useRef(true);
  const stateRef = useRef(state);
  stateRef.current = state;

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const dropServer = useCallback(async (baseUrl: string, reason: string | null) => {
    await clearSession(baseUrl);
    if (!mounted.current) return;
    const current = stateRef.current;
    if (current.status !== "signed_in") return;
    const servers = current.servers.filter((entry) => entry.api.baseUrl !== baseUrl);
    if (servers.length === 0) {
      if (reason) setNotice(reason);
      setState({ status: "signed_out" });
      return;
    }
    const active =
      current.active === baseUrl ? (servers[0]?.api.baseUrl ?? current.active) : current.active;
    setState({ status: "signed_in", servers, active });
  }, []);

  /**
   * Build the connection to one server. Rotated tokens are written back to the
   * keyring as they arrive, so the stored token is always the live one — if the
   * app is killed a second later, the next launch still works. A refused
   * refresh only drops *this* server.
   */
  const connect = useCallback((baseUrl: string, tokens: Tokens): AuthedApi => {
    return new AuthedApi(baseUrl, tokens, {
      onTokens: (fresh) => {
        void saveSession({ base_url: baseUrl, refresh_token: fresh.refreshToken }).then(
          (result) => {
            if (result.kind === "unavailable" && mounted.current) {
              setKeyringNotice(result.reason);
            }
          },
        );
      },
      onSignedOut: (reason) => {
        if (!mounted.current) return;
        void dropServer(baseUrl, reason);
      },
    });
  }, [dropServer]);

  useEffect(() => {
    restoreOnce ??= restore();
    void restoreOnce.then(async (result) => {
      if (!mounted.current) return;
      setKeyringNotice(result.keyringNotice);
      setNotice(result.notice);
      if (result.entries.length === 0 || result.active === null) {
        setState({ status: "signed_out" });
        return;
      }
      const servers: ServerEntry[] = [];
      for (const entry of result.entries) {
        const api = connect(entry.baseUrl, entry.tokens);
        try {
          const user = await api.me();
          servers.push({ api, user });
        } catch (error) {
          if (!mounted.current) return;
          setNotice(error instanceof Error ? error.message : "Couldn't sign you back in.");
        }
      }
      if (!mounted.current) return;
      const first = servers[0];
      if (first === undefined) {
        setState({ status: "signed_out" });
        return;
      }
      const active =
        result.active !== null && servers.some((entry) => entry.api.baseUrl === result.active)
          ? result.active
          : first.api.baseUrl;
      setState({ status: "signed_in", servers, active });
    });
  }, [connect]);

  const signIn = useCallback(
    async (baseUrl: string, auth: AuthResponse) => {
      const tokens: Tokens = {
        accessToken: auth.access_token,
        refreshToken: auth.refresh_token,
        expiresAt: expiryOf(auth.expires_in),
      };
      const stored = await saveSession({
        base_url: baseUrl,
        refresh_token: auth.refresh_token,
      });
      if (!mounted.current) return;
      setKeyringNotice(stored.kind === "unavailable" ? stored.reason : null);
      setNotice(null);
      const api = connect(baseUrl, tokens);
      const entry: ServerEntry = { api, user: auth.user };
      const current = stateRef.current;
      if (current.status === "signed_in") {
        const without = current.servers.filter((held) => held.api.baseUrl !== baseUrl);
        setState({ status: "signed_in", servers: [...without, entry], active: baseUrl });
        return;
      }
      setState({ status: "signed_in", servers: [entry], active: baseUrl });
    },
    [connect],
  );

  const signOut = useCallback(async (baseUrl?: string) => {
    const current = stateRef.current;
    if (current.status !== "signed_in") {
      setState({ status: "signed_out" });
      setNotice(null);
      await clearSession();
      return;
    }
    const target = baseUrl ?? current.active;
    const leaving = current.servers.find((entry) => entry.api.baseUrl === target);
    const remaining = current.servers.filter((entry) => entry.api.baseUrl !== target);
    setNotice(null);
    if (remaining.length === 0) {
      setState({ status: "signed_out" });
    } else {
      const active =
        current.active === target ? (remaining[0]?.api.baseUrl ?? current.active) : current.active;
      setState({ status: "signed_in", servers: remaining, active });
    }
    await clearSession(target);
    if (leaving) {
      // Best effort: this revokes the token family server-side, but a failure
      // here must not keep someone signed in on their own screen.
      await new PublicApi(leaving.api.baseUrl)
        .logout(leaving.api.refreshToken)
        .catch(() => undefined);
    }
  }, []);

  const setActive = useCallback((baseUrl: string) => {
    const current = stateRef.current;
    if (current.status !== "signed_in") return;
    if (!current.servers.some((entry) => entry.api.baseUrl === baseUrl)) return;
    if (current.active === baseUrl) return;
    setState({ ...current, active: baseUrl });
    void setActiveSession(baseUrl);
  }, []);

  return { state, keyringNotice, notice, signIn, signOut, setActive };
}
