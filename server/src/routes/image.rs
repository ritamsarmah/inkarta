use std::io::Cursor;

use anyhow::{Context, Result};
use axum::{
    body::{Body, Bytes},
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use image::{
    imageops::*, load_from_memory, DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Luma,
};
use serde::Deserialize;
use tracing::{debug, error};

use crate::{
    db,
    model::{Identifier, Image},
    state::AppState,
    utils::{not_found_error, server_error},
};

const THUMBNAIL_SIZE: u32 = 512;

#[derive(Deserialize)]
struct ImageSizeParams {
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

/// Get raw image data scaled to an optional height and width
async fn get_image(
    Path(id): Path<Identifier>,
    Query(query): Query<ImageSizeParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match db::get_image(&state.pool, id).await {
        Ok(image) => {
            let buffer = resize_into_bitmap(image, query.width, query.height);
            Response::builder()
                .header(header::CONTENT_TYPE, "image/bmp")
                .body(Body::from(buffer.into_inner()))
                .unwrap()
        }
        Err(err) => not_found_error(err).into_response(),
    }
}

/// Delete image with specified identifier
async fn delete_image(
    Path(id): Path<Identifier>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match db::delete_image(&state.pool, id).await {
        Ok(_) => {
            debug!("Successfully deleted image with id: {id}");

            let mut headers = HeaderMap::new();
            headers.insert("HX-Refresh", HeaderValue::from_static("true"));
            (StatusCode::OK, headers).into_response()
        }
        Err(err) => server_error(err).into_response(),
    }
}

/// Get next image for display
async fn get_next_image(
    Query(query): Query<ImageSizeParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = &state.pool;

    // Retrieve next image (or random image ID if no next value set)
    let next_id = match db::get_next_id(pool).await {
        Some(id) => Some(id),
        None => db::get_random_id(pool).await,
    };

    if let Some(id) = next_id {
        match db::get_image(pool, id).await {
            Ok(next_image) => {
                // Update current image ID and set a new random ID
                match db::set_current_id(pool).await {
                    Ok(_) => {}
                    Err(e) => error!("Failed to set current ID: {}", e),
                }

                match db::set_random_next_id(pool).await {
                    Ok(_) => {}
                    Err(e) => error!("Failed to set random next ID: {}", e),
                }

                let buffer = resize_into_bitmap(next_image, query.width, query.height);
                Response::builder()
                    .header(header::CONTENT_TYPE, "image/bmp")
                    .body(Body::from(buffer.into_inner()))
                    .unwrap()
            }
            Err(err) => server_error(err).into_response(),
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            "No images available in the database".to_owned(),
        )
            .into_response()
    }
}

async fn set_next_id(
    Path(id): Path<Identifier>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match db::set_next_id(&state.pool, id).await {
        Ok(_) => "<button class=\"btn\" disabled>Selected</button".into_response(),
        Err(err) => server_error(err).into_response(),
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
                let data = field.bytes().await.unwrap();
                match process_image(&data, &mut bitmap, &mut thumbnail) {
                    Ok(_) => debug!("Processed uploaded image"),
                    Err(err) => error!("Failed to process image: {err}"),
                };
            }
            _ => {}
        };
    }

    // Process image in the background and return immediately
    if let (Some(title), Some(artist)) = (title, artist) {
        tokio::spawn(async move {
            let background: u8 = if dark { 0 } else { 255 };
            match db::create_image(&state.pool, &title, &artist, background, bitmap, thumbnail)
                .await
            {
                Ok(_) => debug!("Created new image"),
                Err(err) => error!("Failed to create image: {err}"),
            }
        });

        let mut headers = HeaderMap::new();
        headers.insert("HX-Refresh", HeaderValue::from_static("true"));
        (StatusCode::OK, headers).into_response()
    } else {
        (StatusCode::BAD_REQUEST, "Failed to upload image".to_owned()).into_response()
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

fn resize_into_bitmap(image: Image, width: Option<u32>, height: Option<u32>) -> Cursor<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());
    let bmp = load_from_memory(&image.data).unwrap();

    if let (Some(width), Some(height)) = (width, height) {
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

    buffer
}
