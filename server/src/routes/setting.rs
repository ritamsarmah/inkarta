use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, put},
    Router,
};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/setting/next", put(set_next))
        .route("/setting/rtc", get(get_epoch_timestamp))
}

async fn set_next(State(state): State<AppState>) -> impl IntoResponse {
    todo!()
}

async fn get_epoch_timestamp() -> impl IntoResponse {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn valid_epoch_timestamp() {
//         assert!(false);
//     }
// }
