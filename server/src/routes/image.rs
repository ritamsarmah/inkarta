use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{db, model::Identifier, state::AppState, utils};

// #[derive(Deserialize)]
// struct FetchImageQuery {
//     width: Option<u32>,
//     height: Option<u32>,
// }

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/image", post(create_image))
        .route("/image/:id", delete(delete_image))
        .route("/image/next", get(fetch_next_image))
}

// async fn get_image(
//     Path(id): Path<Identifier>,
//     Query(query): Query<FetchImageQuery>,
//     State(state): State<AppState>,
// ) -> impl IntoResponse {
//     db::get_image(, )
//     todo!()
// }

async fn delete_image(Path(id): Path<Identifier>, State(state): State<AppState>) -> Redirect {
    match db::delete_image(&state.pool, id).await {
        Ok(_) => Redirect::to("/"),
        Err(error) => Redirect::to(format!("/error/{}", error).as_ref()),
    }
}

async fn fetch_next_image() -> impl IntoResponse {
    todo!()
}

async fn create_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut title = None;
    let mut artist = None;
    let mut dark = false;
    let mut data = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        match name.as_str() {
            "title" => title = field.text().await.ok(),
            "artist" => artist = field.text().await.ok(),
            "dark" => dark = field.text().await.ok().map_or(false, |value| value == "on"),
            "data" => {
                data = field.bytes().await.ok().map(|bytes| bytes.to_vec());
            }
            _ => {}
        };
    }

    if let Some(title) = title {
        db::create_image(&title).await;
        Redirect::to("/")
    } else {
        utils::redirect_error("error".into())
    }
}
