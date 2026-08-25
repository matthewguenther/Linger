/**
 * Who is signed in, on which servers, and staying signed in across restarts.
 *
 * The shape of it: every server's refresh token lives in the OS keyring, and on
 * every start the client trades each one for a fresh access token and asks that
 * server who we are there. Nothing about an account is cached locally, so a name
 * or style changed on another device is right the moment the app opens.
 *
 * Since T-412 this is a list. Each server is its own sign-in with its own
 * account, its own tokens and its own keyring entry, and they do not touch each
 * other: signing out of one, or having one refuse our token, leaves the rest
 * exactly where they were.
 *
 * AGENTS allows local state plus one gateway store. This is the local half; the
 * gateway store is `./gateway.ts`. The split is deliberate — who you are outlives
 * any one connection, and the connection is not this file's business.
 */
import { useCallback, useEffect, useRef, useState } from "react";

import type { AuthResponse } from "../generated/AuthResponse";
import type { User } from "../generated/User";
import { ApiError, AuthedApi, expiryOf, PublicApi, type Tokens } from "./api";
import { forgetSession, loadSessions, saveSession } from "./ipc";
import { hostOf } from "./link";

/** One signed-in server. */
export interface ServerSession {
  /** Origin of the server. The key everything else is filed under. */
  baseUrl: string;
  api: AuthedApi;
  user: User;
}

export type SessionsState =
  | { status: "restoring" }
  /** Every server we are signed into, oldest first. Empty means signed out. */
  | { status: "ready"; servers: ServerSession[] };

export interface Sessions {
  state: SessionsState;
  /**
   * Set when this computer can't remember a sign-in — no wallet, a locked one,
   * or the UI running in a plain browser. Screens show it so the person finds
   * out before they're surprised by it, not after.
   */
  keyringNotice: string | null;
  /** Why a sign-in ended, when it ended on its own. */
  notice: string | null;
  /** Sign into a server, or sign back into one already in the list. */
  addServer: (baseUrl: string, auth: AuthResponse) => Promise<void>;
  /** Sign out of one server. The others are untouched. */
  signOut: (baseUrl: string) => Promise<void>;
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
  servers: { baseUrl: string; tokens: Tokens }[];
  keyringNotice: string | null;
  notice: string | null;
}

/** Trade one stored refresh token for a live one. `null` means this server is
 *  not coming back this launch, and `notice` says why. */
async function restoreOne(
  baseUrl: string,
  refreshToken: string,
): Promise<{ tokens: Tokens | null; notice: string | null }> {
  try {
    const fresh = await new PublicApi(baseUrl).refresh({ refresh_token: refreshToken });
    await saveSession({ base_url: baseUrl, refresh_token: fresh.refresh_token });
    return {
      tokens: {
        accessToken: fresh.access_token,
        refreshToken: fresh.refresh_token,
        expiresAt: expiryOf(fresh.expires_in),
      },
      notice: null,
    };
  } catch (error) {
    // A refused token is genuinely dead, so forget it. Anything else — the
    // server is down, the laptop is on a train — leaves it in place, because
    // it will very likely work on the next launch.
    const refused =
      error instanceof ApiError &&
      (error.code === "UNAUTHENTICATED" || error.code === "FORBIDDEN");
    if (refused) await forgetSession(baseUrl);
    const why =
      error instanceof Error
        ? refused
          ? "Your saved sign-in is no longer valid. Please sign in again."
          : error.message
        : "Couldn't restore your last sign-in.";
    return { tokens: null, notice: `${hostOf(baseUrl)}: ${why}` };
  }
}

