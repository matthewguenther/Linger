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

/// Everything the S3 backend needs, all of it from the environment.
///
/// There is no default that could work: a bucket nobody named is not a bucket,
/// so `LINGER_STORAGE=s3` without these is a startup error rather than a server
/// that boots and then cannot take a single file.
#[derive(Clone)]
pub struct S3Config {
    /// `LINGER_S3_BUCKET`.
    pub bucket: String,
    /// `LINGER_S3_REGION` — default `us-east-1`, which is also what MinIO
    /// answers to. Cloudflare R2 wants `auto`.
    pub region: String,
    /// `LINGER_S3_ENDPOINT` — the S3 API host. Unset means real AWS, and the
    /// endpoint is derived from the region.
    pub endpoint: String,
    /// `LINGER_S3_PATH_STYLE` — `bucket.host` (virtual-host) or `host/bucket`
    /// (path). Defaults to path style when an endpoint is configured, because
    /// that is what MinIO and every other self-hosted implementation expect,
    /// and to virtual-host on real AWS.
    pub path_style: bool,
    /// `LINGER_S3_ACCESS_KEY_ID`.
    pub access_key_id: String,
    /// `LINGER_S3_SECRET_ACCESS_KEY`.
    pub secret_access_key: String,
}

// The secret is a secret. Debug output of `Config` gets logged, and a key that
// only leaks into a log file is still a key that leaked.
impl std::fmt::Debug for S3Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3Config")
            .field("bucket", &self.bucket)
            .field("region", &self.region)
            .field("endpoint", &self.endpoint)
            .field("path_style", &self.path_style)
            .field("access_key_id", &self.access_key_id)
            .field("secret_access_key", &"<redacted>")
            .finish()
    }
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
    /// Set iff `storage` is [`Storage::S3`].
    pub s3: Option<S3Config>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("LINGER_BIND is not a valid socket address: {0}")]
    Bind(String),
    #[error("LINGER_STORAGE must be 'local' or 's3', got {0:?}")]
    Storage(String),
    #[error("LINGER_STORAGE=s3 needs {0} set")]
    S3Missing(&'static str),
    #[error("LINGER_S3_PATH_STYLE must be 'true' or 'false', got {0:?}")]
    S3PathStyle(String),
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

        let s3 = match storage {
            Storage::Local => None,
            Storage::S3 => Some(Self::s3_from_env()?),
        };

        Ok(Self {
            data_dir,
            bind,
            domain: std::env::var("LINGER_DOMAIN").ok(),
            storage,
            s3,
        })
    }

    fn s3_from_env() -> Result<S3Config, ConfigError> {
        fn required(name: &'static str) -> Result<String, ConfigError> {
            std::env::var(name)
                .ok()
                .filter(|value| !value.trim().is_empty())
                .ok_or(ConfigError::S3Missing(name))
        }

        let bucket = required("LINGER_S3_BUCKET")?;
        let region = std::env::var("LINGER_S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let endpoint = std::env::var("LINGER_S3_ENDPOINT")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let path_style = match std::env::var("LINGER_S3_PATH_STYLE").as_deref() {
            Ok("true") => true,
            Ok("false") => false,
            Ok(other) => return Err(ConfigError::S3PathStyle(other.to_string())),
            Err(_) => endpoint.is_some(),
        };

        Ok(S3Config {
            bucket,
            region: region.clone(),
            endpoint: endpoint.unwrap_or_else(|| format!("https://s3.{region}.amazonaws.com")),
            path_style,
            access_key_id: required("LINGER_S3_ACCESS_KEY_ID")?,
            secret_access_key: required("LINGER_S3_SECRET_ACCESS_KEY")?,
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

    /// Scratch space for uploads on their way through.
    ///
    /// Even with S3, an upload has to touch this disk once: the server cannot
    /// sniff a type or strip EXIF from bytes it has never seen. Parts are
    /// gathered here, cleaned in place, and pushed to the bucket, and nothing
    /// is left behind afterwards.
    #[must_use]
    pub fn staging_dir(&self) -> PathBuf {
        self.data_dir.join("staging")
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
