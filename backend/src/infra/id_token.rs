//! Verify Firebase Authentication and Google Sign-In ID tokens via JWKS.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::error::{AppError, AppResult};

const FIREBASE_JWKS_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";
const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
const JWKS_TTL: Duration = Duration::from_secs(3600);

#[derive(Debug, Clone)]
pub struct VerifiedIdentity {
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub firebase_uid: Option<String>,
    pub google_sub: Option<String>,
}

#[derive(Clone)]
pub struct IdTokenVerifier {
    http: reqwest::Client,
    firebase_project_id: Option<String>,
    google_client_ids: Vec<String>,
    cache: Arc<RwLock<HashMap<String, CachedJwks>>>,
}

struct CachedJwks {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
}

#[derive(Debug, Deserialize)]
struct JwksDoc {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: Option<String>,
    kty: String,
    n: Option<String>,
    e: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenClaims {
    sub: String,
    email: Option<String>,
    email_verified: Option<serde_json::Value>,
    name: Option<String>,
    picture: Option<String>,
    #[allow(dead_code)]
    iss: String,
    aud: serde_json::Value,
    exp: i64,
    user_id: Option<String>,
}

impl IdTokenVerifier {
    pub fn new(
        firebase_project_id: Option<String>,
        google_client_ids: Vec<String>,
    ) -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("investiq-api/0.1")
            .build()
            .map_err(|e| AppError::Internal(format!("http client: {e}")))?;

        Ok(Self {
            http,
            firebase_project_id,
            google_client_ids,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn is_configured(&self) -> bool {
        self.firebase_project_id
            .as_ref()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
            || !self.google_client_ids.is_empty()
    }

    pub async fn verify(&self, id_token: &str) -> AppResult<VerifiedIdentity> {
        if !self.is_configured() {
            return Err(AppError::Internal(
                "Google/Firebase auth is not configured (set FIREBASE_PROJECT_ID and/or GOOGLE_CLIENT_IDS)"
                    .into(),
            ));
        }

        let header = decode_header(id_token)
            .map_err(|_| AppError::Unauthorized("invalid id token header".into()))?;
        let kid = header
            .kid
            .ok_or_else(|| AppError::Unauthorized("id token missing kid".into()))?;

        let iss = peek_issuer(id_token)?;
        let (jwks_url, validation, is_firebase) = self.validation_for_issuer(&iss)?;

        let key = self.decoding_key(jwks_url, &kid).await?;
        let data = decode::<TokenClaims>(id_token, &key, &validation).map_err(|e| {
            tracing::warn!(error = %e, "id token verification failed");
            AppError::Unauthorized("invalid or expired id token".into())
        })?;

        let claims = data.claims;
        if claims.exp < chrono::Utc::now().timestamp() {
            return Err(AppError::Unauthorized("id token expired".into()));
        }

        // Re-check audience manually for multi-aud flexibility
        if !self.audience_ok(&claims.aud, is_firebase) {
            return Err(AppError::Unauthorized("id token audience mismatch".into()));
        }

        let email = claims
            .email
            .filter(|e| !e.is_empty())
            .ok_or_else(|| AppError::Unauthorized("id token missing email".into()))?
            .to_lowercase();

        let email_verified = match claims.email_verified {
            Some(serde_json::Value::Bool(b)) => b,
            Some(serde_json::Value::String(s)) => s.eq_ignore_ascii_case("true"),
            _ => false,
        };

        if !email_verified {
            return Err(AppError::Unauthorized(
                "email is not verified with Google".into(),
            ));
        }

        let (firebase_uid, google_sub) = if is_firebase {
            let uid = claims.user_id.clone().unwrap_or_else(|| claims.sub.clone());
            (Some(uid), Some(claims.sub.clone()))
        } else {
            (None, Some(claims.sub.clone()))
        };

        Ok(VerifiedIdentity {
            email,
            name: claims.name,
            picture: claims.picture,
            firebase_uid,
            google_sub,
        })
    }

    fn validation_for_issuer(&self, iss: &str) -> AppResult<(&'static str, Validation, bool)> {
        if let Some(project) = &self.firebase_project_id {
            let expected = format!("https://securetoken.google.com/{project}");
            if iss == expected {
                let mut v = Validation::new(Algorithm::RS256);
                v.set_audience(&[project.clone()]);
                v.set_issuer(&[expected]);
                v.validate_exp = true;
                return Ok((FIREBASE_JWKS_URL, v, true));
            }
        }

        if iss == "accounts.google.com" || iss == "https://accounts.google.com" {
            if self.google_client_ids.is_empty() {
                return Err(AppError::Unauthorized(
                    "Google client IDs not configured for this token".into(),
                ));
            }
            let mut v = Validation::new(Algorithm::RS256);
            v.set_audience(&self.google_client_ids);
            v.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);
            v.validate_exp = true;
            return Ok((GOOGLE_JWKS_URL, v, false));
        }

        Err(AppError::Unauthorized(format!(
            "unsupported id token issuer: {iss}"
        )))
    }

    fn audience_ok(&self, aud: &serde_json::Value, is_firebase: bool) -> bool {
        let audiences: Vec<String> = match aud {
            serde_json::Value::String(s) => vec![s.clone()],
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => return false,
        };

        if is_firebase {
            if let Some(project) = &self.firebase_project_id {
                return audiences.iter().any(|a| a == project);
            }
            return false;
        }

        audiences
            .iter()
            .any(|a| self.google_client_ids.iter().any(|c| c == a))
    }

    async fn decoding_key(&self, jwks_url: &str, kid: &str) -> AppResult<DecodingKey> {
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(jwks_url) {
                if entry.fetched_at.elapsed() < JWKS_TTL {
                    if let Some(key) = entry.keys.get(kid) {
                        return Ok(key.clone());
                    }
                }
            }
        }

        let keys = self.fetch_jwks(jwks_url).await?;
        let key = keys
            .get(kid)
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("id token kid not found in JWKS".into()))?;

