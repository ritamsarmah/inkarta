use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::put,
    Router,
};
use serde::Deserialize;
use tracing::debug;

use crate::{db, model::Identifier, state::AppState};

#[derive(Deserialize)]
pub struct DeviceParams {
    name: String,
}

#[derive(Deserialize)]
pub struct UpdateNextParams {
    id: Identifier,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/frame/register", put(register_device))
        .route("/frame/next", put(update_next_id))
}

/// Initializes picture frame and registers it to server.
/// Response is Unix epoch timestamp in server's timezone.
async fn register_device(
    State(state): State<AppState>,
    params: Query<DeviceParams>,
) -> impl IntoResponse {
    let name = &params.name;

    debug!("Initializing frame named: {name}");
    db::register_frame(&state.pool, name).await.unwrap();

    // Return current time for device's RTC
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

async fn update_next_id(
    State(state): State<AppState>,
    params: Query<UpdateNextParams>,
) -> impl IntoResponse {
    match db::update_next_id(&state.pool, params.id).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
}
