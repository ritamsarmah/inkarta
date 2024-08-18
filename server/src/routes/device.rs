use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use serde::Deserialize;

use crate::{model::Identifier, state::AppState};

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

async fn update_next_id(
    State(state): State<AppState>,
    Query(params): Query<UpdateNextParams>,
) -> impl IntoResponse {
    // db::update_next_id
    panic!("This is a test panic");
    // match db::update_next_id(&state.pool, params.id).await {
    //     Ok(_) => StatusCode::OK.into_response(),
    //     Err(err) => utils::redirect_error(err, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    // };
}
