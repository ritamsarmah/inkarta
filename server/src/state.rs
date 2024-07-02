use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct AppState {
    pub current: Option<i64>,
    pub next: Option<i64>,
    pub pool: Pool<Sqlite>,
}
