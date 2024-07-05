use sqlx::FromRow;

pub type Identifier = i64;

#[derive(FromRow, Clone, Debug)]
pub struct Image {
    /// Unique identifier for the artwork
    pub id: Identifier,
    /// Title of the image
    pub title: String,
    /// Name of the image's artist
    pub artist: Option<String>,
    /// Background color when fitting image to dimensions
    pub background: String,
    /// Binary data for the full image
    pub data: Vec<u8>,
    /// Binary data for the thumbnail
    pub thumbnail: Vec<u8>,
}
