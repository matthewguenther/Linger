/**
 * The member's settings (T-411).
 *
 * Display name, password, density, whether other people's name styling is drawn
 * at all, sign out. One panel over the stream, the same way the host's controls
 * sit over it: no modal stack, and the roster stays visible so you can see your
 * name change on the card that is yours.
 *
 * Username is on this screen because people look for it next to the display
 * name, but it is not a field you can edit — PROTOCOL §2, usernames are
 * immutable. The server is the lock; this is just honest.
 */
import { type ReactNode, useEffect, useId, useRef, useState } from "react";

import type { AuthResponse } from "../generated/AuthResponse";
import type { User } from "../generated/User";
import { ApiError, PublicApi, type AuthedApi } from "../lib/api";
import DensityPicker from "../lib/DensityPicker";
import type { Density } from "../lib/density";
import { THEME_PREFS, type ThemePref } from "../lib/theme";
import {
  appVersion,
  checkForUpdate,
  installUpdate,
  updateLine,
  type UpdateCheck,
} from "../lib/updates";
import StylePicker from "./StylePicker";
import { saveDisplayName } from "../lib/gateway";
import {
  displayNameReady,
  displayNameRequest,
  MAX_DISPLAY_NAME_CHARS,
  MIN_PASSWORD_CHARS,
  passwordReady,
  passwordRequest,
} from "./settings";
import "./settings.css";

function problemText(error: unknown, fallback: string): string {
  return error instanceof ApiError ? error.message : fallback;
}

export default function SettingsPanel({
  api,
  user,
  density,
  onDensityChange,
  normalize,
  onNormalizeChange,
  theme,
  onThemeChange,
  warmth,
  onWarmthChange,
  onSignOut,
  onReauthenticated,
  onClose,
  roster,
}: {
  api: AuthedApi;
  /** `ready` is fresher; the stored session is what we have before it arrives. */
  user: User;
  density: Density;
  onDensityChange: (density: Density) => void;
  normalize: boolean;
  onNormalizeChange: (normalize: boolean) => void;
  theme: ThemePref;
  onThemeChange: (theme: ThemePref) => void;
  warmth: boolean;
  onWarmthChange: (warmth: boolean) => void;
  onSignOut: () => Promise<void>;
  onReauthenticated: (auth: AuthResponse) => Promise<void>;
  onClose: () => void;
  roster?: ReactNode;
}) {
  const panel = useRef<HTMLElement>(null);

  useEffect(() => {
    const node = panel.current;
    if (node === null) return;
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === "Escape") {
        event.stopPropagation();
        onClose();
      }
    };
    node.addEventListener("keydown", onKey);
    return () => node.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <main className="stream settings" ref={panel}>
      <header className="stream-header settings-head">
        <h2 className="panel-label">you</h2>
        <button type="button" className="settings-close meta" onClick={onClose}>
          close
        </button>
      </header>
      <div className="settings-body">
        <NameSection api={api} user={user} />
        <StylePicker
          api={api}
          user={user}
          normalized={normalize}
          dense={density !== "comfortable"}
        />
        <PasswordSection
          api={api}
          username={user.username}
          onReauthenticated={onReauthenticated}
        />
        <section className="settings-section">
          <h3 className="panel-label">density</h3>
          <p className="settings-lead">
            How the stream is laid out. Comfortable is the default. IRC is one
            line per message, no grouping.
          </p>
          <div className="settings-density">
            <DensityPicker density={density} onChange={onDensityChange} />
          </div>
        </section>
        <section className="settings-section">
          <h3 className="panel-label">theme</h3>
          <p className="settings-lead">
            Dark is the one this was designed in. <em>System</em> follows
            whatever your desktop is set to and changes with it.
          </p>
          <div className="settings-density">
            <div className="density" role="group" aria-label="theme">
              {THEME_PREFS.map((pref) => (
                <button
                  key={pref}
                  type="button"
                  className="density-option meta"
                  aria-pressed={pref === theme}
                  onClick={() => onThemeChange(pref)}
                >
                  {pref}
                </button>
              ))}
            </div>
          </div>
          <p className="settings-lead settings-warmth-lead">
            In the evening the background and the text go slightly warmer, the
            way a room does when the lamps come on. It is a small shift and most
            people never notice it on purpose.
          </p>
          <button
            type="button"
            className="settings-mini settings-toggle"
            aria-pressed={warmth}
            onClick={() => onWarmthChange(!warmth)}
          >
            {warmth ? "evening warmth on" : "evening warmth off"}
          </button>
        </section>
        <section className="settings-section">
          <h3 className="panel-label">other people's names</h3>
          <p className="settings-lead">
            Everyone picks how their own name is drawn — a face, a color or two,
            sometimes a shimmer. If you would rather read a quiet room, turn this
            on and every name, including the fonts people set for their messages,
            comes out in your default style.
          </p>
          <button
            type="button"
            className="settings-mini settings-toggle"
            aria-pressed={normalize}
            onClick={() => onNormalizeChange(!normalize)}
          >
            {normalize ? "names normalized" : "normalize everyone"}
          </button>
        </section>
        <UpdatesSection />
        <section className="settings-section">
          <h3 className="panel-label">this computer</h3>
          <p className="settings-lead">
            Signing out forgets this server on this computer. It does not delete
            your account.
          </p>
          <button
            type="button"
            className="settings-mini settings-signout"
            onClick={() => void onSignOut()}
          >
            sign out
          </button>
        </section>
      </div>
      {roster}
    </main>
  );
}

