use std::sync::Mutex;

use anyhow::Result;
use rand::Rng;
use sqlx::{sqlite::SqliteQueryResult, Pool, Sqlite};

use crate::model::{Identifier, Image};

static STORE: Mutex<Vec<Image>> = Mutex::new(Vec::new());

pub async fn initialize(pool: &Pool<Sqlite>) -> Result<SqliteQueryResult> {
    STORE.lock().unwrap().push(Image {
        title: "Sunset Overdrive".into(),
        artist: Some("Alex Johnson".into()),
        id: 1,
    });
    STORE.lock().unwrap().push(Image {
        title: "Ocean's Melody".into(),
        artist: Some("Casey Brown".into()),
        id: 2,
    });
    STORE.lock().unwrap().push(Image {
        title: "Mountain Whisper".into(),
        artist: Some("Dana Lee".into()),
        id: 3,
    });

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

// pub async fn create_image(
//     pool: &Pool<Sqlite>,
//     title: &str,
//     artist: Option<&str>,
//     background: String,
//     data: Vec<u8>,
//     thumbnail: Vec<u8>,
// ) -> Result<Image> {
//     let mut rng = rand::thread_rng();

//     STORE.lock().unwrap().push(Image {
//         title: title.into(),
//         id: rng.gen(),
//     });

//     let artist = artist.unwrap_or("");

//     let query = "
//         insert into images (title, artist, background, data, thumbnail)
//         values (?, ?, ?, ?, ?)
//         returning id, title, artist, background, data, thumbnail
//     ";

//     let row = sqlx::query_as::<_, Image>(query)
//         .bind(title)
//         .bind(artist)
//         .bind(background)
//         .bind(data)
//         .bind(thumbnail)
//         .fetch_one(pool)
//         .await?;

//     Ok(row)
// }

pub async fn create_image(title: &str) {
    let mut rng = rand::thread_rng();
    STORE.lock().unwrap().push(Image {
        title: title.into(),
        artist: Some("Ritam Sarmah".into()),
        id: rng.gen(),
    });
}

pub async fn get_image(pool: &Pool<Sqlite>, id: Identifier) -> Result<Image> {
    // let image = sqlx::query_as("select * from images where id = ?")
    //     .bind(id)
    //     .fetch_one(pool)
    //     .await?;

    let image = STORE
        .lock()
        .unwrap()
        .iter()
        .find(|&image| image.id == id)
        .unwrap()
        .clone();

    Ok(image)
}

pub async fn get_images(pool: &Pool<Sqlite>) -> Result<Vec<Image>> {
    // let images = sqlx::query_as("select * from images")
    //     .fetch_all(pool)
    //     .await?;

    let images = STORE.lock().unwrap().clone();

    Ok(images)
}

// pub async fn update_image(pool: &Pool<Sqlite>, id: Identifier, image: Image) -> Result<Image> {
pub async fn update_image(pool: &Pool<Sqlite>, id: Identifier, image: Image) {
    // let mut original = STORE
    //     .lock()
    //     .unwrap()
    //     .iter()
    //     .find(|&image| image.id == id)
    //     .unwrap();

    // original.title = image.title;
}

pub async fn delete_image(pool: &Pool<Sqlite>, id: Identifier) -> Result<()> {
    STORE.lock().unwrap().retain(|image| image.id != id);

    Ok(())
}
