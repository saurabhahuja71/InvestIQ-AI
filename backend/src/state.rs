use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::Config;
use crate::infra::ai::AiClient;
use crate::infra::crypto::AesCipher;
use crate::infra::id_token::IdTokenVerifier;
use crate::infra::jwt::JwtService;
use crate::infra::nse_ipo::NseIpoClient;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub config: Config,
    pub jwt: JwtService,
    pub ai: AiClient,
    /// Optional field-level encryption when `AES_KEY_BASE64` is configured.
    pub cipher: Option<AesCipher>,
    pub id_tokens: IdTokenVerifier,
    pub nse: NseIpoClient,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(20)
            .connect(&config.database_url)
            .await?;

        let redis_client = redis::Client::open(config.redis_url.as_str())?;
        let redis = ConnectionManager::new(redis_client).await?;

        let jwt = JwtService::new(
            config.jwt_secret.clone(),
            config.jwt_access_ttl_secs,
            config.jwt_refresh_ttl_secs,
        );

        let ai = AiClient::new(
            config.ai_base_url.clone(),
            config.ai_api_key.clone(),
            config.ai_model.clone(),
        );

        let cipher = match &config.aes_key_base64 {
            Some(key) if !key.is_empty() => Some(AesCipher::from_base64_key(key)?),
            _ => None,
        };

        let id_tokens = IdTokenVerifier::new(
            config.firebase_project_id.clone(),
            config.google_client_ids.clone(),
        )
        .map_err(|e| anyhow::anyhow!("{e}"))?;

        let nse = NseIpoClient::new().map_err(|e| anyhow::anyhow!("{e}"))?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                db,
                redis,
                config,
                jwt,
                ai,
                cipher,
                id_tokens,
                nse,
            }),
        })
    }

    pub fn db(&self) -> &PgPool {
        &self.inner.db
    }

    pub fn jwt(&self) -> &JwtService {
        &self.inner.jwt
    }

    pub fn ai(&self) -> &AiClient {
        &self.inner.ai
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn cipher(&self) -> Option<&AesCipher> {
        self.inner.cipher.as_ref()
    }

    pub fn id_tokens(&self) -> &IdTokenVerifier {
        &self.inner.id_tokens
    }

    pub fn nse(&self) -> &NseIpoClient {
        &self.inner.nse
    }
}

impl std::ops::Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
