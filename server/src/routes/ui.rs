use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use base64::prelude::*;
use minijinja::context;
use serde::Serialize;

use crate::{db, state::AppState, utils};

#[derive(Serialize)]
struct TemplateThumbnail {
    title: String,
    artist: String,
    href: String,
    src: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        // .route("/error/:message", get(error_page))
        // .route("/x/image/:id", get(image_partial))
        // .route("/x/upload", get(upload_partial))
        .fallback(not_found_page)
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
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
        Err(error) => {
            println!("{error}");
            utils::redirect_error().into_response()
        }
    }
}

async fn not_found_page(State(state): State<AppState>) -> Html<String> {
    let env = state.reloader.acquire_env().unwrap();
    let template = env.get_template("404.jinja").unwrap();
    let html = template.render(()).unwrap();
    Html(html)
}

fn to_src(data: Vec<u8>) -> String {
    format!("data:image/bmp;base64,{}", BASE64_STANDARD.encode(&data))
}
