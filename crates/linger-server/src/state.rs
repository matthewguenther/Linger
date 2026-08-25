//! Shared application state. Cloned per-request by axum; everything inside is
//! a handle (pools, Arcs), so clones are cheap.

use std::sync::Arc;

use crate::auth::JwtKeys;
use crate::config::{Config, Storage};
use crate::db::Db;
use crate::gateway::Gateway;
use crate::ratelimit::RateLimiter;
use crate::setup::SetupState;
use crate::storage::{LocalStore, ObjectStore};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub config: Arc<Config>,
    pub jwt: Arc<JwtKeys>,
    pub limiter: Arc<RateLimiter>,
    pub gateway: Arc<Gateway>,
    pub setup: Arc<SetupState>,
    /// Where uploaded bytes live (ARCHITECTURE §8).
    pub storage: Arc<dyn ObjectStore>,
    /// The same store again, when it is the local one. Only the local backend
    /// runs an upload listener on this server — with S3 the client PUTs at S3 —
    /// so the listener route needs the concrete type, and its absence is how
    /// that route knows it has nothing to do.
    pub local: Option<Arc<LocalStore>>,
}

impl AppState {
    /// Wire everything up: JWT keys from the data dir, setup armed iff the
    /// server has no users yet.
    pub async fn build(db: Db, config: Config) -> anyhow::Result<Self> {
        let jwt = JwtKeys::load_or_generate(&config.data_dir)?;
        let (user_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&db.read)
            .await?;
        let config = Arc::new(config);
        if config.storage == Storage::S3 {
            anyhow::bail!(
                "LINGER_STORAGE=s3 isn't wired up yet — set it to 'local', which keeps \
                 uploads in the data directory next to linger.db."
            );
        }
        let local = Arc::new(LocalStore::open(config.clone())?);
        Ok(Self {
            db,
            config,
            jwt: Arc::new(jwt),
            limiter: Arc::new(RateLimiter::new()),
            gateway: Arc::new(Gateway::new()),
            setup: Arc::new(SetupState::new(user_count == 0)),
            storage: local.clone(),
            local: Some(local),
        })
    }
}
