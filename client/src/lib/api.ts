/**
 * The typed HTTP client. Everything the frontend knows about the server goes
 * through here.
 *
 * Two rules from AGENTS shape this file. Every request and response type is
 * imported from `src/generated/` (ts-rs output from `linger-core`) — nothing on
 * the wire is described twice. And there is no `any` and no `as` cast across the
 * boundary: `send()` is the single place where an untyped response body becomes
 * a typed one, and it is one annotated assignment, not a cast.
 */
import type { AuthResponse } from "../generated/AuthResponse";
import type { ErrorBody } from "../generated/ErrorBody";
import type { ErrorCode } from "../generated/ErrorCode";
import type { InvitePreview } from "../generated/InvitePreview";
import type { LoginRequest } from "../generated/LoginRequest";
import type { RefreshRequest } from "../generated/RefreshRequest";
import type { RefreshResponse } from "../generated/RefreshResponse";
import type { RegisterRequest } from "../generated/RegisterRequest";
import type { ServerInfo } from "../generated/ServerInfo";
import type { SetupPreview } from "../generated/SetupPreview";
import type { SetupRequest } from "../generated/SetupRequest";
import type { User } from "../generated/User";

/** Everything REST lives under this prefix (PROTOCOL §1). */
const API_PREFIX = "/api/v1";

/**
 * A refusal from the server, carrying the PROTOCOL §1 envelope. `message` is
 * written to be shown to a person, so screens display it directly.
 */
export class ApiError extends Error {
  readonly code: ErrorCode;
  readonly status: number;
  readonly retryAfterMs: number | null;

  constructor(status: number, body: ErrorBody) {
    super(body.message);
    this.name = "ApiError";
    this.code = body.code;
    this.status = status;
    this.retryAfterMs = body.retry_after_ms;
  }
}

/** The request never got an answer we could read: no network, wrong address, a
 *  proxy returning HTML. Distinct from `ApiError`, which is the server talking. */
export class TransportError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "TransportError";
  }
}

/**
 * Every error code the client recognises. This list exists so a code coming off
 * the network can be *checked* rather than assumed, which is what keeps this
 * file free of casts. The `satisfies` clause makes TypeScript fail the build if
 * this ever falls out of step with the generated `ErrorCode` union — verified by
 * deleting a key and watching `pnpm check` refuse it.
 */
const ERROR_CODES = {
  UNAUTHENTICATED: "UNAUTHENTICATED",
  FORBIDDEN: "FORBIDDEN",
  NOT_FOUND: "NOT_FOUND",
  RATE_LIMITED: "RATE_LIMITED",
  VALIDATION_FAILED: "VALIDATION_FAILED",
  INVITE_INVALID: "INVITE_INVALID",
  INVITE_EXPIRED: "INVITE_EXPIRED",
  QUOTA_EXCEEDED: "QUOTA_EXCEEDED",
  FILE_TOO_LARGE: "FILE_TOO_LARGE",
  UNSUPPORTED_MEDIA: "UNSUPPORTED_MEDIA",
  CONFLICT: "CONFLICT",
  INTERNAL: "INTERNAL",
} as const satisfies Record<ErrorCode, ErrorCode>;

function toErrorCode(value: unknown): ErrorCode | null {
  for (const code of Object.values(ERROR_CODES)) if (code === value) return code;
  return null;
}

/** Read a property off a value of unknown shape without asserting anything. */
function field(value: unknown, key: string): unknown {
  if (typeof value !== "object" || value === null) return undefined;
  const found: unknown = Reflect.get(value, key);
  return found;
}

/** Runtime shape check, because a 502 from a reverse proxy is not JSON at all. */
function asErrorBody(value: unknown): ErrorBody | null {
  const error = field(value, "error");
  const message = field(error, "message");
  if (typeof message !== "string") return null;
  // Codes are additive within v1, so a newer server may send one this build has
  // never heard of. Fall back to INTERNAL and keep the server's own wording —
  // the message is what the person reads.
  const code = toErrorCode(field(error, "code")) ?? "INTERNAL";
  const retry = field(error, "retry_after_ms");
  return {
    code,
    message,
    retry_after_ms: typeof retry === "number" ? retry : null,
  };
}

