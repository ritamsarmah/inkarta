use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::Deserialize;

use crate::{db, state::AppState};

#[derive(Deserialize)]
struct FetchImageQuery {
    width: Option<u32>,
    height: Option<u32>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        // .route("/image/:id", get(get_image).put(update_image))
        // .route("/image/next", get(fetch_next_image))
        .route("/image", post(create_image))
}

// async fn get_image(
//     Path(id): Path<Identifier>,
//     Query(query): Query<FetchImageQuery>,
//     State(state): State<AppState>,
// ) -> impl IntoResponse {
//     todo!()
// }

// async fn update_image(
//     Path(id): Path<Identifier>,
//     Json(payload): Json<Image>,
//     State(state): State<AppState>,
// ) -> impl IntoResponse {
//     todo!()
// }

// async fn delete_image(
//     Path(id): Path<Identifier>,
//     State(state): State<AppState>,
// ) -> impl IntoResponse {
//     todo!()
// }

// async fn fetch_next_image() -> impl IntoResponse {
//     todo!()
// }

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
        }
    }

    if let (Some(title), Some(data)) = (title, data) {
        // TODO: Convert to bitmap
        // TODO: Create thumbnail
        let thumbnail = data.clone();
        let background = if dark { "#000000" } else { "#FFFFFF" };

        match db::create_image(
            &state.pool,
            &title,
            artist.as_deref(),
            background.into(),
            data,
            thumbnail,
        )
        .await
        {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}
