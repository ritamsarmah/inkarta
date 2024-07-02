use anyhow::Result;
use axum::{extract::State, routing::get, Router};
use maud::{html, Markup};
use server::{db, routes, state::AppState};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Create and connect to database
    let options = SqliteConnectOptions::new()
        .filename("images.db")
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options);

    let state = AppState {
        current: None,
        next: None,
        pool,
    };

    let app = Router::new()
        .route("/", get(home))
        .merge(routes::image::router())
        .merge(routes::setting::router())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or("5000".into());
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn home(State(state): State<AppState>) -> Markup {
    let gallery = match db::get_images(&state.pool).await {
        Ok(images) => html! {
            @let count = images.len();

            h2 { (format!("{count} images")) }
            @for image in images {
                p { (image.title) }
            }
        },
        Err(error) => html! {
            p { (format!("Failed to retrieve images with error: {error}")) }
        },
    };

    html! {
        h1 { "Hello" }
        (gallery)
    }
}