export interface RequestOptions {
  body?: unknown;
  accessToken?: string;
  signal?: AbortSignal;
}

async function send(
  baseUrl: string,
  method: string,
  path: string,
  options: RequestOptions,
): Promise<Response> {
  const headers: Record<string, string> = { Accept: "application/json" };
  if (options.body !== undefined) headers["Content-Type"] = "application/json";
  if (options.accessToken) headers.Authorization = `Bearer ${options.accessToken}`;

  let response: Response;
  try {
    response = await fetch(`${baseUrl}${API_PREFIX}${path}`, {
      method,
      headers,
      body: options.body === undefined ? undefined : JSON.stringify(options.body),
      signal: options.signal,
    });
  } catch {
    throw new TransportError(
      `Couldn't reach ${baseUrl}. Check the address and that the server is running.`,
    );
  }

  if (response.ok) return response;

  const parsed: unknown = await response.json().catch(() => null);
  const body = asErrorBody(parsed);
  if (body) throw new ApiError(response.status, body);
  throw new TransportError(
    `${baseUrl} answered with ${response.status}, but not in Linger's format. Is that address a Linger server?`,
  );
}

async function requestJson<T>(
  baseUrl: string,
  method: string,
  path: string,
  options: RequestOptions = {},
): Promise<T> {
  const response = await send(baseUrl, method, path, options);
  // The one place the wire becomes typed. `Response.json()` is untyped, and `T`
  // is always a ts-rs type generated from the server's own definition, so a
  // mismatch here is a server bug rather than a guess on our side.
  const parsed: T = await response.json();
  return parsed;
}

async function requestVoid(
  baseUrl: string,
  method: string,
  path: string,
  options: RequestOptions = {},
): Promise<void> {
  await send(baseUrl, method, path, options);
}

/**
 * Calls that need no account: everything on the way *in* to a server.
 */
export class PublicApi {
  constructor(readonly baseUrl: string) {}

  /** Cheap "is there a Linger server at this address" probe. */
  async health(signal?: AbortSignal): Promise<void> {
    await requestVoid(this.baseUrl, "GET", "/health", { signal });
  }

  invitePreview(code: string): Promise<InvitePreview> {
    return requestJson(this.baseUrl, "GET", `/auth/invite/${encodeURIComponent(code)}`);
  }

  setupPreview(token: string): Promise<SetupPreview> {
    return requestJson(this.baseUrl, "GET", `/setup/${encodeURIComponent(token)}`);
  }

  setup(request: SetupRequest): Promise<AuthResponse> {
    return requestJson(this.baseUrl, "POST", "/setup", { body: request });
  }

  login(request: LoginRequest): Promise<AuthResponse> {
    return requestJson(this.baseUrl, "POST", "/auth/login", { body: request });
  }

  register(request: RegisterRequest): Promise<AuthResponse> {
    return requestJson(this.baseUrl, "POST", "/auth/register", { body: request });
  }

  refresh(request: RefreshRequest): Promise<RefreshResponse> {
    return requestJson(this.baseUrl, "POST", "/auth/refresh", { body: request });
  }

  logout(refreshToken: string): Promise<void> {
    const body: RefreshRequest = { refresh_token: refreshToken };
    return requestVoid(this.baseUrl, "POST", "/auth/logout", { body });
  }
}

export interface Tokens {
  accessToken: string;
  refreshToken: string;
  /** When the access token dies, in Unix milliseconds. */
  expiresAt: number;
}

/** Turn the server's `expires_in` (seconds) into a moment we can compare to. */
export function expiryOf(expiresIn: number): number {
  return Date.now() + expiresIn * 1000;
}

/**
 * A signed-in connection to one server.
 *
 * Access tokens last 15 minutes, so expiry during normal use is routine rather
 * than exceptional. When a call comes back `UNAUTHENTICATED` this trades the
 * refresh token for a new pair and runs the call again, once. Refresh tokens
 * rotate and reusing a spent one revokes the whole family (PROTOCOL §2), so two
 * calls refreshing at the same time would log the user out — hence the single
 * in-flight refresh that every caller waits on.
 */
