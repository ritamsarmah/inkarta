use anyhow::Result;
use sqlx::{Pool, Sqlite};

use crate::model::{Identifier, Image};

pub async fn create_image(pool: &Pool<Sqlite>, image: Image) -> Result<Image> {
    todo!()
}

pub async fn get_image(pool: &Pool<Sqlite>, id: Identifier) -> Result<Image> {
    todo!()
}

pub async fn get_images(pool: &Pool<Sqlite>) -> Result<Vec<Image>> {
    todo!()
}

pub async fn update_image(pool: &Pool<Sqlite>, id: Identifier, image: Image) -> Result<Image> {
    todo!()
}

pub async fn delete_image(pool: &Pool<Sqlite>, id: Identifier) -> Result<()> {
    todo!()
}
