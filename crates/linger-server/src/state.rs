//! Shared application state. Cloned per-request by axum; everything inside is
//! already cheap to clone (pools are handles, config is small).

use std::sync::Arc;

use crate::config::Config;
use crate::db::Db;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub config: Arc<Config>,
}

impl AppState {
    #[must_use]
    pub fn new(db: Db, config: Config) -> Self {
        Self { db, config: Arc::new(config) }
    }
}
