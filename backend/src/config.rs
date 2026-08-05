use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_access_ttl_secs: i64,
    pub jwt_refresh_ttl_secs: i64,
    pub aes_key_base64: Option<String>,
    pub ai_api_key: Option<String>,
    pub ai_base_url: String,
    pub ai_model: String,
    pub rate_limit_rps: u32,
    pub cors_origins: Vec<String>,
    pub app_env: String,
    pub firebase_project_id: Option<String>,
    pub google_client_ids: Vec<String>,
    pub ipo_sync_interval_secs: u64,
    pub ipo_list_cache_ttl_secs: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".into());
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            if app_env == "production" {
                String::new()
            } else {
                "dev-only-change-me-to-a-long-secret-key!!".into()
            }
        });

        if app_env == "production"
            && (jwt_secret.len() < 32 || jwt_secret.contains("dev-only"))
        {
            anyhow::bail!("JWT_SECRET must be set to a strong value (≥32 chars) in production");
        }

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "*".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://investiq:investiq@localhost:5432/investiq".into()),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            jwt_secret,
            jwt_access_ttl_secs: env::var("JWT_ACCESS_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(900),
            jwt_refresh_ttl_secs: env::var("JWT_REFRESH_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2_592_000),
            aes_key_base64: env::var("AES_KEY_BASE64").ok(),
            ai_api_key: env::var("AI_API_KEY").ok(),
            ai_base_url: env::var("AI_BASE_URL")
                .unwrap_or_else(|_| "https://api.x.ai/v1".into()),
            ai_model: env::var("AI_MODEL").unwrap_or_else(|_| "grok-2-latest".into()),
            rate_limit_rps: env::var("RATE_LIMIT_RPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            cors_origins,
            app_env,
            firebase_project_id: env::var("FIREBASE_PROJECT_ID")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            google_client_ids: env::var("GOOGLE_CLIENT_IDS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            ipo_sync_interval_secs: env::var("IPO_SYNC_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1800),
            ipo_list_cache_ttl_secs: env::var("IPO_LIST_CACHE_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
        })
    }

    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }
}
