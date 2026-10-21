use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub cors_origins: Vec<String>,
    pub jwt_secret: String,
    pub demo_mode: bool,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in environment or .env file");

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8088);

        let cors_raw = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| {
            "http://localhost:4321,http://127.0.0.1:4321,https://supernovalabs.studio".to_string()
        });

        let cors_origins = cors_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            "supernova_enterprise_cloud_jwt_default_secret_key_2026".to_string()
        });

        let demo_mode = env::var("DEMO_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        Self {
            database_url,
            server_host,
            server_port,
            cors_origins,
            jwt_secret,
            demo_mode,
        }
    }
}
