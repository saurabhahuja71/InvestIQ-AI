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
    /// Email verification code issued by `POST /auth/register/otp`.
    #[validate(length(min = 6, max = 6))]
    pub otp: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RequestRegisterOtpRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct RequestRegisterOtpResponse {
    pub sent: bool,
    /// Present when the code was generated (no SMTP yet, so the code is
    /// returned to the app for now — mirroring the password-reset flow).
    pub otp: Option<String>,
    pub expires_in_secs: i64,
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

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
    pub confirm: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GoogleAuthRequest {
    pub id_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(email)]
    pub email: String,
    pub token: String,
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    pub sent: bool,
    /// Present only when the account exists (no SMTP yet, so the reset
    /// code is returned to the app for now).
    pub reset_token: Option<String>,
    pub expires_in_secs: i64,
}


#[derive(Debug, Serialize, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
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
    pub auth_provider: Option<String>,
    pub avatar_url: Option<String>,
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
            auth_provider: None,
            avatar_url: None,
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
