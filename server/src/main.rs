use std::path::Path;

use anyhow::Result;
use axum::{routing::get_service, Router};
use server::{db, routes, state::AppState};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tokio::net::TcpListener;
use tower_http::services::ServeFile;

const DATABASE_URL: &str = "inkarta.db";
const TCP_PORT: u16 = 5000;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Create and connect to database
    let options = SqliteConnectOptions::new()
        .filename(DATABASE_URL)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    db::initialize(&pool).await?;

    // Initialize app state and routes
    let styles_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("public")
        .join("styles.css");
    let state = AppState { pool };
    let app = Router::new()
        .merge(routes::image::router())
        .merge(routes::setting::router())
        .merge(routes::ui::router())
        .route("/styles.css", get_service(ServeFile::new(styles_path)))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{TCP_PORT}")).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