/**
 * Updates (T-701).
 *
 * Nothing here happens on its own. The check runs when this panel opens and
 * when the button is pressed; the download only ever starts because somebody
 * asked for it. A chat window that restarts itself mid-sentence is worse than
 * one that is a version behind, so installing is always the second click.
 *
 * The signature check is not a setting and is not mentioned as one — it is not
 * optional, and there is no build of this that installs an unsigned update.
 */
function UpdatesSection() {
  const [version, setVersion] = useState<string | null>(null);
  const [check, setCheck] = useState<UpdateCheck | null>(null);
  const [looking, setLooking] = useState(true);
  const [installing, setInstalling] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  const look = async (): Promise<void> => {
    setLooking(true);
    setProblem(null);
    setCheck(await checkForUpdate());
    setLooking(false);
  };

  useEffect(() => {
    let open = true;
    void (async () => {
      const [found, current] = await Promise.all([checkForUpdate(), appVersion()]);
      if (!open) return;
      setCheck(found);
      setVersion(current);
      setLooking(false);
    })();
    return () => {
      open = false;
    };
  }, []);

  const install = async (): Promise<void> => {
    setInstalling(true);
    setProblem(null);
    // On success this never comes back: the app is replaced by the new one.
    const outcome = await installUpdate();
    setInstalling(false);
    setProblem(
      outcome.kind === "failed"
        ? `Couldn't install the update: ${outcome.reason}`
        : "This copy was not built to update itself.",
    );
  };

  const ready = check?.kind === "ready";

  return (
    <section className="settings-section">
      <h3 className="panel-label">updates</h3>
      <p className="settings-lead">
        Linger checks for a new version when you open this panel. Nothing is
        downloaded until you ask for it, and every update is checked against
        this project's signing key before it is installed.
      </p>
      <p className="settings-lead settings-warmth-lead">
        {version === null ? "Running from a browser, so there is no version to update." : `You are on version ${version}.`}
      </p>
      <p className={problem === null ? "settings-ok" : "settings-problem"} aria-live="polite">
        {problem ?? updateLine(check, looking)}
      </p>
      {ready && check.notes !== null ? <p className="settings-notes">{check.notes}</p> : null}
      <div className="settings-update-actions">
        <button
          type="button"
          className="settings-mini"
          disabled={looking || installing}
          onClick={() => void look()}
        >
          check again
        </button>
        {ready ? (
          <button
            type="button"
            className="settings-mini"
            disabled={installing}
            onClick={() => void install()}
          >
            {installing ? "downloading…" : "install and restart"}
          </button>
        ) : null}
      </div>
    </section>
  );
}

