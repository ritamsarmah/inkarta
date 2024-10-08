use std::sync::Arc;

use anyhow::Result;
use axum::{extract::DefaultBodyLimit, Router};
use server::{db, routes, state::AppState};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    ConnectOptions, SqlitePool,
};
use tokio::net::TcpListener;
use tracing::debug;

const DATABASE_URL: &str = "inkarta.db";
const TCP_PORT: u16 = 5000;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = AppState {
        templates: Arc::new(create_template_env()?),
        pool: create_database_pool().await?,
    };

    let app = Router::new()
        .merge(routes::device::router())
        .merge(routes::image::router())
        .merge(routes::ui::router())
        .layer(DefaultBodyLimit::disable()) // Disable body limit to allow large image uploads
        .with_state(state);

    let addr = format!("0.0.0.0:{TCP_PORT}");
    let listener = TcpListener::bind(&addr).await?;

    debug!("Listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_database_pool() -> Result<sqlx::Pool<sqlx::Sqlite>> {
    let options = SqliteConnectOptions::new()
        .filename(DATABASE_URL)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .disable_statement_logging();
    let pool = SqlitePool::connect_with(options).await?;

    db::initialize(&pool).await?;

    Ok(pool)
}

fn create_template_env() -> Result<minijinja::Environment<'static>> {
    let mut env = minijinja::Environment::new();

    // Embed templates directly in binary
    env.add_template("404.jinja", include_str!("templates/404.jinja"))?;
    env.add_template("base.jinja", include_str!("templates/base.jinja"))?;
    env.add_template("index.jinja", include_str!("templates/index.jinja"))?;
    env.add_template("modal.jinja", include_str!("templates/modal.jinja"))?;

    env.add_template(
        "partials/upload.jinja",
        include_str!("templates/partials/upload.jinja"),
    )?;
    env.add_template(
        "partials/device.jinja",
        include_str!("templates/partials/device.jinja"),
    )?;
    env.add_template(
        "partials/image.jinja",
        include_str!("templates/partials/image.jinja"),
    )?;

    Ok(env)
}

// fn create_template_reloader() -> AutoReloader {
//     let reloader = AutoReloader::new(move |notifier| {
//         let template_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
//         let mut env = minijinja::Environment::new();
//         env.set_loader(minijinja::path_loader(&template_path));

//         notifier.set_fast_reload(true);
//         notifier.watch_path(&template_path, true);

//         Ok(env)
//     });

//     reloader
// }
