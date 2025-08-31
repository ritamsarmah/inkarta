pub mod model;
pub mod repository;

use std::io::Cursor;

use maud::{DOCTYPE, Markup, html};
use model::Image;
use serde::Deserialize;

use anyhow::Result;
use axum::{
    Router,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{
        Response, StatusCode,
        header::{CONTENT_LENGTH, CONTENT_TYPE},
    },
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use chrono::{Duration, prelude::*};
use image::{
    DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Luma, imageops, load_from_memory,
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    pub pool: SqlitePool,
    pub current_id: Option<i64>,
    pub next_id: Option<i64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Database
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let pool = SqlitePool::connect(&database_url).await?;

    // Application
    let state = AppState {
        pool,
        current_id: None,
        next_id: None,
    };
    let app = Router::new()
        .route("/", get(home_page))
        // .route("/upload", get(upload_page))
        // .route("/view/{id}", get(view_page))
        .route("/device/rtc", get(device_rtc))
        .route("/device/alarm", get(device_alarm))
        .route("/image/{id}", get(get_image))
        .route("/image/next", get(get_next_image))
        .route("/image/next/{id}", put(set_next_image))
        .route("/image", post(create_image))
        .route("/image/{id}", delete(delete_image))
        .with_state(state);

    let host = std::env::var("HOST").unwrap();
    let port = std::env::var("PORT").unwrap();
    let addr = format!("{host}:{port}");

    info!("Running on {addr}");

    let listener = TcpListener::bind(addr).await.unwrap();
    Ok(axum::serve(listener, app).await?)
}

/* Pages */

async fn home_page(State(state): State<AppState>) -> Markup {
    async fn get_image_title(pool: &SqlitePool, id: Option<i64>) -> String {
        let Some(id) = id else {
            return "None".into();
        };

        match repository::get_image(pool, id).await {
            Ok(image) => image.title,
            Err(_) => "Error".into(),
        }
    }

    let current_image_title = get_image_title(&state.pool, state.current_id).await;
    let next_image_title = get_image_title(&state.pool, state.next_id).await;
    let images = repository::list_images(&state.pool)
        .await
        .unwrap_or_default();

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                meta name="color-scheme" content="light dark";

                title { "Gallery | Inkarta" }

                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css";
                style {
                    r#"
                        :root {
                            --pico-font-family: var(--pico-font-family-monospace);
                        }

                        header {
                            margin-top: 2rem;
                            display: flex;
                            align-items: center;
                            justify-content: space-between;
                        }
                    "#
                }

                script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js" {}
            }
            body.container {
                header {
                    h1 { "Gallery" }
                    a href="/upload" { "Upload Image" }
                }

                article {
                    strong { "Current Image: " }
                    (current_image_title)
                    br;
                    strong { "Next Image: " }
                    (next_image_title)
                }

                main {
                    table {
                        thead {
                            tr {
                                th { "Title" }
                                th { "Artist" }
                            }
                        }
                        tbody {
                            @for image in images {
                                tr {
                                    td { a href={"/view/" (image.id)} { (image.title) } }
                                    td { (image.artist) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/* Device */

// Returns Unix epoch timestamp in server's timezone for device RTC.
async fn device_rtc() -> String {
    let timestamp = Local::now().timestamp();
    info!("Returning timestamp for real-time clock: {}", timestamp);
    timestamp.to_string()
}

// Returns Unix epoch timestamp for next display refresh (i.e., at midnight in server's timezone)
async fn device_alarm() -> String {
    let now = Local::now();
    let midnight = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap();
    let next_midnight = midnight + Duration::days(1);
    next_midnight.timestamp().to_string()
}

/* Layout */

/* Image */

#[derive(Deserialize)]
struct ImageSize {
    width: Option<u32>,
    height: Option<u32>,
}

async fn get_image(
    Path(id): Path<i64>,
    Query(query): Query<ImageSize>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match repository::get_image(&state.pool, id).await {
        Ok(image) => create_image_response(image, query).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_next_image(
    Query(query): Query<ImageSize>,
    State(mut state): State<AppState>,
) -> impl IntoResponse {
    let image = if let Some(next_id) = state.next_id {
        repository::get_image(&state.pool, next_id).await
    } else {
        repository::get_random_image(&state.pool).await
    };

    match image {
        Ok(image) => {
            state.current_id = Some(image.id);
            state.next_id = repository::get_random_image(&state.pool)
                .await
                .ok()
                .map(|image| image.id);

            create_image_response(image, query).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn set_next_image(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
) -> impl IntoResponse {
    state.next_id = Some(id);
    info!("Set next image ID: {id}");
    StatusCode::OK
}

async fn create_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut title = None;
    let mut artist = None;
    let mut dark = false;
    let mut image = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "title" => title = field.text().await.ok(),
            "artist" => artist = field.text().await.ok(),
            "dark" => dark = field.text().await.ok().map_or(false, |value| value == "on"),
            "image" => {
                image = field
                    .bytes()
                    .await
                    .map_err(|err| error!("Failed to read uploaded image: {err}"))
                    .ok()
                    .and_then(|bytes| process_image(&bytes).ok());

                if image.is_none() {
                    return StatusCode::BAD_REQUEST;
                }
            }
            _ => {
                error!("Unexpected form field {name}");
                return StatusCode::BAD_REQUEST;
            }
        }
    }

    if let (Some(title), Some(artist), Some(image)) = (title, artist, image) {
        match repository::create_image(&state.pool, &title, &artist, dark, &image).await {
            Ok(_) => StatusCode::OK,
            Err(err) => {
                error!("Failed to create image: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    } else {
        error!("Missing required fields");
        StatusCode::BAD_REQUEST
    }
}

async fn delete_image(Path(id): Path<i64>, State(state): State<AppState>) -> impl IntoResponse {
    match repository::delete_image(&state.pool, id).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

/* Utilities */

/// Converts an image into a grayscale image.
pub fn process_image(data: &[u8]) -> Result<DynamicImage> {
    let grayscale = load_from_memory(data)?.to_luma8();
    Ok(DynamicImage::ImageLuma8(grayscale))
}

fn create_image_response(image: Image, size: ImageSize) -> impl IntoResponse {
    let full_image = load_from_memory(&image.data).unwrap();
    let (full_width, full_height) = full_image.dimensions();

    let new_width = size.width.unwrap_or(full_width);
    let new_height = size.height.unwrap_or(full_height);

    let mut buffer = Cursor::new(Vec::new());

    if new_width != full_width || new_height != full_height {
        info!(
            "Returning image \"{title}\" resized to {new_width} x {new_height}",
            title = image.title
        );

        // Calculate the scaling factor to maintain aspect ratio
        let scale_x = new_width as f32 / full_width as f32;
        let scale_y = new_height as f32 / full_height as f32;
        let scale = scale_x.min(scale_y);

        // Calculate scaled image dimensions
        let scaled_width = (full_width as f32 * scale) as u32;
        let scaled_height = (full_height as f32 * scale) as u32;

        // Calculate offsets to center the scaled image
        let offset_x = (new_width - scaled_width) / 2;
        let offset_y = (new_height - scaled_height) / 2;

        // Create destination canvas with background fill
        let color = if image.dark { 0 } else { 1 };
        let background = ImageBuffer::from_pixel(new_width, new_height, Luma([color]));
        let mut composite = DynamicImage::ImageLuma8(background).into_luma8();

        // Create resized image
        // NOTE: Do not use Lanczo3 cause it generates image that cannot be processed by Inkplate
        let resized = full_image
            .resize(scaled_width, scaled_height, imageops::FilterType::Triangle)
            .into_luma8();

        // Scale the source image into the destination
        imageops::overlay(&mut composite, &resized, offset_x as i64, offset_y as i64);
        composite.write_to(&mut buffer, ImageFormat::Png).unwrap();
    } else {
        info!(
            "Returning image \"{title}\" at full resolution",
            title = image.title
        );

        full_image.write_to(&mut buffer, ImageFormat::Png).unwrap();
    }

    Response::builder()
        .header(CONTENT_TYPE, "image/png")
        .header(CONTENT_LENGTH, buffer.get_ref().len())
        .body(Body::from(buffer.into_inner()))
        .unwrap()
}
