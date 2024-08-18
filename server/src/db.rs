use anyhow::{Context, Result};
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

        insert into device (next, current) values (null, null);
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
    sqlx::query_as("select * from images where id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .context("Failed to retrieve image with id")
}

pub async fn get_random_id(pool: &Pool<Sqlite>) -> Option<Identifier> {
    sqlx::query_scalar("select id from images order by random() limit 1")
        .fetch_optional(pool)
        .await
        .ok()?
}

pub async fn get_thumbnails(pool: &Pool<Sqlite>) -> Option<Vec<Thumbnail>> {
    sqlx::query_as("select id, title, artist, thumbnail from images")
        .fetch_all(pool)
        .await
        .ok()
}

pub async fn delete_image(pool: &Pool<Sqlite>, id: Identifier) -> Result<()> {
    // Reset the current or next images in the device table if they are set to the ID to delete
    sqlx::query("update device set current = null where current = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("update device set next = null where next = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("delete from images where id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/* Device */

/// Get the next image identifier for display.
pub async fn get_next_id(pool: &Pool<Sqlite>) -> Option<Identifier> {
    sqlx::query_scalar("select next from device limit 1")
        .fetch_optional(pool)
        .await
        .ok()?
}

/// Set the next image identifier for display to the specified identifier.
pub async fn set_next_id(pool: &Pool<Sqlite>, id: Identifier) -> Result<()> {
    sqlx::query("update device set next = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Set the next image identifier to a random identifier.
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

/// Sets the current image identifier to the next, and updates the next identifier.
pub async fn set_current_id(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query("update device set current = next")
        .execute(pool)
        .await?;

    // Update next to a new random identifier
    set_random_next_id(pool).await?;

    Ok(())
}

/// Get the current image title
pub async fn get_current_title(pool: &Pool<Sqlite>) -> Option<String> {
    sqlx::query_scalar(
        "
        select images.title
        from device
        left join images on device.current = images.id
        limit 1
        ",
    )
    .fetch_optional(pool)
    .await
    .ok()?
}

/// Get the next image title
pub async fn get_next_title(pool: &Pool<Sqlite>) -> Option<String> {
    sqlx::query_scalar(
        "
        select images.title
        from device
        left join images on device.next = images.id
        limit 1
        ",
    )
    .fetch_optional(pool)
    .await
    .ok()?
}
