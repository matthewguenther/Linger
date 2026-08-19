//! Operational health endpoint. Not part of the wire contract in PROTOCOL.md —
//! it exists for reverse proxies, uptime checks, and the client's status bar
//! latency probe. Keep it dependency-free and instant.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    // Prove the database file is actually reachable, not just that we're up.
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&state.db.read)
        .await
        .is_ok();
    Json(json!({
        "ok": db_ok,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
