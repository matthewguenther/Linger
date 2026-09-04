//! The voice relay's front door (SPEC §4.14, PROTOCOL §7, T-1403).
//!
//! One endpoint, and it stores nothing: a member asks what to put in their
//! peer connections, and gets the host's relay addresses with a password that
//! was computed for them on the spot and dies on its own (`crate::turn`). No
//! relay configured is an empty answer, not an error — voice between machines
//! on one network is a real thing a server can offer, and the client joins
//! anyway.
//!
//! Audio never comes here. The server's whole part in voice is introducing
//! two clients to each other over the gateway; this is the one extra thing
//! it says at the introduction, which is where to meet when neither can reach
//! the other's door.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use linger_core::wire::IceServers;

use crate::auth::AuthedUser;
use crate::error::ApiError;
use crate::state::AppState;
use crate::turn;

pub fn router() -> Router<AppState> {
    Router::new().route("/voice/ice", get(ice))
}

/// `GET /voice/ice` — the relay, for the member asking, for the next while.
async fn ice(
    State(state): State<AppState>,
    auth: AuthedUser,
) -> Result<Json<IceServers>, ApiError> {
    let answer = match &state.config.turn {
        Some(turn) => turn::ice_servers(turn, auth.id, turn::now_unix()),
        None => IceServers {
            servers: Vec::new(),
            ttl_secs: 0,
        },
    };
    Ok(Json(answer))
}
