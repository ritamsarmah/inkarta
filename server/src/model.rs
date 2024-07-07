use base64::prelude::*;
use sqlx::FromRow;

pub type Identifier = i64;

#[derive(FromRow, Clone, Debug)]
pub struct Image {
    pub id: Identifier,
    pub title: String,
    pub artist: String,
    pub background: String,
    pub data: Vec<u8>,
}

impl Image {
    /// Returns base64 encoded src for use by img elements
    pub fn src(&self) -> String {
        format!(
            "data:image/bmp;base64,{}",
            BASE64_STANDARD.encode(&self.data)
        )
    }
}
