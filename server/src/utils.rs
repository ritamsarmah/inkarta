use anyhow::Error;
use axum::http::StatusCode;

/* Error Handling */

pub fn not_found_error(err: Error) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, err.to_string())
}

pub fn server_error(err: Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
