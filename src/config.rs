use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://neondb_owner:npg_xJtM0Y7AFvBw@ep-rough-credit-a1xtzvbz-pooler.ap-southeast-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require".to_string()
        });

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8088);

        let cors_raw = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| {
            "http://localhost:4321,http://localhost:3000,https://supernovalabs.studio".to_string()
        });

        let cors_origins = cors_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            database_url,
            server_host,
            server_port,
            cors_origins,
        }
    }
}
