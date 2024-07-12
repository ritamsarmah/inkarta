use anyhow::Error;
use axum::{http::StatusCode, response::Redirect};

pub fn handle_error(error: Error, status: StatusCode) -> Redirect {
    // TODO: Add error to tracing
    // Redirect::to(format!("/r").as_ref())
    Redirect::to("/")
}
