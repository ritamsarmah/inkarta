use anyhow::Context;
use std::io::Cursor;

use anyhow::Result;
use image::{DynamicImage, ImageFormat};
use sqlx::SqlitePool;

use crate::model::Image;

pub async fn get_image(pool: &SqlitePool, id: i64) -> Result<Image> {
    sqlx::query_as!(Image, r#"select * from images where id = ?"#, id)
        .fetch_one(pool)
        .await
        .context("Failed to get image")
}

pub async fn get_random_image(pool: &SqlitePool) -> Result<Image> {
    sqlx::query_as!(Image, r#"select * from images order by random() limit 1"#)
        .fetch_one(pool)
        .await
        .context("Failed to get random image")
}

pub async fn list_images(pool: &SqlitePool) -> Result<Vec<Image>> {
    sqlx::query_as!(Image, r#"select * from images"#)
        .fetch_all(pool)
        .await
        .context("Failed to list images")
}

pub async fn create_image(
    pool: &SqlitePool,
    title: &str,
    artist: &str,
    dark: bool,
    image: &DynamicImage,
) -> Result<i64> {
    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, ImageFormat::Bmp)?;
    let data = buffer.into_inner();

    let record = sqlx::query!(
        r#"
            insert into images (title, artist, dark, data)
            values (?, ?, ?, ?)
            returning id
        "#,
        title,
        artist,
        dark,
        data,
    )
    .fetch_one(pool)
    .await
    .context("Failed to create image")?;

    Ok(record.id)
}

pub async fn delete_image(pool: &SqlitePool, id: i64) -> Result<u64> {
    let result = sqlx::query!(r#"delete from images where id = ?"#, id)
        .execute(pool)
        .await
        .context("Failed to delete image")?;

    Ok(result.rows_affected())
}
