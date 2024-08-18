use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, put},
    Router,
};
use chrono::{Duration, Local, NaiveTime};
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
        .route("/device/alarm", get(alarm))
        .route("/device/next", put(update_next_id))
}

/// Returns Unix epoch timestamp in server's timezone for device RTC.
async fn rtc() -> String {
    let timestamp = Local::now().timestamp().to_string();

    debug!("Returning timestamp for real time clock: {timestamp}");

    timestamp
}

/// Returns Unix epoch timestamp for next display refresh (i.e., at midnight in server's timezone);
async fn alarm() -> String {
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let tomorrow = Local::now().with_time(midnight).unwrap() + Duration::days(1);
    let timestamp = tomorrow.timestamp().to_string();

    debug!("Returning timestamp for alarm: {timestamp}");

    timestamp
}

/// Set ID for next image to display.
async fn update_next_id(
    State(state): State<AppState>,
    Query(params): Query<UpdateNextParams>,
) -> StatusCode {
    match db::set_next_id(&state.pool, params.id).await {
        Ok(_) => {
            debug!("Updated next ID to {}", params.id);
            StatusCode::OK
        }
        Err(err) => {
            error!("Failed to update next ID: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
