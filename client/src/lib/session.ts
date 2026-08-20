/**
 * Who is signed in, and staying signed in across restarts.
 *
 * The shape of it: the refresh token lives in the OS keyring, and on every
 * start the client trades it for a fresh access token and asks the server who
 * that is. Nothing about the account is cached locally, so a name or style
 * changed on another device is right the moment the app opens.
 *
 * AGENTS allows local state plus one gateway store. This is the local half; the
 * gateway store is `./gateway.ts`. The split is deliberate — who you are outlives
 * any one connection, and the connection is not this file's business.
 */
import { useCallback, useEffect, useRef, useState } from "react";

import type { AuthResponse } from "../generated/AuthResponse";
import type { User } from "../generated/User";
import { ApiError, AuthedApi, expiryOf, PublicApi, type Tokens } from "./api";
import { clearSession, loadSession, saveSession } from "./ipc";

export type SessionState =
  | { status: "restoring" }
  | { status: "signed_out" }
  | { status: "signed_in"; api: AuthedApi; user: User };

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
  signOut: () => Promise<void>;
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
  tokens: Tokens | null;
  baseUrl: string | null;
  keyringNotice: string | null;
  notice: string | null;
}

async function restore(): Promise<Restored> {
  const loaded = await loadSession();
  if (loaded.kind === "unavailable") {
    return { tokens: null, baseUrl: null, keyringNotice: loaded.reason, notice: null };
  }
  if (loaded.kind === "empty") {
    return { tokens: null, baseUrl: null, keyringNotice: null, notice: null };
  }

  const { base_url: baseUrl, refresh_token: refreshToken } = loaded.session;
  try {
    const fresh = await new PublicApi(baseUrl).refresh({ refresh_token: refreshToken });
    await saveSession({ base_url: baseUrl, refresh_token: fresh.refresh_token });
    return {
      tokens: {
        accessToken: fresh.access_token,
        refreshToken: fresh.refresh_token,
        expiresAt: expiryOf(fresh.expires_in),
      },
      baseUrl,
      keyringNotice: null,
      notice: null,
    };
  } catch (error) {
    // A refused token is genuinely dead, so forget it. Anything else — the
    // server is down, the laptop is on a train — leaves it in place, because
    // it will very likely work on the next launch.
    const refused =
      error instanceof ApiError &&
      (error.code === "UNAUTHENTICATED" || error.code === "FORBIDDEN");
    if (refused) await clearSession();
    const notice =
      error instanceof Error
        ? refused
          ? "Your saved sign-in is no longer valid. Please sign in again."
          : error.message
        : "Couldn't restore your last sign-in.";
    return { tokens: null, baseUrl: null, keyringNotice: null, notice };
  }
}

export function useSession(): Session {
  const [state, setState] = useState<SessionState>({ status: "restoring" });
  const [keyringNotice, setKeyringNotice] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  /**
   * Build the connection to one server. Rotated tokens are written back to the
   * keyring as they arrive, so the stored token is always the live one — if the
   * app is killed a second later, the next launch still works.
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
        void clearSession();
        setNotice(reason);
        setState({ status: "signed_out" });
      },
    });
  }, []);

  useEffect(() => {
    restoreOnce ??= restore();
    void restoreOnce.then(async (result) => {
      if (!mounted.current) return;
      setKeyringNotice(result.keyringNotice);
      setNotice(result.notice);
      if (!result.tokens || !result.baseUrl) {
        setState({ status: "signed_out" });
        return;
      }
      const api = connect(result.baseUrl, result.tokens);
      try {
        const user = await api.me();
        if (mounted.current) setState({ status: "signed_in", api, user });
      } catch (error) {
        if (!mounted.current) return;
        setNotice(error instanceof Error ? error.message : "Couldn't sign you back in.");
        setState({ status: "signed_out" });
      }
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
      setState({ status: "signed_in", api: connect(baseUrl, tokens), user: auth.user });
    },
    [connect],
  );

  const signOut = useCallback(async () => {
    const current = state;
    setState({ status: "signed_out" });
    setNotice(null);
    await clearSession();
    if (current.status === "signed_in") {
      // Best effort: this revokes the token family server-side, but a failure
      // here must not keep someone signed in on their own screen.
      await new PublicApi(current.api.baseUrl)
        .logout(current.api.refreshToken)
        .catch(() => undefined);
    }
  }, [state]);

  return { state, keyringNotice, notice, signIn, signOut };
}
