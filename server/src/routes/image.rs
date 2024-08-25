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
use image::{imageops, load_from_memory, DynamicImage, ImageBuffer, ImageFormat, Luma};
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

/// Get raw image data for specified identifier, optionally resized
async fn get_image(
    Path(id): Path<Identifier>,
    Query(query): Query<ImageSizeParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match db::get_image(&state.pool, id).await {
        Ok(image) => create_image_response(image, query),
        Err(err) => not_found_error(err).into_response(),
    }
}

/// Get raw image data for next image to display, optionally resized
async fn get_next_image(
    Query(query): Query<ImageSizeParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = &state.pool;

    // Retrieve next image (or random image ID if no next value set)
    let next_id = match db::get_next_id(pool).await {
        Some(id) => Some(id),
        None => {
            debug!("No next ID set. Selecting random ID");
            db::get_random_id(pool).await
        }
    };

    debug!("Next image ID: {next_id:?}");

    if let Some(id) = next_id {
        match db::get_image(pool, id).await {
            Ok(next_image) => {
                // Update current image ID and set a new random ID
                if let Err(err) = db::set_current_id(pool, id).await {
                    error!("Failed to update current ID: {err}");
                }

                if let Err(err) = db::set_random_next_id(pool).await {
                    error!("Failed to update random next ID: {err}");
                }

                create_image_response(next_image, query)
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

/// Create image
async fn create_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut title = None;
    let mut artist = None;
    let mut dark = false;
    let mut image: Vec<u8> = Vec::new();
    let mut thumbnail: Vec<u8> = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "title" => title = field.text().await.ok(),
            "artist" => artist = field.text().await.ok(),
            "dark" => dark = field.text().await.ok().map_or(false, |value| value == "on"),
            "image" => {
                let data = field.bytes().await.unwrap();
                match process_image(&data, &mut image, &mut thumbnail) {
                    Ok(_) => debug!("Processed uploaded image"),
                    Err(err) => error!("Failed to process image: {err}"),
                };
            }
            _ => {}
        };
    }

    if let (Some(title), Some(artist)) = (title, artist) {
        let background: u8 = if dark { 0 } else { 255 };
        match db::create_image(&state.pool, &title, &artist, background, image, thumbnail).await {
            Ok(_) => debug!("Created new image"),
            Err(err) => error!("Failed to create image: {err}"),
        }

        let mut headers = HeaderMap::new();
        headers.insert("HX-Refresh", HeaderValue::from_static("true"));
        (StatusCode::OK, headers).into_response()
    } else {
        (StatusCode::BAD_REQUEST, "Failed to upload image".to_owned()).into_response()
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

/* Utilities */

fn process_image(
    data: &Bytes,
    image_buffer: &mut Vec<u8>,
    thumbnail_buffer: &mut Vec<u8>,
) -> Result<()> {
    // Create main image
    let image = load_from_memory(data)
        .context("Failed to load image data")?
        .grayscale()
        .into_luma8();

    let mut cursor = Cursor::new(image_buffer);
    image.write_to(&mut cursor, ImageFormat::Png)?;

    // Create thumbnail image
    let thumbnail = DynamicImage::ImageLuma8(image).thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);

    let mut cursor = Cursor::new(thumbnail_buffer);
    thumbnail.write_to(&mut cursor, ImageFormat::Jpeg)?;

    Ok(())
}

fn create_image_response(image: Image, query: ImageSizeParams) -> Response<Body> {
    let width = query.width;
    let height = query.height;

    let mut buffer = Cursor::new(Vec::new());
    let original = load_from_memory(&image.data).unwrap();

    if let (Some(width), Some(height)) = (width, height) {
        debug!(
            "Returning image \"{title}\" resized to {width} x {height}",
            title = image.title
        );

        // NOTE: Do not use Lanczo3 cause it generates image that cannot be processed by Inkplate
        let resized = original
            .resize(width, height, imageops::FilterType::Triangle)
            .into_luma8();

        let background = ImageBuffer::from_pixel(width, height, Luma([image.background]));
        let mut composite = DynamicImage::ImageLuma8(background).into_luma8();

        let (new_width, new_height) = resized.dimensions();
        let x_offset = (width - new_width) / 2;
        let y_offset = (height - new_height) / 2;

        imageops::overlay(&mut composite, &resized, x_offset as i64, y_offset as i64);
        composite.write_to(&mut buffer, ImageFormat::Png).unwrap();
    } else {
        debug!(
            "Returning image \"{title}\" at full resolution",
            title = image.title
        );

        original.write_to(&mut buffer, ImageFormat::Png).unwrap();
    }

    Response::builder()
        .header(header::CONTENT_TYPE, "image/png")
        .header(header::CONTENT_LENGTH, buffer.get_ref().len())
        .body(Body::from(buffer.into_inner()))
        .unwrap()
}
