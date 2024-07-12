use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use base64::prelude::*;
use minijinja::context;
use serde::Serialize;

use crate::{db, model::Identifier, state::AppState, utils};

#[derive(Serialize)]
struct TemplateThumbnail {
    title: String,
    artist: String,
    href: String,
    src: String,
}

#[derive(Serialize)]
struct TemplateImage {
    id: Identifier,
    title: String,
    artist: String,
    src: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(gallery))
        .route("/x/upload", get(partial_upload))
        .route("/x/settings", get(partial_settings))
        .route("/x/image/:id", get(partial_image))
        .fallback(not_found)
}

async fn gallery(State(state): State<AppState>) -> impl IntoResponse {
    // TODO: paginate? and implement infinite scroll
    match db::get_thumbnails(&state.pool).await {
        Ok(thumbnails) => {
            let env = state.reloader.acquire_env().unwrap();
            let template = env.get_template("index.jinja").unwrap();
            let thumbnails: Vec<TemplateThumbnail> = thumbnails
                .into_iter()
                .map(|thumbnail| TemplateThumbnail {
                    title: thumbnail.title,
                    artist: thumbnail.artist,
                    href: format!("/x/image/{}", thumbnail.id),
                    src: to_src(thumbnail.thumbnail),
                })
                .collect();

            let html = template
                .render(context! {
                    thumbnails => thumbnails,
                    debug => cfg!(debug_assertions),
                })
                .unwrap();

            Html(html).into_response()
        }
        Err(error) => utils::handle_error(error, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn not_found(State(state): State<AppState>) -> Html<String> {
    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("404.jinja").unwrap();
    let html = template.render(()).unwrap();
    Html(html)
}

/* Partials */

async fn partial_upload(State(state): State<AppState>) -> impl IntoResponse {
    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("partials/upload.jinja").unwrap();
    let html = template.render(()).unwrap();

    Html(html).into_response()
}

async fn partial_settings(State(state): State<AppState>) -> impl IntoResponse {
    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("partials/settings.jinja").unwrap();
    let html = template.render(()).unwrap();

    Html(html).into_response()
}

async fn partial_image(
    Path(id): Path<Identifier>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match db::get_image(&state.pool, id).await {
        Ok(image) => {
            let image = TemplateImage {
                id: image.id,
                title: image.title,
                artist: image.artist,
                src: to_src(image.data),
            };

            let env = state.reloader.acquire_env().unwrap();
            let template = env.get_template("partials/image.jinja").unwrap();
            let html = template.render(context!(image => image)).unwrap();

            Html(html).into_response()
        }
        Err(error) => utils::handle_error(error, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

/* Utilities */

fn to_src(data: Vec<u8>) -> String {
    format!("data:image/bmp;base64,{}", BASE64_STANDARD.encode(data))
}
