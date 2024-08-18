use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use serde::Deserialize;
use tracing::{debug, error};

use crate::{db, model::Identifier, state::AppState};

#[derive(Deserialize)]
pub struct UpdateNextParams {
    id: Identifier,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/device/rtc", get(rtc))
        .route("/device/next", put(update_next_id))
}

/// Returns Unix epoch timestamp in server's timezone for device RTC.
async fn rtc() -> impl IntoResponse {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    debug!("Returning timestamp for real-time clock: {timestamp}");

    timestamp
}

async fn update_next_id(State(state): State<AppState>, Query(params): Query<UpdateNextParams>) {
    match db::set_next_id(&state.pool, params.id).await {
        Ok(_) => debug!("Updated next ID to {}", params.id),
        Err(err) => error!("Failed to update next ID: {err}"),
    };
}
