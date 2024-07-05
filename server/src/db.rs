use anyhow::Result;
use sqlx::{sqlite::SqliteQueryResult, Pool, Sqlite};

use crate::model::{Identifier, Image};

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
    artist: Option<&str>,
    background: String,
    data: Vec<u8>,
    thumbnail: Vec<u8>,
) -> Result<Image> {
    let artist = artist.unwrap_or("");

    let query = "
        insert into images (title, artist, background, data, thumbnail)
        values (?, ?, ?, ?, ?)
        returning id, title, artist, background, data, thumbnail
    ";

    let row = sqlx::query_as::<_, Image>(query)
        .bind(title)
        .bind(artist)
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

pub async fn get_images(pool: &Pool<Sqlite>) -> Result<Vec<Image>> {
    let images = sqlx::query_as("select * from images")
        .fetch_all(pool)
        .await?;

    Ok(images)
}

// pub async fn update_image(pool: &Pool<Sqlite>, id: Identifier, image: Image) -> Result<Image> {
//     todo!()
// }

// pub async fn delete_image(pool: &Pool<Sqlite>, id: Identifier) -> Result<()> {
//     todo!()
// }
