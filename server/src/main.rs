pub mod model;
pub mod repository;

use std::{io::Cursor, path::PathBuf, sync::Arc};

use minijinja::{Environment, context, path_loader};
use minijinja_autoreload::AutoReloader;
use model::Image;
use serde::Deserialize;

use anyhow::{Context, Result, bail};
use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{
        Response, StatusCode,
        header::{CONTENT_LENGTH, CONTENT_TYPE},
    },
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
};
use chrono::{Duration, prelude::*};
use image::{
    DynamicImage, GenericImageView, ImageBuffer, ImageFormat, ImageReader, Luma, imageops,
    load_from_memory,
};
use sqlx::SqlitePool;
use tokio::{net::TcpListener, sync::Mutex};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

const IMAGE_UPLOAD_MAX_BYTES: usize = 32 * 1024 * 1024; // 32 MB

#[derive(Clone)]
struct AppState {
    pub pool: SqlitePool,
    pub templates: Arc<AutoReloader>,
    pub current_id: Arc<Mutex<Option<i64>>>,
    pub next_id: Arc<Mutex<Option<i64>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Database

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = SqlitePool::connect(&database_url).await?;

    // Templates

    let enable_autoreload = std::env::var("ENABLE_AUTORELOAD").is_ok();
    debug!(
        "Template auto-reloading is {}",
        if enable_autoreload {
            "enabled"
        } else {
            "disabled"
        }
    );

    let reloader = AutoReloader::new(move |notifier| {
        let mut env = Environment::new();
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        if enable_autoreload {
            env.set_loader(path_loader(&template_path));
            notifier.watch_path(&template_path, true);
            notifier.set_fast_reload(true);
        } else {
            // Inline templates to avoid copying templates when deploying
            env.add_template("index.jinja", include_str!("../templates/index.jinja"))?;
            env.add_template("upload.jinja", include_str!("../templates/upload.jinja"))?;
            env.add_template("view.jinja", include_str!("../templates/view.jinja"))?;
        }

        Ok(env)
    });

    // Application

    let state = AppState {
        pool,
        templates: Arc::new(reloader),
        current_id: Arc::new(Mutex::new(None)),
        next_id: Arc::new(Mutex::new(None)),
    };
    let app = Router::new()
        .route("/", get(home_page))
        .route("/ui/upload", get(upload_modal))
        .route("/ui/view/{id}", get(view_modal))
        .route("/device/rtc", get(device_rtc))
        .route("/device/alarm", get(device_alarm))
        .route("/image/{id}", get(get_image))
        .route("/image/next", get(get_next_image))
        .route("/image/next/{id}", put(set_next_image))
        .route("/image", post(create_image))
        .route("/image/{id}", delete(delete_image))
        .layer(DefaultBodyLimit::max(IMAGE_UPLOAD_MAX_BYTES))
        .with_state(state);

    let host = std::env::var("HOST")?;
    let port = std::env::var("PORT")?;
    let addr = format!("{host}:{port}");

    info!("Running on {addr}");

    let listener = TcpListener::bind(addr).await?;

    Ok(axum::serve(listener, app).await?)
}

/* Views */

async fn home_page(State(state): State<AppState>) -> Html<String> {
    async fn get_image_title(pool: &SqlitePool, id: Option<i64>) -> String {
        let Some(id) = id else {
            return "None".into();
        };

        match repository::get_image_detail(pool, id).await {
            Ok(detail) => detail.title,
            Err(err) => {
                error!("Failed to load image detail: {err}");
                err.to_string()
            }
        }
    }

    let current_id = *state.current_id.lock().await;
    let next_id = *state.next_id.lock().await;

    let current_title = get_image_title(&state.pool, current_id).await;
    let next_title = get_image_title(&state.pool, next_id).await;
    let details = repository::list_image_details(&state.pool)
        .await
        .unwrap_or_default();

    let context = context! {
        current_title => current_title,
        next_title => next_title,
        images => details,
    };

    let templates = state.templates.acquire_env().unwrap();
    let template = templates.get_template("index.jinja").unwrap();
    let rendered = template.render(context).unwrap();

    Html(rendered)
}

async fn upload_modal(State(state): State<AppState>) -> Html<String> {
    let templates = state.templates.acquire_env().unwrap();
    let template = templates.get_template("upload.jinja").unwrap();
    let rendered = template.render({}).unwrap();

    Html(rendered)
}

