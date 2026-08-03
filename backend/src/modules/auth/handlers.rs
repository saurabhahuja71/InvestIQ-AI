use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::infra::password::{hash_password, verify_password};
use crate::middleware::AuthUser;
use crate::modules::auth::models::*;
use crate::modules::common::ApiResponse;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(me).patch(update_me))
        .route("/change-password", post(change_password))
        .route("/export", post(export_data))
        .route("/account", axum::routing::delete(delete_account))
}

fn token_hash(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<Json<ApiResponse<AuthResponse>>> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let email = body.email.trim().to_lowercase();
    let password_hash = hash_password(&body.password)?;

    let user = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (email, password_hash, full_name)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, full_name, preferred_currency,
                  preferred_locale, theme_preference, biometric_enabled,
                  email_verified, created_at
        "#,
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(&body.full_name)
    .fetch_one(state.db())
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.constraint().is_some() => {
            AppError::Conflict("email already registered".into())
        }
        other => other.into(),
    })?;

    // Default portfolio
    sqlx::query(
        r#"INSERT INTO portfolios (user_id, name, is_default) VALUES ($1, 'Main', true)"#,
    )
    .bind(user.id)
    .execute(state.db())
    .await?;

    let tokens = issue_tokens(&state, &user).await?;
    Ok(Json(ApiResponse::ok(tokens)))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<AuthResponse>>> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let email = body.email.trim().to_lowercase();
    let status: Option<String> = sqlx::query_scalar(
        r#"SELECT status::text FROM users WHERE email = $1"#,
    )
    .bind(&email)
    .fetch_optional(state.db())
    .await?;

    match status.as_deref() {
        Some("suspended") => {
            return Err(AppError::Forbidden("account suspended".into()));
        }
        Some("deleted") => {
            return Err(AppError::Forbidden("account deleted".into()));
        }
        Some("active") => {}
        Some(_) => {
            return Err(AppError::Forbidden("account not allowed to login".into()));
        }
        None => {
            return Err(AppError::Unauthorized("invalid credentials".into()));
        }
    }

    let user = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, password_hash, full_name, preferred_currency,
               preferred_locale, theme_preference, biometric_enabled,
               email_verified, created_at
        FROM users WHERE email = $1 AND status = 'active'
        "#,
    )
    .bind(&email)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;

    if !verify_password(&body.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }

    let tokens = issue_tokens(&state, &user).await?;
    Ok(Json(ApiResponse::ok(tokens)))
}

async fn issue_tokens(state: &AppState, user: &UserRow) -> AppResult<AuthResponse> {
    let access = state.jwt().issue_access(user.id, &user.email)?;
    let (refresh, jti, refresh_ttl) = state.jwt().issue_refresh(user.id, &user.email)?;
    let hash = token_hash(&refresh);
    let expires = Utc::now() + Duration::seconds(refresh_ttl);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user.id)
    .bind(&hash)
    .bind(expires)
    .execute(state.db())
    .await?;

    // jti reserved for future denylist
    let _ = jti;

    Ok(AuthResponse {
        user: PublicUser::from(UserRow {
            id: user.id,
            email: user.email.clone(),
            password_hash: String::new(),
            full_name: user.full_name.clone(),
            preferred_currency: user.preferred_currency.clone(),
            preferred_locale: user.preferred_locale.clone(),
            theme_preference: user.theme_preference.clone(),
            biometric_enabled: user.biometric_enabled,
            email_verified: user.email_verified,
            created_at: user.created_at,
        }),
        access_token: access,
        refresh_token: refresh,
        expires_in: state.jwt().access_ttl(),
        token_type: "Bearer",
    })
}

async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> AppResult<Json<ApiResponse<AuthResponse>>> {
    let claims = state.jwt().decode(&body.refresh_token)?;
    if claims.token_type != "refresh" {
        return Err(AppError::Unauthorized("refresh token required".into()));
    }

    let hash = token_hash(&body.refresh_token);
    let row = sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = NOW()
        WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > NOW()
        RETURNING user_id
        "#,
    )
    .bind(&hash)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::Unauthorized("invalid refresh token".into()))?;

    let user = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, password_hash, full_name, preferred_currency,
               preferred_locale, theme_preference, biometric_enabled,
               email_verified, created_at
        FROM users WHERE id = $1 AND status = 'active'
        "#,
    )
    .bind(row)
    .fetch_one(state.db())
    .await?;

    let tokens = issue_tokens(&state, &user).await?;
    Ok(Json(ApiResponse::ok(tokens)))
}

async fn logout(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<RefreshRequest>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    let hash = token_hash(&body.refresh_token);
    sqlx::query(
        r#"
        UPDATE refresh_tokens SET revoked_at = NOW()
        WHERE token_hash = $1 AND user_id = $2
        "#,
    )
    .bind(&hash)
    .bind(user.user_id)
    .execute(state.db())
    .await?;

    Ok(Json(ApiResponse::ok("logged out")))
}

async fn me(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<PublicUser>>> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, password_hash, full_name, preferred_currency,
               preferred_locale, theme_preference, biometric_enabled,
               email_verified, created_at
        FROM users WHERE id = $1
        "#,
    )
    .bind(user.user_id)
    .fetch_one(state.db())
    .await?;

    Ok(Json(ApiResponse::ok(row.into())))
}

