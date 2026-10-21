use std::sync::Arc;
use sqlx::PgPool;

use crate::{auth::JwtKeys, config::Config};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub jwt_keys: Arc<JwtKeys>,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        let jwt_keys = Arc::new(JwtKeys::new(&config.jwt_secret));
        Self {
            pool,
            config: Arc::new(config),
            jwt_keys,
        }
    }
}
