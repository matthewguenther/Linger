//! Server configuration. Environment variables only — a server is configured by
//! its compose file, not a config-file format to document and version.

use std::net::SocketAddr;
use std::path::PathBuf;

/// Where uploaded objects live (ARCHITECTURE §8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Storage {
    /// Filesystem under the data dir. Default; correct for a home server.
    Local,
    /// Any S3-compatible endpoint (R2 recommended for cloud: zero egress).
    S3,
}

#[derive(Debug, Clone)]
pub struct Config {
    /// `LINGER_DATA_DIR` — holds `linger.db` and `objects/`. Default `./data`.
    pub data_dir: PathBuf,
    /// `LINGER_BIND` — default `0.0.0.0:8420` (the reverse proxy terminates TLS).
    pub bind: SocketAddr,
    /// `LINGER_DOMAIN` — public domain of this server; used in absolute URLs.
    pub domain: Option<String>,
    /// `LINGER_STORAGE` — `local` (default) or `s3`.
    pub storage: Storage,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("LINGER_BIND is not a valid socket address: {0}")]
    Bind(String),
    #[error("LINGER_STORAGE must be 'local' or 's3', got {0:?}")]
    Storage(String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let data_dir = std::env::var("LINGER_DATA_DIR")
            .map_or_else(|_| PathBuf::from("./data"), PathBuf::from);

        let bind_raw = std::env::var("LINGER_BIND").unwrap_or_else(|_| "0.0.0.0:8420".to_string());
        let bind = bind_raw
            .parse()
            .map_err(|_| ConfigError::Bind(bind_raw.clone()))?;

        let storage = match std::env::var("LINGER_STORAGE").as_deref() {
            Err(_) | Ok("local") => Storage::Local,
            Ok("s3") => Storage::S3,
            Ok(other) => return Err(ConfigError::Storage(other.to_string())),
        };

        Ok(Self {
            data_dir,
            bind,
            domain: std::env::var("LINGER_DOMAIN").ok(),
            storage,
        })
    }

    /// Path of the one SQLite file that (with `objects/`) is the entire server.
    #[must_use]
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("linger.db")
    }

    /// Root of locally stored uploads.
    #[must_use]
    pub fn objects_dir(&self) -> PathBuf {
        self.data_dir.join("objects")
    }

    /// Where this server is reachable from outside, for URLs it hands the
    /// client (upload slots, attachment links).
    ///
    /// A configured `LINGER_DOMAIN` means the documented deployment has Caddy
    /// terminating TLS in front (ARCHITECTURE §9), so it is `https`. Without
    /// one we are a bare bind address with no name and no certificate, and the
    /// only honest answer is a root-relative URL the client resolves against
    /// whatever address it reached us on.
    #[must_use]
    pub fn public_origin(&self) -> String {
        match &self.domain {
            Some(domain) => format!("https://{domain}"),
            None => String::new(),
        }
    }

    /// The URL an uploaded object is served from.
    ///
    /// **T-503 moves this to a separate origin** (`cdn.<domain>`), which is the
    /// real defence against a hostile upload touching an app session. Until it
    /// lands, objects come off the app host under `/objects`, with the
    /// download-forcing headers already in place (ARCHITECTURE §7).
    #[must_use]
    pub fn object_url(&self, key: &str) -> String {
        format!("{}/objects/{key}", self.public_origin())
    }
}
