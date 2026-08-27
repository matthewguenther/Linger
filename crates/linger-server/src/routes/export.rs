//! Export (SPEC §4.11, PROTOCOL §7, T-801).
//!
//! Two doors, both open to every member. There is no host approval here and
//! there should never be one: an export is the promise that leaving is possible,
//! and a promise somebody else can refuse is not one.
//!
//! The rate limit is the only gate, and it is about the host's disk and CPU
//! rather than about permission — one archive an hour, per member.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use linger_core::limits::RATE_EXPORT;
use linger_core::wire::{ExportJob, ExportStarted};
use linger_core::ExportId;

use crate::auth::AuthedUser;
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/export", post(start))
        .route("/export/{job_id}", get(job))
}

/// `POST /export` — start building this member's archive.
async fn start(
    State(state): State<AppState>,
    auth: AuthedUser,
) -> Result<Json<ExportStarted>, ApiError> {
    // Keyed by member, not by address: two people on one connection are two
    // people, and one person on two machines is still one person asking a
    // server to zip itself.
    if let Err(retry_after_ms) = state
        .limiter
        .check(&format!("export:{}", auth.id), RATE_EXPORT)
    {
        return Err(ApiError::rate_limited(retry_after_ms));
    }
    let job_id = crate::export::start(&state, auth.id).await?;
    Ok(Json(ExportStarted { job_id }))
}

/// `GET /export/:job_id` — how far along, and where to get it.
async fn job(
    State(state): State<AppState>,
    auth: AuthedUser,
    Path(job_id): Path<ExportId>,
) -> Result<Json<ExportJob>, ApiError> {
    crate::export::job(&state, job_id, auth.id)
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("No such export."))
}
