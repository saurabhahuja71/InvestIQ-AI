use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    pub full_name: Option<String>,
    #[validate(length(equal = 3))]
    pub preferred_currency: Option<String>,
    pub preferred_locale: Option<String>,
    pub theme_preference: Option<String>,
    pub biometric_enabled: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub preferred_currency: String,
    pub preferred_locale: String,
    pub theme_preference: String,
    pub biometric_enabled: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub preferred_currency: String,
    pub preferred_locale: String,
    pub theme_preference: String,
    pub biometric_enabled: bool,
    pub email_verified: bool,
}

impl From<UserRow> for PublicUser {
    fn from(u: UserRow) -> Self {
        Self {
            id: u.id,
            email: u.email,
            full_name: u.full_name,
            preferred_currency: u.preferred_currency,
            preferred_locale: u.preferred_locale,
            theme_preference: u.theme_preference,
            biometric_enabled: u.biometric_enabled,
            email_verified: u.email_verified,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: PublicUser,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: &'static str,
}
