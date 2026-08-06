//! HTTP handlers for GET /alerts and PUT /alerts/preferences.

use axum::extract::State;
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::modules::alerts::logic::{
    default_ipo_alert_prefs, evaluate_ipo_alerts, merge_ipo_alert_prefs, pref_enabled, IpoAlertKind,
};
use crate::modules::common::ApiResponse;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_alerts))
        .route("/preferences", put(put_preferences).get(get_preferences))
        .route("/sync", axum::routing::post(sync_watchlist_alerts))
}

#[derive(Debug, Serialize, FromRow)]
struct AlertRow {
    id: Uuid,
    notif_type: String,
    title: String,
    body: String,
    payload: serde_json::Value,
    read_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PrefsBody {
    /// Full prefs map or partial IPO alert toggles.
    pub preferences: Option<serde_json::Value>,
    /// Alias accepted for clients that send `prefs`.
    pub prefs: Option<serde_json::Value>,
}

async fn list_alerts(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<Vec<AlertRow>>>> {
    let rows = sqlx::query_as::<_, AlertRow>(
        r#"
        SELECT id, notif_type::text, title, body, payload, read_at, created_at
        FROM notifications
        WHERE user_id = $1
          AND notif_type IN (
            'ipo_open'::notif_type,
            'ipo_close'::notif_type,
            'allotment'::notif_type,
            'listing_day'::notif_type
          )
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn get_preferences(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let prefs = load_merged_prefs(&state, user.user_id).await?;
    Ok(Json(ApiResponse::ok(prefs)))
}

async fn put_preferences(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<PrefsBody>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let incoming = body
        .preferences
        .or(body.prefs)
        .unwrap_or_else(|| serde_json::json!({}));

    // Merge with existing full prefs so portfolio/price keys are preserved.
    let existing: serde_json::Value = sqlx::query_scalar(
        r#"SELECT prefs FROM notification_prefs WHERE user_id = $1"#,
    )
    .bind(user.user_id)
    .fetch_optional(state.db())
    .await?
    .unwrap_or_else(default_ipo_alert_prefs);

    let mut merged = existing;
    let ipo_slice = merge_ipo_alert_prefs(&incoming);
    if let (Some(dst), Some(src)) = (merged.as_object_mut(), ipo_slice.as_object()) {
        for (k, v) in src {
            dst.insert(k.clone(), v.clone());
        }
        // Also accept any extra bool keys from client (e.g. portfolio_alert) without inventing data.
        if let Some(inc) = incoming.as_object() {
            for (k, v) in inc {
                if v.is_boolean() {
                    dst.insert(k.clone(), v.clone());
                }
            }
        }
    }

    sqlx::query(
        r#"
        INSERT INTO notification_prefs (user_id, prefs, updated_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (user_id) DO UPDATE SET prefs = EXCLUDED.prefs, updated_at = NOW()
        "#,
    )
    .bind(user.user_id)
    .bind(&merged)
    .execute(state.db())
    .await?;

    Ok(Json(ApiResponse::ok(merge_ipo_alert_prefs(&merged))))
}

/// Evaluate watchlist IPOs and create in-app notifications for enabled prefs.
async fn sync_watchlist_alerts(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let created = generate_watchlist_alerts(&state, user.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "created": created }))))
}

async fn load_merged_prefs(state: &AppState, user_id: Uuid) -> AppResult<serde_json::Value> {
    let prefs: Option<serde_json::Value> = sqlx::query_scalar(
        r#"SELECT prefs FROM notification_prefs WHERE user_id = $1"#,
    )
    .bind(user_id)
    .fetch_optional(state.db())
    .await?;

    let prefs = match prefs {
        Some(p) => p,
        None => {
            let default = {
                let mut d = default_ipo_alert_prefs();
                if let Some(obj) = d.as_object_mut() {
                    obj.insert("portfolio_alert".into(), serde_json::json!(true));
                    obj.insert("price_alert".into(), serde_json::json!(true));
                    obj.insert("dividend_alert".into(), serde_json::json!(true));
                    obj.insert("news_alert".into(), serde_json::json!(false));
                }
                d
            };
            sqlx::query(
                r#"INSERT INTO notification_prefs (user_id, prefs) VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
            )
            .bind(user_id)
            .bind(&default)
            .execute(state.db())
            .await?;
            default
        }
    };

    Ok(merge_ipo_alert_prefs(&prefs))
}

/// Shared by `/alerts/sync` and notifications sync path.
pub async fn generate_watchlist_alerts(state: &AppState, user_id: Uuid) -> AppResult<i64> {
    let prefs: serde_json::Value = sqlx::query_scalar(
        r#"SELECT prefs FROM notification_prefs WHERE user_id = $1"#,
    )
    .bind(user_id)
    .fetch_optional(state.db())
    .await?
    .unwrap_or_else(default_ipo_alert_prefs);

    #[derive(FromRow)]
    struct WatchedIpo {
        id: Uuid,
        name: String,
        status: String,
        open_date: Option<chrono::NaiveDate>,
        close_date: Option<chrono::NaiveDate>,
        allotment_date: Option<chrono::NaiveDate>,
        listing_date: Option<chrono::NaiveDate>,
    }

    let ipos = sqlx::query_as::<_, WatchedIpo>(
        r#"
        SELECT i.id, c.name, i.status::text, i.open_date, i.close_date, i.allotment_date, i.listing_date
        FROM ipo_watchlist w
        JOIN ipos i ON i.id = w.ipo_id
        JOIN companies c ON c.id = i.company_id
        WHERE w.user_id = $1
        ORDER BY i.open_date NULLS LAST
        "#,
    )
    .bind(user_id)
    .fetch_all(state.db())
    .await?;

    let today = Utc::now().date_naive();
    let mut created = 0i64;

    for ipo in ipos {
        let kinds = evaluate_ipo_alerts(
            today,
            &ipo.status,
            ipo.open_date,
            ipo.close_date,
            ipo.allotment_date,
            ipo.listing_date,
        );
        for kind in kinds {
            if !pref_enabled(&prefs, kind) {
                continue;
            }
            let event = match kind {
                IpoAlertKind::Open => "ipo_open",
                IpoAlertKind::ClosesToday => "ipo_close",
                IpoAlertKind::AllotmentAnnounced => "allotment",
                IpoAlertKind::ListingTomorrow => "listing_tomorrow",
                IpoAlertKind::ListingToday => "listing_day",
            };
            created += insert_unique_alert(
                state,
                user_id,
                kind.notif_type(),
                &kind.title(&ipo.name),
                kind.body(),
                serde_json::json!({
                    "ipo_id": ipo.id,
                    "event": event
                }),
            )
            .await?;
        }
    }

    Ok(created)
}

async fn insert_unique_alert(
    state: &AppState,
    user_id: Uuid,
    notif_type: &str,
    title: &str,
    body: &str,
    payload: serde_json::Value,
) -> AppResult<i64> {
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
