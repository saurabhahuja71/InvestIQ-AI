use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub token_type: String,
}

#[derive(Clone)]
pub struct JwtService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    access_ttl: i64,
    refresh_ttl: i64,
}

impl JwtService {
    pub fn new(secret: String, access_ttl: i64, refresh_ttl: i64) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            access_ttl,
            refresh_ttl,
        }
    }

    pub fn issue_access(&self, user_id: Uuid, email: &str) -> AppResult<String> {
        self.issue(user_id, email, "access", self.access_ttl)
    }

    pub fn issue_refresh(&self, user_id: Uuid, email: &str) -> AppResult<(String, String, i64)> {
        let jti = Uuid::new_v4().to_string();
        let token = self.issue_with_jti(user_id, email, "refresh", self.refresh_ttl, &jti)?;
        Ok((token, jti, self.refresh_ttl))
    }

    fn issue(&self, user_id: Uuid, email: &str, token_type: &str, ttl: i64) -> AppResult<String> {
        let jti = Uuid::new_v4().to_string();
        self.issue_with_jti(user_id, email, token_type, ttl, &jti)
    }

    fn issue_with_jti(
        &self,
        user_id: Uuid,
        email: &str,
        token_type: &str,
        ttl: i64,
        jti: &str,
    ) -> AppResult<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: (now + Duration::seconds(ttl)).timestamp(),
            iat: now.timestamp(),
            jti: jti.to_string(),
            token_type: token_type.to_string(),
        };
        encode(&Header::default(), &claims, &self.encoding)
            .map_err(|e| AppError::Internal(format!("jwt encode: {e}")))
    }

    pub fn decode(&self, token: &str) -> AppResult<Claims> {
        let data = decode::<Claims>(token, &self.decoding, &Validation::default())
            .map_err(|_| AppError::Unauthorized("invalid token".into()))?;
        Ok(data.claims)
    }

    pub fn access_ttl(&self) -> i64 {
        self.access_ttl
    }
}