        let mut cache = self.cache.write().await;
        cache.insert(
            jwks_url.to_string(),
            CachedJwks {
                keys,
                fetched_at: Instant::now(),
            },
        );
        Ok(key)
    }

    async fn fetch_jwks(&self, url: &str) -> AppResult<HashMap<String, DecodingKey>> {
        let doc: JwksDoc = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("jwks fetch failed: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Internal(format!("jwks status: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("jwks parse: {e}")))?;

        let mut map = HashMap::new();
        for jwk in doc.keys {
            if jwk.kty != "RSA" {
                continue;
            }
            let (Some(kid), Some(n), Some(e)) = (jwk.kid, jwk.n, jwk.e) else {
                continue;
            };
            match DecodingKey::from_rsa_components(&n, &e) {
                Ok(key) => {
                    map.insert(kid, key);
                }
                Err(err) => tracing::warn!(%err, "skip invalid jwk"),
            }
        }

        if map.is_empty() {
            return Err(AppError::Internal("empty JWKS set".into()));
        }
        Ok(map)
    }
}

fn peek_issuer(id_token: &str) -> AppResult<String> {
    let mut parts = id_token.split('.');
    let _header = parts
        .next()
        .ok_or_else(|| AppError::Unauthorized("invalid id token".into()))?;
    let payload = parts
        .next()
        .ok_or_else(|| AppError::Unauthorized("invalid id token".into()))?;
    let bytes = base64_url_decode(payload)
        .map_err(|_| AppError::Unauthorized("invalid id token payload".into()))?;
    let v: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| AppError::Unauthorized("invalid id token payload json".into()))?;
    v.get("iss")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Unauthorized("id token missing iss".into()))
}

fn base64_url_decode(input: &str) -> Result<Vec<u8>, ()> {
    use base64::Engine;
    let rem = input.len() % 4;
    let padded = if rem == 0 {
        input.to_string()
    } else {
        format!("{}{}", input, "=".repeat(4 - rem))
    };
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(input)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(padded.as_bytes()))
        .map_err(|_| ())
}