function NameSection({ api, user }: { api: AuthedApi; user: User }) {
  const nameId = useId();
  const userId = useId();
  const [name, setName] = useState(user.display_name);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  const dirty = name.trim() !== user.display_name;

  useEffect(() => {
    input.current?.focus();
    input.current?.select();
  }, []);

  // Follow a name that changed somewhere else (another device, or the
  // `user.update` from our own save) — but not while this box is mid-edit.
  useEffect(() => {
    if (!dirty) setName(user.display_name);
  }, [user.display_name, dirty]);

  const submit = async (): Promise<void> => {
    if (!displayNameReady(name, user.display_name) || busy) return;
    setBusy(true);
    setProblem(null);
    setSaved(false);
    try {
      await saveDisplayName(api, displayNameRequest(name));
      setSaved(true);
    } catch (error) {
      setProblem(problemText(error, "Couldn't save your display name."));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="settings-section">
      <h3 className="panel-label">who you are</h3>
      <p className="settings-lead">
        Your display name is what people see in the roster and next to your
        messages. It can change. Your username cannot.
      </p>
      <form
        className="settings-form"
        onSubmit={(event) => {
          event.preventDefault();
          void submit();
        }}
      >
        <label className="settings-field" htmlFor={nameId}>
          <span className="panel-label">display name</span>
          <input
            id={nameId}
            ref={input}
            className="settings-input"
            value={name}
            maxLength={MAX_DISPLAY_NAME_CHARS}
            autoCapitalize="off"
            autoCorrect="off"
            spellCheck={false}
            onChange={(event) => {
              setName(event.target.value);
              setSaved(false);
            }}
          />
        </label>
        <label className="settings-field" htmlFor={userId}>
          <span className="panel-label">username</span>
          <input
            id={userId}
            className="settings-input"
            value={user.username}
            readOnly
          />
          <span className="settings-hint meta">
            People mention you with @{user.username}. This never changes.
          </span>
        </label>
        {problem === null ? null : (
          <p className="settings-problem" role="alert">
            {problem}
          </p>
        )}
        {saved ? (
          <p className="settings-ok meta" role="status">
            saved
          </p>
        ) : null}
        <div className="settings-actions">
          <button
            type="submit"
            className="settings-save"
            disabled={busy || !displayNameReady(name, user.display_name)}
          >
            {busy ? "saving…" : "save"}
          </button>
        </div>
      </form>
    </section>
  );
}

function PasswordSection({
  api,
  username,
  onReauthenticated,
}: {
  api: AuthedApi;
  username: string;
  onReauthenticated: (auth: AuthResponse) => Promise<void>;
}) {
  const currentId = useId();
  const nextId = useId();
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const submit = async (): Promise<void> => {
    if (!passwordReady(current, next) || busy) return;
    setBusy(true);
    setProblem(null);
    setSaved(false);
    try {
      await api.changePassword(passwordRequest(current, next));
    } catch (error) {
      setProblem(problemText(error, "Couldn't change your password."));
      setBusy(false);
      return;
    }

    // Changing a password revokes every refresh family this account owns
    // (the server treats it as "someone else may have had the old one").
    // Sign back in with the new password so this window is not kicked out
    // the moment the access token dies.
    try {
      const auth = await new PublicApi(api.baseUrl).login({
        username,
        password: next,
      });
      await onReauthenticated(auth);
      setCurrent("");
      setNext("");
      setSaved(true);
    } catch {
      setProblem("Password changed. Sign out and back in with the new one.");
      setCurrent("");
      setNext("");
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="settings-section">
      <h3 className="panel-label">password</h3>
      <form
        className="settings-form"
        onSubmit={(event) => {
          event.preventDefault();
          void submit();
        }}
      >
        <label className="settings-field" htmlFor={currentId}>
          <span className="panel-label">current password</span>
          <input
            id={currentId}
            className="settings-input"
            type="password"
            value={current}
            autoComplete="current-password"
            onChange={(event) => {
              setCurrent(event.target.value);
              setSaved(false);
            }}
          />
        </label>
        <label className="settings-field" htmlFor={nextId}>
          <span className="panel-label">new password</span>
          <input
            id={nextId}
            className="settings-input"
            type="password"
            value={next}
            autoComplete="new-password"
            onChange={(event) => {
              setNext(event.target.value);
              setSaved(false);
            }}
          />
          <span className="settings-hint meta">
            At least {MIN_PASSWORD_CHARS} characters. No silly rules about
            symbols.
          </span>
        </label>
        {problem === null ? null : (
          <p className="settings-problem" role="alert">
            {problem}
          </p>
        )}
        {saved ? (
          <p className="settings-ok meta" role="status">
            password changed
          </p>
        ) : null}
        <div className="settings-actions">
          <button
            type="submit"
            className="settings-mini"
            disabled={busy || !passwordReady(current, next)}
          >
            {busy ? "saving…" : "change password"}
          </button>
        </div>
      </form>
    </section>
  );
}
