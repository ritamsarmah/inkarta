use anyhow::Result;
use axum::{
    routing::{get, put},
    Router,
};
use maud::{html, Markup};
use tokio::net::TcpListener;

const IMAGE_FORMATS: [&str; 6] = ["bmp", "png", "jpg", "jpeg", "tiff", "tif"];

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {};

    let app = Router::new()
        .route("/", get(home))
        .fallback(not_found)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or("5000".into());
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn home() -> Markup {
    html! {
        h1 { "Hello" }
    }
}

async fn not_found() -> Markup {
    html! {
        h1 { "Page not found" }
        a href="/" { "Return Home" }
    }
}
