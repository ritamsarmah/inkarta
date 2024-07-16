use anyhow::Error;
use axum::{http::StatusCode, response::Redirect};
use tracing::error;

pub fn redirect_error(err: Error, status: StatusCode) -> Redirect {
    error!("{status} - {err}");
    let path = format!("/error/{code}", code = status.as_u16());
    Redirect::to(&path)
}
