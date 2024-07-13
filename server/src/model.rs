use sqlx::FromRow;

pub type Identifier = i64;

#[derive(FromRow, Clone, Debug)]
pub struct Image {
    pub id: Identifier,
    pub title: String,
    pub artist: String,
    pub background: u8, // luma intensity value
    pub data: Vec<u8>,
    pub thumbnail: Vec<u8>,
}

#[derive(FromRow, Clone, Debug)]
pub struct Thumbnail {
    pub id: Identifier,
    pub title: String,
    pub artist: String,
    pub thumbnail: Vec<u8>,
}
