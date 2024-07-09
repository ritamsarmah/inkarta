use std::sync::Arc;

use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct AppState {
    // pub templates: Arc<minijinja::Environment<'static>>,
    pub reloader: Arc<minijinja_autoreload::AutoReloader>,
    pub pool: Pool<Sqlite>,
}
