use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use serde::Deserialize;

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
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

async fn update_next_id(State(state): State<AppState>, Query(params): Query<UpdateNextParams>) {
    db::set_next_id(&state.pool, params.id);
}