async function restore(): Promise<Restored> {
  const loaded = await loadSessions();
  if (loaded.kind === "unavailable") {
    return { servers: [], keyringNotice: loaded.reason, notice: null };
  }
  if (loaded.kind === "empty") {
    return { servers: [], keyringNotice: null, notice: null };
  }

  // All at once. One server being slow or down should not hold up the ones
  // that are fine, and refresh tokens rotate per server so there is nothing
  // shared to serialize on.
  const results = await Promise.all(
    loaded.sessions.map(async (stored) => ({
      baseUrl: stored.base_url,
      ...(await restoreOne(stored.base_url, stored.refresh_token)),
    })),
  );

  const servers = results.flatMap((result) =>
    result.tokens === null ? [] : [{ baseUrl: result.baseUrl, tokens: result.tokens }],
  );
  const notices = results.map((result) => result.notice).filter((line) => line !== null);
  return {
    servers,
    keyringNotice: null,
    notice: notices.length === 0 ? null : notices.join(" "),
  };
}

export function useSessions(): Sessions {
  const [state, setState] = useState<SessionsState>({ status: "restoring" });
  const [keyringNotice, setKeyringNotice] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  /** Drop one server from the list, leaving the rest alone. */
  const remove = useCallback((baseUrl: string) => {
    setState((held) =>
      held.status === "ready"
        ? { status: "ready", servers: held.servers.filter((s) => s.baseUrl !== baseUrl) }
        : held,
    );
  }, []);

  /**
   * Build the connection to one server. Rotated tokens are written back to the
   * keyring as they arrive, so the stored token is always the live one — if the
   * app is killed a second later, the next launch still works.
   */
  const connect = useCallback(
    (baseUrl: string, tokens: Tokens): AuthedApi => {
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
          // Only this server. A token going stale on one machine's account
          // says nothing about the others.
          void forgetSession(baseUrl);
          setNotice(`${hostOf(baseUrl)}: ${reason}`);
          remove(baseUrl);
        },
      });
    },
    [remove],
  );

  useEffect(() => {
    restoreOnce ??= restore();
    void restoreOnce.then(async (result) => {
      if (!mounted.current) return;
      setKeyringNotice(result.keyringNotice);
      setNotice(result.notice);

      const opened: (
        | { ok: true; session: ServerSession }
        | { ok: false; why: string }
      )[] = await Promise.all(
        result.servers.map(async (server) => {
          const api = connect(server.baseUrl, server.tokens);
          try {
            const session: ServerSession = {
              baseUrl: server.baseUrl,
              api,
              user: await api.me(),
            };
            return { ok: true as const, session };
          } catch (error) {
            const why = error instanceof Error ? error.message : "Couldn't sign you back in.";
            return { ok: false as const, why: `${hostOf(server.baseUrl)}: ${why}` };
          }
        }),
      );
      if (!mounted.current) return;

      const servers: ServerSession[] = [];
      const failures: string[] = [];
      for (const opening of opened) {
        if (opening.ok) servers.push(opening.session);
        else failures.push(opening.why);
      }
      if (failures.length > 0) {
        setNotice((held) => [held, ...failures].filter((line) => line !== null).join(" "));
      }
      setState({ status: "ready", servers });
    });
  }, [connect]);

  const addServer = useCallback(
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
      const fresh: ServerSession = { baseUrl, api: connect(baseUrl, tokens), user: auth.user };
      setState((held) => {
        const servers = held.status === "ready" ? held.servers : [];
        const at = servers.findIndex((server) => server.baseUrl === baseUrl);
        // Signing back into a server you already have keeps its place in the
        // rail rather than sending it to the bottom of the list.
        if (at < 0) return { status: "ready", servers: [...servers, fresh] };
        const next = [...servers];
        next[at] = fresh;
        return { status: "ready", servers: next };
      });
    },
    [connect],
  );

  const signOut = useCallback(
    async (baseUrl: string) => {
      const held = state.status === "ready" ? state.servers : [];
      const leaving = held.find((server) => server.baseUrl === baseUrl);
      remove(baseUrl);
      setNotice(null);
      await forgetSession(baseUrl);
      if (leaving) {
        // Best effort: this revokes the token family server-side, but a failure
        // here must not keep someone signed in on their own screen.
        await new PublicApi(baseUrl).logout(leaving.api.refreshToken).catch(() => undefined);
      }
    },
    [remove, state],
  );

  return { state, keyringNotice, notice, addServer, signOut };
}
