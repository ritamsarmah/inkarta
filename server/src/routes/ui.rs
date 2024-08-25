use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use base64::prelude::*;
use minijinja::context;
use serde::Serialize;
use tracing::debug;

use crate::{db, model::Identifier, state::AppState};

#[derive(Serialize)]
struct JinjaThumbnail {
    title: String,
    artist: String,
    href: String,
    src: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(gallery))
        .route("/ui/upload", get(partial_upload))
        .route("/ui/device", get(partial_device))
        .route("/ui/image/:id", get(partial_image))
        .fallback(not_found)
}

async fn gallery(State(state): State<AppState>) -> Html<String> {
    let thumbnails: Vec<JinjaThumbnail> = db::get_thumbnails(&state.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|thumbnail| JinjaThumbnail {
            title: thumbnail.title,
            artist: thumbnail.artist,
            href: format!("/ui/image/{}", thumbnail.id),
            src: to_src(thumbnail.thumbnail, "jpeg"),
        })
        .collect();

    // TODO: Paginate with infinite scroll?
    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("index.jinja").unwrap();
    let html = template
        .render(context! {
            thumbnails => thumbnails,
        })
        .unwrap();

    Html(html)
}

async fn not_found(State(state): State<AppState>) -> Html<String> {
    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("404.jinja").unwrap();
    let html = template.render(()).unwrap();

    Html(html)
}

/* Partials */

async fn partial_upload(State(state): State<AppState>) -> Html<String> {
    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("partials/upload.jinja").unwrap();
    let html = template.render(()).unwrap();

    Html(html)
}

async fn partial_device(State(state): State<AppState>) -> Html<String> {
    let current_title = db::get_current_title(&state.pool).await;
    let next_title = db::get_next_title(&state.pool).await;

    debug!("Current title: {current_title:?}");
    debug!("Next title: {next_title:?}");

    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("partials/device.jinja").unwrap();
    let html = template
        .render(context! {
            current => current_title,
            next => next_title
        })
        .unwrap();

    Html(html)
}

async fn partial_image(Path(id): Path<Identifier>, State(state): State<AppState>) -> Html<String> {
    let html = match db::get_image(&state.pool, id).await {
        Ok(image) => {
            let next_id = db::get_next_id(&state.pool).await;

            let env = state.reloader.acquire_env().unwrap();
            let template = env.get_template("partials/image.jinja").unwrap();

            template
                .render(context! {
                    id => image.id,
                    title => image.title,
                    artist => image.artist,
                    src => to_src(image.data, "png"),
                    next_id => next_id
                })
                .unwrap()
        }
        Err(err) => format!("<p>Failed to load image: {err}</p>"),
    };

    Html(html)
}

/* Utilities */

fn to_src(data: Vec<u8>, format: &str) -> String {
    format!(
        "data:image/{};base64,{}",
        format,
        BASE64_STANDARD.encode(data)
    )
}
