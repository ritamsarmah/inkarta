use anyhow::Result;
use sqlx::{sqlite::SqliteQueryResult, Pool, Sqlite};

use crate::model::{Identifier, Image, Thumbnail};

pub async fn initialize(pool: &Pool<Sqlite>) -> Result<SqliteQueryResult> {
    let result = sqlx::query(
        "create table if not exists images (
            id integer primary key autoincrement,
            title text not null,
            artist text,
            background text not null,
            data blob not null,
            thumbnail blob not null
        );",
    )
    .execute(pool)
    .await?;

    Ok(result)
}

pub async fn create_image(
    pool: &Pool<Sqlite>,
    title: &str,
    artist: &str,
    background: &str,
    data: Vec<u8>,
    thumbnail: Vec<u8>,
) -> Result<Image> {
    let query = "
        insert into images (title, artist, background, data, thumbnail)
        values (?, ?, ?, ?, ?)
        returning id, title, artist, background, data, thumbnail
    ";

    let row = sqlx::query_as::<_, Image>(query)
        .bind(title.trim())
        .bind(artist.trim())
        .bind(background)
        .bind(data)
        .bind(thumbnail)
        .fetch_one(pool)
        .await?;

    Ok(row)
}

pub async fn get_image(pool: &Pool<Sqlite>, id: Identifier) -> Result<Image> {
    let image = sqlx::query_as("select * from images where id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(image)
}

pub async fn get_thumbnails(pool: &Pool<Sqlite>) -> Result<Vec<Thumbnail>> {
    let thumbnails = sqlx::query_as("select id, title, artist, thumbnail from images")
        .fetch_all(pool)
        .await?;

    Ok(thumbnails)
}

// pub async fn update_image(pool: &Pool<Sqlite>, id: Identifier, image: Image) -> Result<Image> {

pub async fn delete_image(pool: &Pool<Sqlite>, id: Identifier) -> Result<()> {
    sqlx::query("delete from images where id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
