use sqlx::FromRow;

pub type Identifier = i64;

#[derive(FromRow, Clone, Debug)]
pub struct Image {
    pub id: Identifier,
    pub title: String,
    pub artist: Option<String>,
}
