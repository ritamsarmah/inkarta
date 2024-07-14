use anyhow::Error;
use axum::{http::StatusCode, response::Redirect};
use tracing::error;

pub fn redirect_error(err: Error, status: StatusCode) -> Redirect {
    error!("{err}");
    // Redirect::to(format!("/r").as_ref())
    Redirect::to("/")
}
