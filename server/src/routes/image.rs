use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    model::{Identifier, Image},
    state::AppState,
};

/// Supported image formats for upload
const IMAGE_FORMATS: [&str; 6] = ["bmp", "png", "jpg", "jpeg", "tiff", "tif"];

#[derive(Deserialize)]
struct FetchImageQuery {
    width: Option<u32>,
    height: Option<u32>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/image/:id", get(get_image))
        .route("/image/next", get(fetch_next_image))
        .route("/image", post(create_image))
}

async fn get_image(
    Path(id): Path<Identifier>,
    Query(query): Query<FetchImageQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    todo!()
}

async fn update_image(
    Path(id): Path<Identifier>,
    Json(payload): Json<Image>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    todo!()
}

async fn delete_image(
    Path(id): Path<Identifier>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    todo!()
}

async fn fetch_next_image() -> impl IntoResponse {
    todo!()
}

async fn create_image() -> impl IntoResponse {
    todo!()
}
