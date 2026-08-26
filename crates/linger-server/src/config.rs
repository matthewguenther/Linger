//! Server configuration. Environment variables only — a server is configured by
//! its compose file, not a config-file format to document and version.

use std::net::SocketAddr;
use std::path::PathBuf;

use linger_core::limits::{DEFAULT_FILE_EXPIRY_DAYS, DEFAULT_POOL_BYTES, MAX_FILE_BYTES};

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
    /// `LINGER_MEDIA_DOMAIN` — the origin uploaded files are served from.
    /// Defaults to `cdn.<LINGER_DOMAIN>`; `None` only when this server has no
    /// domain at all (ARCHITECTURE §7).
    pub media_domain: Option<String>,
    /// `LINGER_STORAGE` — `local` (default) or `s3`.
    pub storage: Storage,
    /// Set iff `storage` is [`Storage::S3`].
    pub s3: Option<S3Config>,
    /// `LINGER_POOL_BYTES` — the storage ceiling for the whole server, default
    /// 50 GB (SPEC §4.10). Accepts a plain byte count or a size like `250GB`.
    pub pool_bytes: u64,
    /// `LINGER_FILE_EXPIRY_DAYS` — how long a file stands before the sweeper
    /// takes it, default 365 (SPEC §4.10). `off` (or `0`) means never, and a
    /// starred or pinned file never expires whatever this says.
    pub file_expiry_days: Option<u32>,
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
    #[error("LINGER_MEDIA_DOMAIN is a bare hostname, not a URL: got {0:?}")]
    MediaDomainNotAHost(String),
    #[error(
        "LINGER_POOL_BYTES is {0:?}. It wants a size: a plain number of bytes, or one \
         with a unit on it like 250GB, 500MB or 2TB."
    )]
    PoolBytes(String),
    #[error(
        "LINGER_FILE_EXPIRY_DAYS is {0:?}. It wants a whole number of days, or `off` \
         to keep every file forever."
    )]
    FileExpiryDays(String),
    #[error(
        "LINGER_POOL_BYTES is {0:?}, which is smaller than the 500 MB one file may be. \
         A server that cannot hold a single upload refuses every one of them."
    )]
    PoolTooSmall(String),
    #[error(
        "LINGER_MEDIA_DOMAIN is {0:?}, which is also LINGER_DOMAIN. Uploaded files are \
         served from their own host so a hostile file cannot touch the app (ARCHITECTURE \
         §7); pointing both names at one origin gives that up. Use cdn.{0} — the default \
         if you leave LINGER_MEDIA_DOMAIN unset."
    )]
    MediaDomainIsAppDomain(String),
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

        let domain = std::env::var("LINGER_DOMAIN").ok();
        let media_domain =
            media_domain(domain.as_deref(), std::env::var("LINGER_MEDIA_DOMAIN").ok())?;

        Ok(Self {
            data_dir,
            bind,
            domain,
            media_domain,
            storage,
            s3,
            pool_bytes: pool_bytes(std::env::var("LINGER_POOL_BYTES").ok())?,
            file_expiry_days: file_expiry_days(std::env::var("LINGER_FILE_EXPIRY_DAYS").ok())?,
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

    /// Where uploaded files are served from — a different host from the app
    /// (ARCHITECTURE §7 "user content is hostile").
    ///
    /// An upload is somebody else's bytes. Served from the app's own origin it
    /// would be same-origin with the app; served from `cdn.<domain>` it is a
    /// stranger, and a stranger is what it is. `/objects` answers on this host
    /// and nowhere else, and the rest of the server answers everywhere else —
    /// see `routes::media_origin_gate`.
    ///
    /// A server with no `LINGER_DOMAIN` has no second name to use, so it falls
    /// back to the one origin it has. That is honest for a box on a LAN and it
    /// is what every test server runs as; the split needs DNS.
    #[must_use]
    pub fn media_origin(&self) -> String {
        match &self.media_domain {
            Some(domain) => format!("https://{domain}"),
            None => self.public_origin(),
        }
    }

    /// The hostname `/objects` is served on, when it is a host of its own.
    #[must_use]
    pub fn media_host(&self) -> Option<&str> {
        self.media_domain.as_deref()
    }

    /// The URL an uploaded object is served from.
    #[must_use]
    pub fn object_url(&self, key: &str) -> String {
        format!("{}/objects/{key}", self.media_origin())
    }
}

/// Read `LINGER_POOL_BYTES`: a plain byte count, or a number with a unit on it.
///
/// The unit is the concession. The default written out is `53687091200`, and a
/// host who wants a quarter of that has to be able to type `250GB` and be
/// right — asking somebody to multiply by 1024 three times to change one
/// number in a compose file is how the number ends up wrong.
///
/// Kept a free function so it can be tested without touching the process
/// environment, which every other test in the workspace is also using.
fn pool_bytes(configured: Option<String>) -> Result<u64, ConfigError> {
    let Some(raw) = configured.filter(|value| !value.trim().is_empty()) else {
        return Ok(DEFAULT_POOL_BYTES);
    };
    let text = raw.trim();
    let upper = text.to_ascii_uppercase();
    // Longest suffix first: `GB` has to win over `B`.
    let (digits, scale) = [
        ("TB", 1u64 << 40),
        ("GB", 1 << 30),
        ("MB", 1 << 20),
        ("KB", 1 << 10),
    ]
    .into_iter()
    .find_map(|(suffix, scale)| upper.strip_suffix(suffix).map(|rest| (rest, scale)))
    .or_else(|| upper.strip_suffix('B').map(|rest| (rest, 1)))
    .unwrap_or((upper.as_str(), 1));

    let count: u64 = digits
        .trim()
        .parse()
        .map_err(|_| ConfigError::PoolBytes(text.to_string()))?;
    let bytes = count
        .checked_mul(scale)
        .ok_or_else(|| ConfigError::PoolBytes(text.to_string()))?;
    // A pool that cannot hold one file is a server that refuses every upload
    // with "storage is full", which reads as a broken server rather than a
    // misconfigured one.
    if bytes < MAX_FILE_BYTES {
        return Err(ConfigError::PoolTooSmall(text.to_string()));
    }
    Ok(bytes)
}

/// Read `LINGER_FILE_EXPIRY_DAYS`: a whole number of days, or off.
///
/// `off` and `0` are the same answer and both are spelled out, because a host
/// turning expiry off should not have to guess which one this build wants.
fn file_expiry_days(configured: Option<String>) -> Result<Option<u32>, ConfigError> {
    let Some(raw) = configured.filter(|value| !value.trim().is_empty()) else {
        return Ok(Some(DEFAULT_FILE_EXPIRY_DAYS));
    };
    let text = raw.trim();
    if text.eq_ignore_ascii_case("off") || text.eq_ignore_ascii_case("never") {
        return Ok(None);
    }
    match text.parse::<u32>() {
        Ok(0) => Ok(None),
        Ok(days) => Ok(Some(days)),
        Err(_) => Err(ConfigError::FileExpiryDays(text.to_string())),
    }
}

/// Work out the media origin's hostname: what was asked for, or `cdn.` in
/// front of the app's domain.
///
/// Kept a free function so it can be tested without touching the process
/// environment, which every other test in the workspace is also using.
fn media_domain(
    domain: Option<&str>,
    configured: Option<String>,
) -> Result<Option<String>, ConfigError> {
    let app = domain.map(|d| d.trim().trim_end_matches('/').to_ascii_lowercase());

    let Some(raw) = configured.filter(|value| !value.trim().is_empty()) else {
        // The default is a subdomain of the app's, because one extra DNS record
        // pointing at the same machine is the whole setup cost.
        return Ok(app.map(|domain| format!("cdn.{domain}")));
    };

    let media = raw.trim().trim_end_matches('/').to_ascii_lowercase();
    // A hostname, not a URL: it is compared against the `Host` header of every
    // request, and `https://cdn.example` never equals a `Host` header.
    if media.contains("://") || media.contains('/') || media.contains(':') {
        return Err(ConfigError::MediaDomainNotAHost(media));
    }
    if app.as_deref() == Some(media.as_str()) {
        return Err(ConfigError::MediaDomainIsAppDomain(media));
    }
    Ok(Some(media))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(domain: Option<&str>, media: Option<&str>) -> Config {
        Config {
            data_dir: PathBuf::from("/tmp/linger"),
            bind: "0.0.0.0:8420".parse().unwrap(),
            domain: domain.map(str::to_string),
            media_domain: media_domain(domain, media.map(str::to_string)).unwrap(),
            storage: Storage::Local,
            s3: None,
            pool_bytes: DEFAULT_POOL_BYTES,
            file_expiry_days: Some(DEFAULT_FILE_EXPIRY_DAYS),
        }
    }

    #[test]
    fn files_are_served_from_their_own_host_by_default() {
        let config = config(Some("linger.example"), None);
        assert_eq!(config.public_origin(), "https://linger.example");
        assert_eq!(config.media_origin(), "https://cdn.linger.example");
        assert_eq!(
            config.object_url("ab/cd/abcd"),
            "https://cdn.linger.example/objects/ab/cd/abcd"
        );
    }

    #[test]
    fn a_host_can_name_the_media_origin_itself() {
        let config = config(Some("linger.example"), Some("Files.Example "));
        assert_eq!(config.media_host(), Some("files.example"));
        assert_eq!(config.media_origin(), "https://files.example");
    }

    #[test]
    fn a_server_with_no_domain_has_one_origin_and_relative_urls() {
        let config = config(None, None);
        assert_eq!(config.media_host(), None);
        assert_eq!(config.object_url("ab/cd/abcd"), "/objects/ab/cd/abcd");
    }

    #[test]
    fn serving_files_from_the_app_origin_is_refused() {
        let err =
            media_domain(Some("linger.example"), Some("LINGER.example".to_string())).unwrap_err();
        assert!(matches!(err, ConfigError::MediaDomainIsAppDomain(_)));
    }

    #[test]
    fn a_url_where_a_hostname_belongs_is_refused() {
        for wrong in ["https://cdn.linger.example", "cdn.linger.example/files"] {
            assert!(
                matches!(
                    media_domain(Some("linger.example"), Some(wrong.to_string())),
                    Err(ConfigError::MediaDomainNotAHost(_))
                ),
                "{wrong} should not be accepted"
            );
        }
    }

    #[test]
    fn a_pool_size_can_be_written_the_way_a_person_would_write_it() {
        assert_eq!(pool_bytes(None).unwrap(), DEFAULT_POOL_BYTES);
        assert_eq!(
            pool_bytes(Some("  ".to_string())).unwrap(),
            DEFAULT_POOL_BYTES
        );
        assert_eq!(pool_bytes(Some("2TB".to_string())).unwrap(), 2 * (1 << 40));
        assert_eq!(
            pool_bytes(Some("250gb".to_string())).unwrap(),
            250 * (1 << 30)
        );
        assert_eq!(
            pool_bytes(Some("800 MB".to_string())).unwrap(),
            800 * (1 << 20)
        );
        assert_eq!(
            pool_bytes(Some("53687091200".to_string())).unwrap(),
            DEFAULT_POOL_BYTES
        );
    }

    #[test]
    fn a_pool_size_that_is_not_a_size_is_a_startup_error() {
        // The last one overflows the multiply rather than the parse, which is
        // the case a `checked_mul` is there for.
        for wrong in ["fifty gigs", "50 GiB", "-1", "99999999TB"] {
            assert!(
                matches!(
                    pool_bytes(Some(wrong.to_string())),
                    Err(ConfigError::PoolBytes(_))
                ),
                "{wrong:?} should not be accepted"
            );
        }
    }

    #[test]
    fn a_pool_too_small_for_one_file_is_refused() {
        assert!(matches!(
            pool_bytes(Some("100MB".to_string())),
            Err(ConfigError::PoolTooSmall(_))
        ));
    }

    #[test]
    fn expiry_defaults_to_a_year_and_can_be_turned_off() {
        assert_eq!(
            file_expiry_days(None).unwrap(),
            Some(DEFAULT_FILE_EXPIRY_DAYS)
        );
        assert_eq!(file_expiry_days(Some("30".to_string())).unwrap(), Some(30));
        for off in ["off", "OFF", "never", "0"] {
            assert_eq!(
                file_expiry_days(Some(off.to_string())).unwrap(),
                None,
                "{off}"
            );
        }
        assert!(matches!(
            file_expiry_days(Some("a year".to_string())),
            Err(ConfigError::FileExpiryDays(_))
        ));
    }
}
