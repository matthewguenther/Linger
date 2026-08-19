//! The bundled app registry (`registry/apps.json`) and its lookup.
//!
//! The registry is the *allowlist*: an app appears in a roster only if it resolves
//! here. Users can add unknown apps to a local override file; those entries never
//! sync to the server.

use serde::Deserialize;

use crate::ProcessIdent;

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("registry parse failed: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("unsupported registry version {0}")]
    Version(u32),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryApp {
    pub id: String,
    pub label: String,
    /// game | browser | creative | editor | media | dev | chat | other.
    /// Drives roster iconography; never validated as a closed set so the
    /// registry file can grow kinds without a code change.
    pub kind: String,
    #[serde(rename = "match")]
    pub matcher: MatchSpec,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MatchSpec {
    /// Normalized executable names (lowercase, no `.exe`).
    #[serde(default)]
    pub exe: Vec<String>,
    /// macOS bundle identifiers.
    #[serde(default)]
    pub bundle: Vec<String>,
    /// Steam app id; matched against `steam_app_<id>` exe names and Steam
    /// launch paths (resolution implemented with the Steam matcher in M5).
    #[serde(default)]
    pub steam_appid: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct RegistryFile {
    version: u32,
    apps: Vec<RegistryApp>,
}

#[derive(Debug, Clone)]
pub struct Registry {
    apps: Vec<RegistryApp>,
}

impl Registry {
    /// Parse a registry JSON document (the bundled file or a local override).
    pub fn from_json(json: &str) -> Result<Self, RegistryError> {
        let file: RegistryFile = serde_json::from_str(json)?;
        if file.version != 1 {
            return Err(RegistryError::Version(file.version));
        }
        Ok(Self { apps: file.apps })
    }

    /// Resolve a process identity to a registry entry. `None` means default deny.
    #[must_use]
    pub fn resolve(&self, ident: &ProcessIdent) -> Option<&RegistryApp> {
        self.apps.iter().find(|app| {
            let m = &app.matcher;
            let exe_hit = m.exe.iter().any(|e| e == &ident.exe_name);
            let bundle_hit = ident
                .bundle_id
                .as_ref()
                .is_some_and(|b| m.bundle.iter().any(|mb| mb == b));
            let steam_hit = m
                .steam_appid
                .is_some_and(|id| ident.exe_name == format!("steam_app_{id}"));
            exe_hit || bundle_hit || steam_hit
        })
    }

    /// Look up an entry by its stable id. Used server-side to resolve a
    /// client-reported registry id into label + kind for `PresenceEntry` —
    /// an id the server doesn't know resolves to no activity (default deny
    /// holds on both ends).
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&RegistryApp> {
        self.apps.iter().find(|a| a.id == id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.apps.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.apps.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bundled registry must always parse — this is the guard against a
    /// malformed edit to registry/apps.json landing on main.
    #[test]
    fn bundled_registry_parses_and_has_entries() {
        let json = include_str!("../../../registry/apps.json");
        let reg = Registry::from_json(json).expect("registry/apps.json must parse");
        assert!(!reg.is_empty());

        // Every id must be unique — duplicate ids would make resolution ambiguous.
        let mut ids: Vec<&str> = reg.apps.iter().map(|a| a.id.as_str()).collect();
        let before = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(before, ids.len(), "duplicate app ids in registry/apps.json");
    }

    #[test]
    fn steam_appid_matching() {
        let reg = Registry::from_json(
            r#"{ "version": 1, "apps": [
                { "id": "cs2", "label": "Counter-Strike 2", "kind": "game",
                  "match": { "steam_appid": 730 } }
            ] }"#,
        )
        .unwrap();
        let ident = ProcessIdent {
            exe_name: "steam_app_730".into(),
            exe_path: None,
            bundle_id: None,
        };
        assert_eq!(reg.resolve(&ident).unwrap().id, "cs2");
    }

    #[test]
    fn future_version_is_rejected_not_misread() {
        let err = Registry::from_json(r#"{ "version": 2, "apps": [] }"#).unwrap_err();
        assert!(matches!(err, RegistryError::Version(2)));
    }
}