async fn view_modal(Path(id): Path<i64>, State(state): State<AppState>) -> impl IntoResponse {
    let Ok(detail) = repository::get_image_detail(&state.pool, id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let next_id = *state.next_id.lock().await;

    let context = context! {
        image => detail,
        next_id => next_id,
    };

    let templates = state.templates.acquire_env().unwrap();
    let template = templates.get_template("view.jinja").unwrap();
    let rendered = template.render(context).unwrap();

    Html(rendered).into_response()
}

/* Device */

// Returns Unix epoch timestamp in server's timezone for device RTC.
async fn device_rtc() -> String {
    let timestamp = Local::now().timestamp();
    info!("Returning timestamp for real-time clock: {timestamp}");
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
        Err(err) => {
            error!("Failed to get image {id}: {err}");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

async fn get_next_image(
    Query(query): Query<ImageSize>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut current_id = state.current_id.lock().await;
    let mut next_id = state.next_id.lock().await;

    let image = if let Some(next_id) = *next_id {
        repository::get_image(&state.pool, next_id).await
    } else {
        repository::get_random_image(&state.pool).await
    };

    match image {
        Ok(image) => {
            *current_id = Some(image.id);
            *next_id = repository::get_random_image(&state.pool)
                .await
                .ok()
                .map(|image| image.id);

            create_image_response(image, query).into_response()
        }
        Err(err) => {
            error!("Failed to get next image: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn set_next_image(Path(id): Path<i64>, State(state): State<AppState>) -> impl IntoResponse {
    *state.next_id.lock().await = Some(id);
    info!("Set next image ID: {id}");
    create_refresh_response(StatusCode::OK)
}

async fn create_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut title = String::new();
    let mut artist = String::new();
    let mut dark = false;
    let mut data = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or_default();
        match name {
            "title" => title = field.text().await.unwrap_or_default(),
            "artist" => artist = field.text().await.unwrap_or_default(),
            "dark" => dark = field.text().await.unwrap_or_default() == "on",
            "image" => {
                let bytes = field.bytes().await.unwrap_or_default();
                if bytes.is_empty() {
                    error!("Image was empty");
                    return StatusCode::BAD_REQUEST.into_response();
                }
                data.extend_from_slice(&bytes);
            }
            _ => {}
        }
    }

    let bitmap = match process_image(&data) {
        Ok(bitmap) => bitmap,
        Err(err) => {
            error!("Failed to process image into bitmap: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(err) = repository::create_image(&state.pool, &title, &artist, dark, &bitmap).await {
        error!("Failed to create image: {err}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    info!("Created new image: {title} by {artist}");
    create_refresh_response(StatusCode::OK).into_response()
}

async fn delete_image(Path(id): Path<i64>, State(state): State<AppState>) -> impl IntoResponse {
    let status = match repository::delete_image(&state.pool, id).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            error!("Failed to delete image: {err}");
            StatusCode::NOT_FOUND
        }
    };

    create_refresh_response(status)
}

/* Utilities */

/// Converts an image into a grayscale image.
pub fn process_image(data: &[u8]) -> Result<DynamicImage> {
    if data.is_empty() {
        bail!("No image data uploaded")
    }

    let cursor = std::io::Cursor::new(data);
    let reader = ImageReader::new(cursor)
        .with_guessed_format()
        .context("Cannot determine image format")?;

    Ok(reader
        .decode()
        .context("Failed to decode image")?
        .grayscale())
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
        let color = if image.dark { 0 } else { 255 };
        let background = ImageBuffer::from_pixel(new_width, new_height, Luma([color]));
        let mut composite = DynamicImage::ImageLuma8(background).into_luma8();

        // Create resized image
        // NOTE: Do not use Lanczo3 cause it generates image that cannot be processed by Inkplate
        let resized = full_image
            .resize(scaled_width, scaled_height, imageops::FilterType::Triangle)
            .into_luma8();

        // Scale the source image into the destination
        imageops::overlay(&mut composite, &resized, offset_x as i64, offset_y as i64);
        composite.write_to(&mut buffer, ImageFormat::Bmp).unwrap();
    } else {
        info!(
            "Returning image \"{title}\" at full resolution",
            title = image.title
        );

        full_image.write_to(&mut buffer, ImageFormat::Bmp).unwrap();
    }

    Response::builder()
        .header(CONTENT_TYPE, "image/bmp")
        .header(CONTENT_LENGTH, buffer.get_ref().len())
        .body(Body::from(buffer.into_inner()))
        .unwrap()
}

fn create_refresh_response(status: StatusCode) -> impl IntoResponse {
    (status, [("HX-Refresh", "true")])
}
