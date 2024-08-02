use std::io::Cursor;

use anyhow::{anyhow, Context, Result};
use axum::{
    body::{Body, Bytes},
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post, put},
    Router,
};
use image::{
    imageops::*, load_from_memory, DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Luma,
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    db::{self},
    model::Identifier,
    state::AppState,
    utils,
};

const THUMBNAIL_SIZE: u32 = 512;

#[derive(Deserialize)]
struct FetchImageParams {
    width: Option<u32>,
    height: Option<u32>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/image", post(create_image))
        .route("/image/:id", get(get_image).delete(delete_image))
        .route("/image/next", get(get_next_image))
        .route("/image/next/:id", put(set_next_id))
}

/// Gets raw image data scaled to an optional height and width
async fn get_image(
    Path(id): Path<Identifier>,
    Query(query): Query<FetchImageParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match db::get_image(&state.pool, id).await {
        Ok(image) => {
            let mut buffer = Cursor::new(Vec::new());
            let bmp = load_from_memory(&image.data).unwrap();

            if let (Some(width), Some(height)) = (query.width, query.height) {
                debug!("Returning image resized to {width} x {height}");

                let resized = bmp.resize(width, height, FilterType::Lanczos3);

                let background = ImageBuffer::from_pixel(width, height, Luma([image.background]));
                let mut composite = DynamicImage::ImageLuma8(background);

                let (new_width, new_height) = resized.dimensions();
                let x_offset = (width - new_width) / 2;
                let y_offset = (height - new_height) / 2;

                overlay(&mut composite, &resized, x_offset as i64, y_offset as i64);
                composite.write_to(&mut buffer, ImageFormat::Bmp).unwrap();
            } else {
                debug!("Returning full-sized image");
                bmp.write_to(&mut buffer, ImageFormat::Bmp).unwrap();
            }

            Response::builder()
                .header(header::CONTENT_TYPE, "image/bmp")
                .body(Body::from(buffer.into_inner()))
                .unwrap()
        }
        Err(err) => utils::redirect_error(err, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

/// Deletes image with specified identifier
async fn delete_image(
    Path(id): Path<Identifier>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = state.pool;

    if let Err(err) = db::delete_image(&pool, id).await {
        debug!("Failed to delete image with id: {id}");
        utils::redirect_error(err, StatusCode::INTERNAL_SERVER_ERROR).into_response()
    } else {
        debug!("Successfully delete image with id: {id}");
        // After deletion, check if the next image was set to the deleted ID and update
        if let Some(next_id) = db::get_next_id(&pool).await {
            if next_id == id {
                let _ = db::update_random_next_id(&pool).await;
            }
        }

        let mut headers = HeaderMap::new();
        headers.insert("HX-Refresh", HeaderValue::from_static("true"));
        (StatusCode::OK, headers).into_response()
    }
}

/// Get next image for display
async fn get_next_image(
    Query(query): Query<FetchImageParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    fn handle_error(err: anyhow::Error) -> Redirect {
        utils::redirect_error(err, StatusCode::INTERNAL_SERVER_ERROR)
    }

    if let Some(next_id) = db::get_next_id(&state.pool).await {
        // Update the current and next id in database
        match db::set_current_to_next(&state.pool).await {
            Ok(_) => {
                // Return the next image
                get_image(Path(next_id), Query(query), State(state))
                    .await
                    .into_response()
            }
            Err(err) => handle_error(err).into_response(),
        }
    } else {
        // No next ID set for frame, retrieve a random next ID
        match db::get_random_id(&state.pool).await {
            Ok(next_id) => {
                // Images exist, but were not set for frame (or frame not registered)
                match db::update_next_id(&state.pool, next_id).await {
                    Ok(_) => {
                        // Return the next image
                        get_image(Path(next_id), Query(query), State(state))
                            .await
                            .into_response()
                    }
                    Err(err) => handle_error(err).into_response(),
                }
            }
            Err(err) => handle_error(err).into_response(),
        }
    }
}

async fn set_next_id(
    Path(id): Path<Identifier>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match db::update_next_id(&state.pool, id).await {
        Ok(_) => "<button disabled>Selected</button>".into_response(),
        Err(err) => utils::redirect_error(err, StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn create_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut title = None;
    let mut artist = None;
    let mut dark = false;
    let mut bitmap: Vec<u8> = Vec::new();
    let mut thumbnail: Vec<u8> = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "title" => title = field.text().await.ok(),
            "artist" => artist = field.text().await.ok(),
            "dark" => dark = field.text().await.ok().map_or(false, |value| value == "on"),
            "image" => {
                if let Ok(data) = field.bytes().await {
                    match process_image(&data, &mut bitmap, &mut thumbnail) {
                        Ok(_) => debug!("Processed bitmap image successfully"),
                        Err(err) => {
                            return utils::redirect_error(err, StatusCode::INTERNAL_SERVER_ERROR)
                                .into_response()
                        }
                    }
                } else {
                    return utils::redirect_error(
                        anyhow!("No image provided in form data"),
                        StatusCode::BAD_REQUEST,
                    )
                    .into_response();
                }
            }
            _ => {}
        };
    }

    // TODO: Store to database in separate thread, but redirect immediately

    if let (Some(title), Some(artist)) = (title, artist) {
        let background: u8 = if dark { 0 } else { 255 };

        match db::create_image(&state.pool, &title, &artist, background, bitmap, thumbnail).await {
            Ok(_) => debug!("Created new image with title ({title}) and artist ({artist})"),
            Err(err) => {
                return utils::redirect_error(err, StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        };

        let mut headers = HeaderMap::new();
        headers.insert("HX-Refresh", HeaderValue::from_static("true"));
        (StatusCode::OK, headers).into_response()
    } else {
        utils::redirect_error(
            anyhow!("Failed to parse image upload form"),
            StatusCode::BAD_REQUEST,
        )
        .into_response()
    }
}

fn process_image(
    data: &Bytes,
    bmp_buffer: &mut Vec<u8>,
    thumbnail_buffer: &mut Vec<u8>,
) -> Result<()> {
    // Create main bitmap image
    let mut bmp = load_from_memory(data)
        .context("Failed to load image data")?
        .grayscale()
        .to_luma8();

    dither(&mut bmp, &BiLevel);

    let mut cursor = Cursor::new(bmp_buffer);
    bmp.write_to(&mut cursor, ImageFormat::Bmp)?;

    // Create thumbnail image
    let thumbnail = DynamicImage::ImageLuma8(bmp).thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);

    let mut cursor = Cursor::new(thumbnail_buffer);
    thumbnail.write_to(&mut cursor, ImageFormat::Jpeg)?;

    Ok(())
}
