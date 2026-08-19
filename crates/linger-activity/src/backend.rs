//! Per-platform backend selection (ARCHITECTURE §6).
//!
//! Selection reads `$XDG_SESSION_TYPE` and `$XDG_CURRENT_DESKTOP` with substring
//! matching and must *always* return something usable — an unsupported platform
//! gets [`NullBackend`], which cleanly reports nothing. Never crash, never block
//! startup, never take down presence because activity detection is unavailable.
//!
//! Implementation status (real backends land in M5; see TASKS.md):
//!
//! | Platform          | Approach                                             | Status |
//! |-------------------|------------------------------------------------------|--------|
//! | Windows           | GetForegroundWindow → QueryFullProcessImageNameW     | M5     |
//! | macOS             | NSWorkspace.frontmostApplication.bundleIdentifier    | M5     |
//! | Linux / X11       | _NET_ACTIVE_WINDOW → _NET_WM_PID                     | M5     |
//! | Linux / KWin      | KWin scripting over D-Bus — **spike-verified** on    | M5     |
//! |                   | Plasma 6.6 Wayland, 2026-08-19; recipe below         |        |
//! | Linux / Hyprland  | IPC socket                                           | M5     |
//! | Linux / sway      | i3 IPC                                               | M5     |
//! | Linux / GNOME     | X11 backend only; Wayland+GNOME = presence without   | never  |
//! |                   | activity (documented limitation, no shell extension) |        |
//!
//! ## The verified KWin recipe (works on X11 and Wayland sessions alike)
//!
//! 1. Register a session D-Bus service (e.g. `org.linger.Activity`) exposing a
//!    `Report(s app_id, i pid)` method.
//! 2. Write a KWin script to a temp file:
//!    ```js
//!    function report() {
//!        const w = workspace.activeWindow;  // Plasma 6 name (activeClient on 5)
//!        if (w) callDBus("org.linger.Activity", "/Activity", "org.linger.Activity",
//!                        "Report", String(w.resourceClass), w.pid);
//!    }
//!    workspace.windowActivated.connect(report);
//!    report();
//!    ```
//! 3. Load it: `org.kde.KWin /Scripting org.kde.kwin.Scripting.loadScript(path, name)`
//!    → returns an id; then call `run()` on `/Scripting/Script<id>` (Plasma 6) or
//!    `/<id>` (Plasma 5), interface `org.kde.kwin.Script`.
//! 4. `resourceClass` is the app id (never a title); resolve `pid` via
//!    `/proc/<pid>/exe` for the executable path.
//! 5. On shutdown call `unloadScript(name)`. Also call it before loading, to clear
//!    a stale copy from a crashed previous run.
//!
//! Note the KWin backend is *event-driven*: it caches the latest report and
//! answers `foreground_process()` from the cache, rather than polling.

use crate::{ActivityBackend, BackendError, ProcessIdent};

/// The clean fallback: reports nothing, forever. Used when the platform is
/// unsupported or a real backend failed to initialize.
pub struct NullBackend;

impl ActivityBackend for NullBackend {
    fn foreground_process(&self) -> Result<Option<ProcessIdent>, BackendError> {
        Ok(None)
    }
}

/// Which backend `select_backend` chose, for the status bar and for tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Windows,
    MacOs,
    LinuxX11,
    LinuxKwin,
    LinuxHyprland,
    LinuxSway,
    /// Presence-only: activity detection unavailable on this platform.
    Null,
}

/// Decide which backend fits the current environment. Pure function of the
/// provided environment values so it is testable without a live session.
#[must_use]
pub fn classify(
    os: &str,
    session_type: Option<&str>,
    current_desktop: Option<&str>,
    hyprland_signature: Option<&str>,
    sway_sock: Option<&str>,
) -> BackendKind {
    match os {
        "windows" => BackendKind::Windows,
        "macos" => BackendKind::MacOs,
        "linux" => {
            let desktop = current_desktop.unwrap_or("").to_lowercase();
            // KWin scripting works identically under X11 and Wayland sessions,
            // so KDE always takes the KWin path.
            if desktop.contains("kde") {
                return BackendKind::LinuxKwin;
            }
            if hyprland_signature.is_some_and(|s| !s.is_empty()) {
                return BackendKind::LinuxHyprland;
            }
            if desktop.contains("sway") || sway_sock.is_some_and(|s| !s.is_empty()) {
                return BackendKind::LinuxSway;
            }
            match session_type {
                Some("x11") => BackendKind::LinuxX11,
                // Wayland on GNOME (or anything unrecognized): presence only.
                _ => BackendKind::Null,
            }
        }
        _ => BackendKind::Null,
    }
}

/// Select and construct the backend for this machine. Until the real backends
/// land (M5), every classification maps to [`NullBackend`] — presence still works,
/// activity is simply absent, which is the correct degraded behavior.
#[must_use]
pub fn select_backend() -> (BackendKind, Box<dyn ActivityBackend>) {
    let kind = classify(
        std::env::consts::OS,
        std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        std::env::var("XDG_CURRENT_DESKTOP").ok().as_deref(),
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok().as_deref(),
        std::env::var("SWAYSOCK").ok().as_deref(),
    );
    (kind, Box::new(NullBackend))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_covers_the_matrix() {
        assert_eq!(classify("windows", None, None, None, None), BackendKind::Windows);
        assert_eq!(classify("macos", None, None, None, None), BackendKind::MacOs);
        assert_eq!(
            classify("linux", Some("wayland"), Some("KDE"), None, None),
            BackendKind::LinuxKwin
        );
        assert_eq!(
            classify("linux", Some("x11"), Some("KDE"), None, None),
            BackendKind::LinuxKwin,
            "KWin scripting works on X11 sessions too"
        );
        assert_eq!(
            classify("linux", Some("wayland"), Some("Hyprland"), Some("abc123"), None),
            BackendKind::LinuxHyprland
        );
        assert_eq!(
            classify("linux", Some("wayland"), Some("sway"), None, Some("/run/sway.sock")),
            BackendKind::LinuxSway
        );
        assert_eq!(
            classify("linux", Some("x11"), Some("GNOME"), None, None),
            BackendKind::LinuxX11
        );
        assert_eq!(
            classify("linux", Some("wayland"), Some("GNOME"), None, None),
            BackendKind::Null,
            "Wayland+GNOME is presence-only, by decision (no shell extension in V1)"
        );
    }

    /// AGENTS.md smoke test: backends return cleanly (possibly None) and never
    /// panic on an unsupported or bizarre session type.
    #[test]
    fn selection_never_panics_and_null_reports_none() {
        let (_kind, backend) = select_backend();
        let result = backend.foreground_process();
        assert!(result.is_ok());

        assert_eq!(
            classify("linux", Some("mir?!"), Some("Enlightenment"), None, None),
            BackendKind::Null
        );
        assert!(NullBackend.foreground_process().unwrap().is_none());
    }
}