export class AuthedApi {
  readonly baseUrl: string;
  #tokens: Tokens;
  #refreshing: Promise<void> | null = null;
  #onTokens: (tokens: Tokens) => void;
  #onSignedOut: (reason: string) => void;

  constructor(
    baseUrl: string,
    tokens: Tokens,
    handlers: {
      onTokens: (tokens: Tokens) => void;
      onSignedOut: (reason: string) => void;
    },
  ) {
    this.baseUrl = baseUrl;
    this.#tokens = tokens;
    this.#onTokens = handlers.onTokens;
    this.#onSignedOut = handlers.onSignedOut;
  }

  get refreshToken(): string {
    return this.#tokens.refreshToken;
  }

  /**
   * An access token good enough to hand to the gateway, refreshed first if it
   * is about to expire. The gateway connection lives in the Tauri core and has
   * no refresh token of its own — on purpose, since two parties spending a
   * rotating refresh token revokes the family (PROTOCOL §2). This is the one
   * door it comes through.
   *
   * `force` is for the case where the server refused a token that had not
   * expired yet, which happens when a server comes back with new signing keys.
   */
  async accessToken(force = false): Promise<{ token: string; expiresAt: number }> {
    const soon = Date.now() + 60_000;
    if (force || this.#tokens.expiresAt <= soon) await this.#refresh();
    return { token: this.#tokens.accessToken, expiresAt: this.#tokens.expiresAt };
  }

  get<T>(path: string, signal?: AbortSignal): Promise<T> {
    return this.#withAuth((accessToken) =>
      requestJson<T>(this.baseUrl, "GET", path, { accessToken, signal }),
    );
  }

  post<T>(path: string, body?: unknown): Promise<T> {
    return this.#withAuth((accessToken) =>
      requestJson<T>(this.baseUrl, "POST", path, { accessToken, body }),
    );
  }

  patch<T>(path: string, body: unknown): Promise<T> {
    return this.#withAuth((accessToken) =>
      requestJson<T>(this.baseUrl, "PATCH", path, { accessToken, body }),
    );
  }

  delete(path: string): Promise<void> {
    return this.#withAuth((accessToken) =>
      requestVoid(this.baseUrl, "DELETE", path, { accessToken }),
    );
  }

  me(signal?: AbortSignal): Promise<User> {
    return this.get<User>("/me", signal);
  }

  serverInfo(signal?: AbortSignal): Promise<ServerInfo> {
    return this.get<ServerInfo>("/server", signal);
  }

  async #withAuth<T>(call: (accessToken: string) => Promise<T>): Promise<T> {
    try {
      return await call(this.#tokens.accessToken);
    } catch (error) {
      if (!(error instanceof ApiError) || error.code !== "UNAUTHENTICATED") throw error;
      await this.#refresh();
      return await call(this.#tokens.accessToken);
    }
  }

  #refresh(): Promise<void> {
    if (this.#refreshing) return this.#refreshing;
    const attempt = (async () => {
      const previous = this.#tokens.refreshToken;
      try {
        const fresh = await new PublicApi(this.baseUrl).refresh({
          refresh_token: previous,
        });
        this.#tokens = {
          accessToken: fresh.access_token,
          refreshToken: fresh.refresh_token,
          expiresAt: expiryOf(fresh.expires_in),
        };
        this.#onTokens(this.#tokens);
      } catch (error) {
        // A refused refresh is the end of this sign-in: the token expired, or
        // it was already spent and the family got revoked. Either way the only
        // way back is signing in again. A network failure is not that, so it
        // propagates without ending the session.
        if (error instanceof ApiError) {
          this.#onSignedOut("Your sign-in expired. Please sign in again.");
        }
        throw error;
      } finally {
        this.#refreshing = null;
      }
    })();
    this.#refreshing = attempt;
    return attempt;
  }
}
