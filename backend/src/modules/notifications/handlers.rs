use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::middleware::AuthUser;
use crate::modules::common::ApiResponse;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notifications))
        .route("/read-all", post(read_all))
        .route("/{id}/read", post(mark_read))
        .route("/devices", post(register_device))
        .route("/prefs", get(get_prefs).put(put_prefs))
        .route("/alerts/price", get(list_price_alerts).post(create_price_alert))
        .route("/alerts/price/{id}", axum::routing::delete(delete_price_alert))
        .route("/sync-ipo-events", post(sync_ipo_events))
}

#[derive(Debug, Serialize, FromRow)]
struct NotificationRow {
    id: Uuid,
    notif_type: String,
    title: String,
    body: String,
    payload: serde_json::Value,
    read_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDevice {
    pub fcm_token: String,
    pub platform: String,
}

#[derive(Debug, Deserialize)]
pub struct PrefsBody {
    pub prefs: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreatePriceAlert {
    pub symbol: String,
    pub condition: String,
    pub threshold: Decimal,
}

#[derive(Debug, Serialize, FromRow)]
struct PriceAlertRow {
    id: Uuid,
    symbol: String,
    condition: String,
    threshold: Decimal,
    active: bool,
    triggered_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

async fn list_notifications(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<Vec<NotificationRow>>>> {
    let rows = sqlx::query_as::<_, NotificationRow>(
        r#"
        SELECT id, notif_type::text, title, body, payload, read_at, created_at
        FROM notifications WHERE user_id = $1
        ORDER BY created_at DESC LIMIT 100
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn mark_read(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    let res = sqlx::query(
        r#"UPDATE notifications SET read_at = NOW() WHERE id = $1 AND user_id = $2 AND read_at IS NULL"#,
    )
    .bind(id)
    .bind(user.user_id)
    .execute(state.db())
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("notification not found".into()));
    }
    Ok(Json(ApiResponse::ok("ok")))
}

async fn read_all(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    sqlx::query(
        r#"UPDATE notifications SET read_at = NOW() WHERE user_id = $1 AND read_at IS NULL"#,
    )
    .bind(user.user_id)
    .execute(state.db())
    .await?;
    Ok(Json(ApiResponse::ok("ok")))
}

async fn register_device(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<RegisterDevice>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    if body.fcm_token.trim().is_empty() {
        return Err(AppError::Validation("fcm_token required".into()));
    }
    let platform = body.platform.to_lowercase();
    if platform != "ios" && platform != "android" && platform != "web" {
        return Err(AppError::Validation("platform must be ios|android|web".into()));
    }
    sqlx::query(
        r#"
        INSERT INTO devices (user_id, fcm_token, platform)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, fcm_token) DO UPDATE SET platform = EXCLUDED.platform
        "#,
    )
    .bind(user.user_id)
    .bind(body.fcm_token.trim())
    .bind(platform)
    .execute(state.db())
    .await?;
    Ok(Json(ApiResponse::ok("registered")))
}

async fn get_prefs(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let prefs: Option<serde_json::Value> = sqlx::query_scalar(
        r#"SELECT prefs FROM notification_prefs WHERE user_id = $1"#,
    )
    .bind(user.user_id)
    .fetch_optional(state.db())
    .await?;

    let prefs = if let Some(p) = prefs {
        p
    } else {
        let default = serde_json::json!({
            "ipo_open": true,
            "ipo_close": true,
            "allotment": true,
            "listing_tomorrow": true,
            "listing_day": true,
            "portfolio_alert": true,
            "price_alert": true,
            "dividend_alert": true,
            "news_alert": false
        });
        sqlx::query(
            r#"INSERT INTO notification_prefs (user_id, prefs) VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
        )
        .bind(user.user_id)
        .bind(&default)
        .execute(state.db())
        .await?;
        default
    };

    Ok(Json(ApiResponse::ok(prefs)))
}

async fn put_prefs(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<PrefsBody>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    sqlx::query(
        r#"
        INSERT INTO notification_prefs (user_id, prefs, updated_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (user_id) DO UPDATE SET prefs = EXCLUDED.prefs, updated_at = NOW()
        "#,
    )
    .bind(user.user_id)
    .bind(&body.prefs)
    .execute(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(body.prefs)))
}

async fn list_price_alerts(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<Vec<PriceAlertRow>>>> {
    let rows = sqlx::query_as::<_, PriceAlertRow>(
        r#"
        SELECT id, symbol, condition, threshold, active, triggered_at, created_at
        FROM price_alerts WHERE user_id = $1 ORDER BY created_at DESC
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn create_price_alert(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreatePriceAlert>,
) -> AppResult<Json<ApiResponse<PriceAlertRow>>> {
    let cond = body.condition.to_lowercase();
    if cond != "above" && cond != "below" {
        return Err(AppError::Validation("condition must be above|below".into()));
    }
    let row = sqlx::query_as::<_, PriceAlertRow>(
        r#"
        INSERT INTO price_alerts (user_id, symbol, condition, threshold)
        VALUES ($1, $2, $3, $4)
        RETURNING id, symbol, condition, threshold, active, triggered_at, created_at
        "#,
    )
    .bind(user.user_id)
    .bind(body.symbol.to_uppercase())
    .bind(cond)
    .bind(body.threshold)
    .fetch_one(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(row)))
}

async fn delete_price_alert(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    let res = sqlx::query(r#"DELETE FROM price_alerts WHERE id = $1 AND user_id = $2"#)
        .bind(id)
        .bind(user.user_id)
        .execute(state.db())
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("alert not found".into()));
    }
    Ok(Json(ApiResponse::ok("deleted")))
}

/// Generate IPO alerts for the user's **watchlist** + evaluate price alerts.
async fn sync_ipo_events(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let mut created =
        crate::modules::alerts::handlers::generate_watchlist_alerts(&state, user.user_id).await?;

    // Evaluate price alerts against holdings last_price (unchanged; not Milestone 3 scope)
    let alerts = sqlx::query_as::<_, PriceAlertRow>(
        r#"
        SELECT id, symbol, condition, threshold, active, triggered_at, created_at
        FROM price_alerts WHERE user_id = $1 AND active = true
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;

    for alert in alerts {
        let price: Option<Decimal> = sqlx::query_scalar(
            r#"
            SELECT last_price FROM holdings h
            JOIN portfolios p ON p.id = h.portfolio_id
            WHERE p.user_id = $1 AND UPPER(COALESCE(h.symbol,'')) = $2
            LIMIT 1
            "#,
        )
        .bind(user.user_id)
        .bind(&alert.symbol)
        .fetch_optional(state.db())
        .await?
        .flatten();

        if let Some(px) = price {
            let hit = match alert.condition.as_str() {
                "above" => px >= alert.threshold,
                "below" => px <= alert.threshold,
                _ => false,
            };
            if hit {
                created += insert_unique_notif(
                    &state,
                    user.user_id,
                    "price_alert",
                    &format!("{} price alert", alert.symbol),
                    &format!(
                        "{} is {} {} (last {}). Alerts are informational only.",
                        alert.symbol, alert.condition, alert.threshold, px
                    ),
                    serde_json::json!({ "alert_id": alert.id, "price": px }),
                )
                .await?;
                let _ = sqlx::query(
                    r#"UPDATE price_alerts SET triggered_at = NOW(), active = false WHERE id = $1"#,
                )
                .bind(alert.id)
                .execute(state.db())
                .await;
            }
        }
    }

    Ok(Json(ApiResponse::ok(serde_json::json!({ "created": created }))))
}

async fn insert_unique_notif(
    state: &AppState,
    user_id: Uuid,
    notif_type: &str,
    title: &str,
    body: &str,
    payload: serde_json::Value,
) -> AppResult<i64> {
    // Avoid spamming identical titles in last 24h
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
          SELECT 1 FROM notifications
          WHERE user_id = $1 AND title = $2 AND created_at > NOW() - INTERVAL '24 hours'
        )
        "#,
    )
    .bind(user_id)
    .bind(title)
    .fetch_one(state.db())
    .await?;

    if exists {
        return Ok(0);
    }

    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, notif_type, title, body, payload)
        VALUES ($1, $2::notif_type, $3, $4, $5)
        "#,
    )
    .bind(user_id)
    .bind(notif_type)
    .bind(title)
    .bind(body)
    .bind(payload)
    .execute(state.db())
    .await?;
    Ok(1)
}
