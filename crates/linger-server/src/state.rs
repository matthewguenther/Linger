//! Shared application state. Cloned per-request by axum; everything inside is
//! a handle (pools, Arcs), so clones are cheap.

use std::sync::Arc;

use crate::auth::JwtKeys;
use crate::config::Config;
use crate::db::Db;
use crate::gateway::Gateway;
use crate::ratelimit::RateLimiter;
use crate::setup::SetupState;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub config: Arc<Config>,
    pub jwt: Arc<JwtKeys>,
    pub limiter: Arc<RateLimiter>,
    pub gateway: Arc<Gateway>,
    pub setup: Arc<SetupState>,
}

impl AppState {
    /// Wire everything up: JWT keys from the data dir, setup armed iff the
    /// stoop has no users yet.
    pub async fn build(db: Db, config: Config) -> anyhow::Result<Self> {
        let jwt = JwtKeys::load_or_generate(&config.data_dir)?;
        let (user_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&db.read)
            .await?;
        Ok(Self {
            db,
            config: Arc::new(config),
            jwt: Arc::new(jwt),
            limiter: Arc::new(RateLimiter::new()),
            gateway: Arc::new(Gateway::new()),
            setup: Arc::new(SetupState::new(user_count == 0)),
        })
    }
}
