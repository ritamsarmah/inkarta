use std::io::Cursor;

use anyhow::anyhow;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
    Json, Router,
};
use image::{
    imageops::{resize, FilterType},
    load_from_memory, DynamicImage, ImageFormat,
};
use serde::Deserialize;
use tracing::{event, Level};

use crate::{db, model::Identifier, state::AppState, utils};

const THUMBNAIL_SIZE: u32 = 256;

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
        Err(error) => utils::handle_error(error, StatusCode::INTERNAL_SERVER_ERROR),
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
    let mut bitmap: Vec<u8> = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "title" => title = field.text().await.ok(),
            "artist" => artist = field.text().await.ok(),
            "dark" => dark = field.text().await.ok().map_or(false, |value| value == "on"),
            "image" => {
                let data = field.bytes().await.unwrap();
                let img = load_from_memory(&data).unwrap();
                match img.write_to(&mut Cursor::new(&mut bitmap), ImageFormat::Bmp) {
                    Ok(_) => event!(Level::DEBUG, "Generated bitmap from uploaded image"),
                    Err(error) => {
                        return utils::handle_error(
                            anyhow!("Failed to generate bitmap from uploaded image: {}", error),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )
                        .into_response();
                    }
                }

                // TODO: Implement black and white conversion
            }
            _ => {}
        };
    }

    if let (Some(title), Some(artist)) = (title, artist) {
        let background = if dark { "#000000" } else { "#FFFFFF" };

        // TODO: Process image to black and white

        let thumbnail = load_from_memory(&bitmap)
            .unwrap()
            .resize(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Lanczos3)
            .to_rgb8();

        let mut thumbnail_bytes: Vec<u8> = Vec::new();
        DynamicImage::ImageRgb8(thumbnail)
            .write_to(&mut Cursor::new(&mut thumbnail_bytes), ImageFormat::Bmp)
            .unwrap();

        db::create_image(
            &state.pool,
            &title,
            &artist,
            &background,
            bitmap,
            thumbnail_bytes,
        )
        .await;

        Redirect::to("/").into_response()
    } else {
        utils::handle_error(
            anyhow!("Failed to parse image upload form"),
            StatusCode::BAD_REQUEST,
        )
        .into_response()
    }
}
