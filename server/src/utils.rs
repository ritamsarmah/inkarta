use axum::response::Redirect;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

pub fn redirect_error(message: String) -> Redirect {
    let message = utf8_percent_encode(&message, NON_ALPHANUMERIC);
    Redirect::to(format!("/error/{message}").as_ref())
}
