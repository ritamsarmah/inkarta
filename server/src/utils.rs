use anyhow::Error;
use axum::{http::StatusCode, response::Redirect};
use tracing::{event, Level};

pub fn redirect_error(err: Error, status: StatusCode) -> Redirect {
    event!(Level::ERROR, "{err}");
    // TODO: Add error to tracing
    // Redirect::to(format!("/r").as_ref())
    Redirect::to("/")
}
