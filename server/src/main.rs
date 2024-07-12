use std::{path::Path, sync::Arc};

use anyhow::Result;
use axum::{extract::DefaultBodyLimit, Router};
use minijinja_autoreload::AutoReloader;
use server::{db, routes, state::AppState};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tokio::net::TcpListener;

const DATABASE_URL: &str = "inkarta.db";
const TCP_PORT: u16 = 5000;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = AppState {
        reloader: Arc::new(create_template_reloader()),
        pool: create_database_pool().await?,
    };

    let app = Router::new()
        .merge(routes::image::router())
        .merge(routes::setting::router())
        .merge(routes::ui::router())
        .layer(DefaultBodyLimit::disable()) // Disable body limit to allow large image uploads
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{TCP_PORT}")).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_database_pool() -> Result<sqlx::Pool<sqlx::Sqlite>> {
    let options = SqliteConnectOptions::new()
        .filename(DATABASE_URL)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    db::initialize(&pool).await?;

    Ok(pool)
}

fn create_template_reloader() -> AutoReloader {
    let reloader = AutoReloader::new(move |notifier| {
        let template_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
        let mut env = minijinja::Environment::new();
        env.set_loader(minijinja::path_loader(&template_path));

        notifier.set_fast_reload(true);
        notifier.watch_path(&template_path, true);

        Ok(env)
    });

    reloader
}

// fn create_template_env() -> minijinja::Environment<'static> {
// let mut env = minijinja::Environment::new();
// let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
// // TODO: Setup dynamic loader
// env.set_loader(minijinja::path_loader(path));
// }
