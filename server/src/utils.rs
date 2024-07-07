use axum::response::Redirect;

pub fn redirect_error() -> Redirect {
    Redirect::to(format!("/error").as_ref())
}
