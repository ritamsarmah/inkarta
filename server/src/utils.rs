use anyhow::Error;
use axum::{http::StatusCode, response::Redirect};
use tracing::{event, Level};

pub fn handle_error(error: Error, status: StatusCode) -> Redirect {
    event!(Level::ERROR, "{error}");
    // TODO: Add error to tracing
    // Redirect::to(format!("/r").as_ref())
    Redirect::to("/")
}
