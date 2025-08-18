use sqlx::FromRow;

#[derive(FromRow)]
pub struct Image {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub dark: bool,
    pub data: Vec<u8>,
}
