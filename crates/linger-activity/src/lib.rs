//! Activity detection (ARCHITECTURE §6, SPEC §4.3).
//!
//! The privacy rules are enforced by this crate's types, not by discipline:
//!
//! - [`ProcessIdent`] has **no window-title field**. Do not add one. Not to the
//!   server, not to other clients, not to logs, not "temporarily for debugging."
//! - **Default deny**: a process that doesn't resolve against the bundled registry
//!   reports [`Activity::None`]. Unknown things report nothing at all.
//! - Browsers resolve to the browser ("Firefox"), never a site or tab.
//!
//! Backends select per-platform (see [`backend`]) and must fall back to a clean
//! `None` — never crash, never block startup.

pub mod backend;
pub mod registry;

use std::path::PathBuf;
use std::time::SystemTime;

/// What gets reported upward. Either a resolved, registry-listed app, or nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Activity {
    None,
    App { registry_id: String, since: SystemTime },
}

/// The raw identity of a foreground process, as read from the OS.
/// There is deliberately no field for a window title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessIdent {
    /// e.g. "firefox", "steam_app_730". Lowercased, `.exe` stripped, by [`normalize`].
    pub exe_name: String,
    pub exe_path: Option<PathBuf>,
    /// macOS bundle identifier, e.g. "org.mozilla.firefox".
    pub bundle_id: Option<String>,
}

/// One platform's way of answering "what app is in the foreground right now?".
///
/// Pull-based by design: the poller (3s focused / 15s unfocused, 20s debounce —
/// ARCHITECTURE §6) calls this. Event-driven platforms (KWin) keep a cache updated
/// by their event stream and answer from it.
pub trait ActivityBackend: Send + Sync {
    fn foreground_process(&self) -> Result<Option<ProcessIdent>, BackendError>;
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("platform call failed: {0}")]
    Platform(String),
}

/// Normalize a raw executable name for registry lookup: lowercase, strip a
/// trailing `.exe`. Kept as a pure function so it's trivially testable.
#[must_use]
pub fn normalize_exe_name(raw: &str) -> String {
    let lower = raw.to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

/// The full resolution pipeline (ARCHITECTURE §6): normalize → registry lookup →
/// per-app hide list → report. A miss anywhere is `Activity::None` — default deny.
#[must_use]
pub fn resolve(
    ident: &ProcessIdent,
    registry: &registry::Registry,
    hidden_ids: &[String],
    since: SystemTime,
) -> Activity {
    match registry.resolve(ident) {
        Some(app) if !hidden_ids.iter().any(|h| h == &app.id) => Activity::App {
            registry_id: app.id.clone(),
            since,
        },
        _ => Activity::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> registry::Registry {
        registry::Registry::from_json(
            r#"{ "version": 1, "apps": [
                { "id": "firefox", "label": "Firefox", "kind": "browser",
                  "match": { "exe": ["firefox", "firefox-bin"], "bundle": ["org.mozilla.firefox"] } },
                { "id": "blender", "label": "Blender", "kind": "creative",
                  "match": { "exe": ["blender"] } }
            ] }"#,
        )
        .unwrap()
    }

    fn ident(exe: &str) -> ProcessIdent {
        ProcessIdent {
            exe_name: normalize_exe_name(exe),
            exe_path: None,
            bundle_id: None,
        }
    }

    #[test]
    fn normalization_strips_exe_and_lowercases() {
        assert_eq!(normalize_exe_name("Firefox.EXE"), "firefox");
        assert_eq!(normalize_exe_name("blender"), "blender");
    }

    #[test]
    fn unknown_process_reports_nothing_at_all() {
        // Default deny is the core product rule of activity detection.
        let reg = test_registry();
        let act = resolve(&ident("definitely-not-listed"), &reg, &[], SystemTime::now());
        assert_eq!(act, Activity::None);
    }

    #[test]
    fn known_process_resolves_to_registry_id() {
        let reg = test_registry();
        let act = resolve(&ident("Firefox.exe"), &reg, &[], SystemTime::UNIX_EPOCH);
        assert_eq!(
            act,
            Activity::App { registry_id: "firefox".into(), since: SystemTime::UNIX_EPOCH }
        );
    }

    #[test]
    fn hidden_apps_report_nothing() {
        let reg = test_registry();
        let act = resolve(&ident("blender"), &reg, &["blender".into()], SystemTime::now());
        assert_eq!(act, Activity::None);
    }

    #[test]
    fn process_ident_has_no_title_field() {
        // Compile-time documentation: constructing the full struct requires exactly
        // these fields. If someone adds a title field, this stops compiling and
        // they meet the hard rule in AGENTS.md.
        let _ = ProcessIdent {
            exe_name: String::new(),
            exe_path: None,
            bundle_id: None,
        };
    }
}
