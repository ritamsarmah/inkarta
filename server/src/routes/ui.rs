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
        .route("/ui/upload", get(partial_upload))
        .route("/ui/frame", get(partial_frame))
        .route("/ui/image/:id", get(partial_image))
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
                    href: format!("/ui/image/{}", thumbnail.id),
                    src: to_src(thumbnail.thumbnail),
                })
                .collect();

            let html = template
                .render(context! {
                    thumbnails => thumbnails,
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
    let context = match db::get_frame(&state.pool).await {
        Some(frame) => {
            let next_title = if let Some(next) = frame.next {
                db::get_image(&state.pool, next).await.unwrap().title
            } else {
                "None".to_string()
            };

            let current_title = if let Some(current) = frame.current {
                db::get_image(&state.pool, current).await.unwrap().title
            } else {
                "None".to_string()
            };

            let frame = JinjaFrame {
                name: frame.name,
                next: next_title,
                current: current_title,
            };
            context!(frame => frame)
        }
        None => context!(frame => ()),
    };

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

            let next_id = db::get_next_id(&state.pool).await;

            let env = state.reloader.acquire_env().unwrap();
            let template = env.get_template("partials/image.jinja").unwrap();
            let html = template
                .render(context!(image => image, next_id => next_id))
                .unwrap();

            Html(html).into_response()
        }
        Err(err) => utils::redirect_error(err, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

/* Utilities */

fn to_src(data: Vec<u8>) -> String {
    format!("data:image/bmp;base64,{}", BASE64_STANDARD.encode(data))
}