async fn update_me(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateProfileRequest>,
) -> AppResult<Json<ApiResponse<PublicUser>>> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let row = sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE users SET
            full_name = COALESCE($2, full_name),
            preferred_currency = COALESCE($3, preferred_currency),
            preferred_locale = COALESCE($4, preferred_locale),
            theme_preference = COALESCE($5, theme_preference),
            biometric_enabled = COALESCE($6, biometric_enabled),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, email, password_hash, full_name, preferred_currency,
                  preferred_locale, theme_preference, biometric_enabled,
                  email_verified, created_at
        "#,
    )
    .bind(user.user_id)
    .bind(&body.full_name)
    .bind(&body.preferred_currency)
    .bind(&body.preferred_locale)
    .bind(&body.theme_preference)
    .bind(body.biometric_enabled)
    .fetch_one(state.db())
    .await?;

    Ok(Json(ApiResponse::ok(row.into())))
}

async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChangePasswordRequest>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let hash: String = sqlx::query_scalar(r#"SELECT password_hash FROM users WHERE id = $1"#)
        .bind(user.user_id)
        .fetch_one(state.db())
        .await?;

    if !verify_password(&body.current_password, &hash)? {
        return Err(AppError::Unauthorized("current password is incorrect".into()));
    }

    let new_hash = hash_password(&body.new_password)?;
    sqlx::query(r#"UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1"#)
        .bind(user.user_id)
        .bind(&new_hash)
        .execute(state.db())
        .await?;

    sqlx::query(
        r#"UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL"#,
    )
    .bind(user.user_id)
    .execute(state.db())
    .await?;

    tracing::info!(email = %user.email, "password changed; sessions revoked");
    Ok(Json(ApiResponse::ok("password changed")))
}

async fn export_data(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let profile = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, password_hash, full_name, preferred_currency,
               preferred_locale, theme_preference, biometric_enabled,
               email_verified, created_at
        FROM users WHERE id = $1
        "#,
    )
    .bind(user.user_id)
    .fetch_one(state.db())
    .await?;

    let portfolios = sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        SELECT COALESCE(json_agg(row_to_json(p)), '[]'::json)
        FROM (SELECT id, name, base_currency, is_default, created_at FROM portfolios WHERE user_id = $1) p
        "#,
    )
    .bind(user.user_id)
    .fetch_one(state.db())
    .await?;

    let trades = sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)
        FROM (
          SELECT id, symbol, side::text, strategy_name, entry_price, exit_price, quantity,
                 entry_at, exit_at, pnl, notes, tags, created_at
          FROM journal_trades WHERE user_id = $1 AND deleted_at IS NULL
        ) t
        "#,
    )
    .bind(user.user_id)
    .fetch_one(state.db())
    .await?;

    let watchlist = sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        SELECT COALESCE(json_agg(row_to_json(w)), '[]'::json)
        FROM (
          SELECT ipo_id, created_at FROM ipo_watchlist WHERE user_id = $1
        ) w
        "#,
    )
    .bind(user.user_id)
    .fetch_one(state.db())
    .await?;

    let mut payload = serde_json::json!({
        "exported_at": chrono::Utc::now(),
        "user": PublicUser::from(profile),
        "portfolios": portfolios,
        "journal_trades": trades,
        "ipo_watchlist": watchlist,
    });

    // Optional field-level encryption seal for at-rest export archive metadata
    if let Some(cipher) = state.cipher() {
        let plaintext = format!("export:{}", user.user_id);
        let seal = cipher.encrypt_str(&plaintext)?;
        // Round-trip verify so decrypt path is exercised in production configs
        let verified = cipher.decrypt_str(&seal)?;
        if verified != plaintext {
            return Err(AppError::Internal("export seal verification failed".into()));
        }
        payload
            .as_object_mut()
            .map(|o| o.insert("integrity_seal".into(), serde_json::json!(seal)));
    }

    sqlx::query(
        r#"INSERT INTO data_export_jobs (user_id, status, payload) VALUES ($1, 'completed', $2)"#,
    )
    .bind(user.user_id)
    .bind(&payload)
    .execute(state.db())
    .await?;

    Ok(Json(ApiResponse::ok(payload)))
}

async fn delete_account(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<DeleteAccountRequest>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    if body.confirm != "DELETE" {
        return Err(AppError::Validation(
            "confirm must be the string DELETE".into(),
        ));
    }

    let hash: String = sqlx::query_scalar(r#"SELECT password_hash FROM users WHERE id = $1"#)
        .bind(user.user_id)
        .fetch_one(state.db())
        .await?;

    if !verify_password(&body.password, &hash)? {
        return Err(AppError::Unauthorized("password is incorrect".into()));
    }

    sqlx::query(
        r#"UPDATE users SET status = 'deleted', email = email || '.deleted.' || id::text, updated_at = NOW() WHERE id = $1"#,
    )
    .bind(user.user_id)
    .execute(state.db())
    .await?;

    sqlx::query(r#"DELETE FROM users WHERE id = $1"#)
        .bind(user.user_id)
        .execute(state.db())
        .await?;

    tracing::info!(email = %user.email, "account deleted");
    Ok(Json(ApiResponse::ok("account deleted")))
}
