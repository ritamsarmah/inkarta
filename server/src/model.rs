use sqlx::FromRow;

pub type Identifier = i64;

#[derive(FromRow, Clone, Debug)]
pub struct Image {
    /// Unique identifier for the artwork
    pub id: Identifier,
    /// Title of the image
    pub title: String,
}
