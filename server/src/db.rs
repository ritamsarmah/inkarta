use anyhow::Result;
use sqlx::{sqlite::SqliteQueryResult, Pool, Sqlite};

use crate::model::{Identifier, Image, Thumbnail};

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

        create table if not exists device (
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

pub async fn get_image(pool: &Pool<Sqlite>, id: Identifier) -> Option<Image> {
    sqlx::query_as("select * from images where id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .ok()
}

pub async fn get_random_id(pool: &Pool<Sqlite>) -> Option<Identifier> {
    sqlx::query_scalar("select id from images order by random() limit 1")
        .fetch_one(pool)
        .await
        .ok()
}

pub async fn get_thumbnails(pool: &Pool<Sqlite>) -> Option<Vec<Thumbnail>> {
    sqlx::query_as("select id, title, artist, thumbnail from images")
        .fetch_all(pool)
        .await
        .ok()
}

pub async fn delete_image(pool: &Pool<Sqlite>, id: Identifier) -> Result<()> {
    sqlx::query("delete from images where id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/* Device */

/// Get the next image ID for display
pub async fn get_next_id(pool: &Pool<Sqlite>) -> Option<Identifier> {
    sqlx::query_scalar("select next from device limit 1")
        .fetch_one(pool)
        .await
        .ok()
}

/// Set the next image ID for display to the specified ID
pub async fn set_next_id(pool: &Pool<Sqlite>, id: Identifier) -> Result<()> {
    sqlx::query("update device set next = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Set the next image ID to a random ID
pub async fn set_random_next_id(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(
        "
        update device
        set next = (
            select id from images
            order by random()
            limit 1
        )
        ",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Sets the current display image ID to the next ID, and updates the next ID.
pub async fn set_current(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query("update device set current = next")
        .execute(pool)
        .await?;

    // Update next to a new random ID
    set_random_next_id(pool).await?;

    Ok(())
}
