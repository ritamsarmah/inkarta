use anyhow::Result;
use sqlx::{sqlite::SqliteQueryResult, Pool, Sqlite};

use crate::model::{Frame, Identifier, Image, Thumbnail};

pub async fn initialize(pool: &Pool<Sqlite>) -> Result<SqliteQueryResult> {
    let result = sqlx::query(
        "create table if not exists images (
            id integer primary key autoincrement,
            title text not null,
            artist text,
            background integer not null,
            data blob not null,
            thumbnail blob not null
        );

        create table if not exists frame (
            name text not null,
            next integer,
            current integer,
            foreign key(next) references images(id),
            foreign key(current) references images(id)
        );
        ",
    )
    .execute(pool)
    .await?;

    Ok(result)
}

/* Images */

pub async fn create_image(
    pool: &Pool<Sqlite>,
    title: &str,
    artist: &str,
    background: u8,
    data: Vec<u8>,
    thumbnail: Vec<u8>,
) -> Result<()> {
    let query = "
        insert into images (title, artist, background, data, thumbnail)
        values (?, ?, ?, ?, ?)
    ";

    sqlx::query(query)
        .bind(title.trim())
        .bind(artist.trim())
        .bind(background)
        .bind(data)
        .bind(thumbnail)
        .execute(pool)
        .await?;

    Ok(())
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

pub async fn delete_image(pool: &Pool<Sqlite>, id: Identifier) -> Result<()> {
    sqlx::query("delete from images where id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/* Frame */

pub async fn register_frame(pool: &Pool<Sqlite>, name: &str) -> Result<()> {
    let query = "
        insert or replace into frame (name)
        values (?)
    ";

    sqlx::query(query).bind(name).execute(pool).await?;

    Ok(())
}

pub async fn get_frame(pool: &Pool<Sqlite>) -> Option<Frame> {
    sqlx::query_as("select * from frame")
        .fetch_one(pool)
        .await
        .ok()
}

pub async fn update_next_id(pool: &Pool<Sqlite>, id: Identifier) -> Result<()> {
    sqlx::query("update frame set next = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
