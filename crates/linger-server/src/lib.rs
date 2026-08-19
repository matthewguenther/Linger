//! linger-server: the stoop. One Rust binary, one data directory.
//!
//! Library layout exists so integration tests can build the exact production
//! router against a temp SQLite file (AGENTS.md testing rules). `main.rs` is a
//! thin shell around [`config`] + [`db`] + [`app`].

pub mod config;
pub mod db;
pub mod error;
pub mod routes;
pub mod state;

pub use state::AppState;

/// Build the full application router. Everything REST lives under `/api/v1`
/// (PROTOCOL preamble); the gateway will mount at `/api/v1/gateway` in M2.
pub fn app(state: AppState) -> axum::Router {
    routes::router(state)
}
