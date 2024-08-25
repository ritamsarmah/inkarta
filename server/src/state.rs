use std::sync::Arc;

use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct AppState {
    pub templates: Arc<minijinja::Environment<'static>>,
    pub pool: Pool<Sqlite>,
}
