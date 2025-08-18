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
    routing::{get, put},
};
use chrono::{Duration, prelude::*};
use image::{DynamicImage, ImageBuffer, ImageFormat, Luma, imageops, load_from_memory};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tracing::info;
use tracing::{debug, error};

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
        // .route("/upload", get(upload_form))
        // .route("/view/{id}", get(image_modal))
        .route("/device/rtc", get(device_rtc))
        .route("/device/alarm", get(device_alarm))
        .route("/image/{id}", get(get_image))
        .route("/image/next", get(get_next_image))
        .route("/image/next/{id}", put(set_next_image))
        // .route("/image", post(create_image))
        // .route("/image/:id", delete(delete_image))
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
    .into_response()
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
    (StatusCode::OK, [("HX-Refresh", "true")])
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

async fn delete_image(State(state): State<AppState>, id: i64) {
    todo!()
}

/* Utilities */

/// Converts an image into a grayscale image.
pub fn process_image(data: &[u8]) -> Result<DynamicImage> {
    let grayscale = image::load_from_memory(data)?.to_luma8();
    Ok(DynamicImage::ImageLuma8(grayscale))
}

fn create_image_response(image: Image, query: ImageSize) -> impl IntoResponse {
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

        let color = if image.dark { 0 } else { 1 };
        let background = ImageBuffer::from_pixel(width, height, Luma([color]));
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
        .header(CONTENT_TYPE, "image/png")
        .header(CONTENT_LENGTH, buffer.get_ref().len())
        .body(Body::from(buffer.into_inner()))
        .unwrap()
}
