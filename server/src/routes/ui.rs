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
struct JinjaThumbnail {
    title: String,
    artist: String,
    href: String,
    src: String,
}

#[derive(Serialize)]
struct JinjaImage {
    id: Identifier,
    title: String,
    artist: String,
    src: String,
}

#[derive(Serialize)]
struct JinjaFrame {
    name: String,
    next: String,
    current: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(gallery))
        .route("/error/:code", get(error_page))
        .route("/x/upload", get(partial_upload))
        .route("/x/frame", get(partial_frame))
        .route("/x/image/:id", get(partial_image))
        .fallback(not_found)
}

async fn gallery(State(state): State<AppState>) -> impl IntoResponse {
    // TODO: paginate? and implement infinite scroll
    match db::get_thumbnails(&state.pool).await {
        Ok(thumbnails) => {
            let env = state.reloader.acquire_env().unwrap();
            let template = env.get_template("index.jinja").unwrap();
            let thumbnails: Vec<JinjaThumbnail> = thumbnails
                .into_iter()
                .map(|thumbnail| JinjaThumbnail {
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
        Err(err) => utils::redirect_error(err, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn error_page(Path(code): Path<u16>, State(state): State<AppState>) -> impl IntoResponse {
    let status = StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST);
    let message = status.canonical_reason().unwrap_or("Unknown Error");
    let code = status.as_u16();

    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("error.jinja").unwrap();
    let html = template
        .render(context!(message => message, code => code))
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

async fn partial_upload(State(state): State<AppState>) -> impl IntoResponse {
    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("partials/upload.jinja").unwrap();
    let html = template.render(()).unwrap();

    Html(html).into_response()
}

async fn partial_frame(State(state): State<AppState>) -> impl IntoResponse {
    let context = db::get_frame(&state.pool).await.map_or_else(
        || context!(frame => ()),
        |frame| {
            println!("{frame:?}");
            let frame = JinjaFrame {
                name: frame.name,
                next: frame.next.map_or_else(
                    || "Next image has not been selected".to_string(),
                    |x| x.to_string(),
                ),
                current: frame.current.map_or_else(
                    || "No image currently showing".to_string(),
                    |x| x.to_string(),
                ),
            };
            context!(frame => frame)
        },
    );

    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("partials/frame.jinja").unwrap();
    let html = template.render(context).unwrap();

    Html(html).into_response()
}

async fn partial_image(
    Path(id): Path<Identifier>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match db::get_image(&state.pool, id).await {
        Ok(image) => {
            let image = JinjaImage {
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
        Err(err) => utils::redirect_error(err, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

/* Utilities */

fn to_src(data: Vec<u8>) -> String {
    format!("data:image/bmp;base64,{}", BASE64_STANDARD.encode(data))
}
