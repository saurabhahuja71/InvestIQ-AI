use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use redis::AsyncCommands;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::middleware::AuthUser;
use crate::modules::common::{ApiResponse, Meta};
use crate::modules::ipo::models::*;
use crate::state::AppState;

const DETAIL_SELECT: &str = r#"
        SELECT i.id, i.company_id, c.name AS company_name, c.symbol, c.sector, c.industry, c.description,
               c.logo_url, c.website, i.board::text, i.status::text, i.issue_type,
               i.price_band_low, i.price_band_high, i.issue_price, i.lot_size, i.face_value,
               i.min_investment, i.issue_size_cr, i.shares_offered,
               i.open_date, i.close_date, i.allotment_date, i.refund_date, i.listing_date,
               i.exchange, i.registrar, COALESCE(i.lead_managers, '[]'::jsonb) AS lead_managers,
               i.subscription_total, i.subscription_retail, i.subscription_qib, i.subscription_nii,
               i.listing_open, i.listing_close,
               i.gmp_value, i.gmp_updated_at, i.gmp_disclaimer,
               COALESCE(i.financials, '{}'::jsonb) AS financials,
               COALESCE(i.pros, '[]'::jsonb) AS pros,
               COALESCE(i.risks, '[]'::jsonb) AS risks,
               i.ai_summary, i.drhp_url, i.rhp_url,
               COALESCE(i.issue_info, '{}'::jsonb) AS issue_info,
               i.source, i.source_synced_at, i.nse_symbol, i.nse_series
        FROM ipos i
        JOIN companies c ON c.id = i.company_id
"#;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_ipos))
        .route("/sync", post(sync_ipos))
        .route("/calendar", get(calendar))
        .route("/watchlist", get(watchlist))
        .route("/{id}", get(get_ipo))
        .route("/{id}/watch", post(add_watch).delete(remove_watch))
        .route("/{id}/ai-summary", get(ai_summary))
        .route("/{id}/allotment-check", post(allotment_check))
}

async fn sync_ipos(State(state): State<AppState>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let report = state
        .ipo_sync()
        .sync_now()
        .await
        .map_err(|e| AppError::BadRequest(format!("IPO sync failed: {e}")))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "synced": report.synced,
        "details_enriched": report.details_enriched,
        "duration_ms": report.duration_ms,
        "source": report.source,
        "errors": report.errors,
        "last_sync_at": state.ipo_sync().last_sync_at().await,
    }))))
}

async fn list_ipos(
    State(state): State<AppState>,
    Query(q): Query<IpoQuery>,
) -> AppResult<Json<ApiResponse<Vec<IpoListItem>>>> {
    if q.refresh.unwrap_or(false) {
        match state.ipo_sync().sync_now().await {
            Ok(r) => tracing::info!(synced = r.synced, "IPO list refresh sync ok"),
            Err(e) => tracing::warn!(error = %e, "IPO list refresh sync skipped/failed"),
        }
    } else {
        // Soft background refresh when stale — do not block the list response.
        let sync = state.ipo_sync().clone();
        let interval = state.config().ipo_sync_interval_secs;
        tokio::spawn(async move {
            if let Err(e) = sync.sync_if_stale(interval).await {
                tracing::debug!(error = %e, "stale IPO sync not started");
            }
        });
    }

    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let status = q.status.as_deref();
    let board = q.board.as_deref();
    let search = q.q.as_deref();

    let cache_key = format!(
        "ipos:list:{}:{}:{}:{}:{}",
        status.unwrap_or("all"),
        board.unwrap_or("all"),
        search.unwrap_or(""),
        page,
        per_page
    );

    if !q.refresh.unwrap_or(false) {
        if let Ok(Some(cached)) = get_cached_list(&state, &cache_key).await {
            return Ok(Json(cached));
        }
    }

    let rows = sqlx::query_as::<_, IpoListItem>(
        r#"
        SELECT i.id,
               c.name AS company_name,
               c.symbol,
               i.board::text AS board,
               i.status::text AS status,
               i.price_band_low,
               i.price_band_high,
               i.issue_price,
               i.lot_size,
               i.min_investment,
               i.open_date,
               i.close_date,
               i.listing_date,
               i.exchange,
               i.subscription_total,
               i.gmp_value,
               TRUE AS gmp_unofficial,
               c.logo_url,
               i.source,
               i.source_synced_at
        FROM ipos i
        JOIN companies c ON c.id = i.company_id
        WHERE ($1::text IS NULL OR i.status::text = $1)
          AND ($2::text IS NULL OR i.board::text = $2)
          AND ($3::text IS NULL OR c.name ILIKE '%' || $3 || '%' OR COALESCE(c.symbol,'') ILIKE '%' || $3 || '%')
        ORDER BY
          CASE i.status::text
            WHEN 'open' THEN 0
            WHEN 'upcoming' THEN 1
            WHEN 'closed' THEN 2
            WHEN 'listed' THEN 3
            ELSE 4
          END,
          i.open_date DESC NULLS LAST
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(status)
    .bind(board)
    .bind(search)
    .bind(per_page)
    .bind(offset)
    .fetch_all(state.db())
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM ipos i
        JOIN companies c ON c.id = i.company_id
        WHERE ($1::text IS NULL OR i.status::text = $1)
          AND ($2::text IS NULL OR i.board::text = $2)
          AND ($3::text IS NULL OR c.name ILIKE '%' || $3 || '%' OR COALESCE(c.symbol,'') ILIKE '%' || $3 || '%')
        "#,
    )
    .bind(status)
    .bind(board)
    .bind(search)
    .fetch_one(state.db())
    .await?;

    let meta = Meta {
        page,
        per_page,
        total,
    };
    let response = ApiResponse::ok_with_meta(rows, meta);
    let _ = put_cached_list(&state, &cache_key, &response).await;
    Ok(Json(response))
}

async fn get_cached_list(
    state: &AppState,
    key: &str,
) -> anyhow::Result<Option<ApiResponse<Vec<IpoListItem>>>> {
    let mut conn = state.redis.clone();
    let raw: Option<String> = conn.get(key).await?;
    Ok(raw.and_then(|s| serde_json::from_str(&s).ok()))
}

async fn put_cached_list(
    state: &AppState,
    key: &str,
    value: &ApiResponse<Vec<IpoListItem>>,
) -> anyhow::Result<()> {
    let mut conn = state.redis.clone();
    let ttl = state.config().ipo_list_cache_ttl_secs.max(30);
    let payload = serde_json::to_string(value)?;
    let _: () = conn.set_ex(key, payload, ttl).await?;
    Ok(())
}

async fn get_ipo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<IpoDetail>>> {
    let row = sqlx::query_as::<_, IpoDetailRow>(&format!("{DETAIL_SELECT} WHERE i.id = $1"))
        .bind(id)
        .fetch_optional(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("IPO not found".into()))?;

    Ok(Json(ApiResponse::ok(IpoDetail::from(row))))
}

async fn calendar(
    State(state): State<AppState>,
    Query(q): Query<IpoQuery>,
) -> AppResult<Json<ApiResponse<Vec<IpoListItem>>>> {
    let mut q = q;
    if q.per_page.is_none() {
        q.per_page = Some(100);
    }
    list_ipos(State(state), Query(q)).await
}

async fn add_watch(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    sqlx::query(
        r#"
        INSERT INTO ipo_watchlist (user_id, ipo_id) VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(user.user_id)
    .bind(id)
    .execute(state.db())
    .await?;
    Ok(Json(ApiResponse::ok("watched")))
}

async fn remove_watch(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    sqlx::query(r#"DELETE FROM ipo_watchlist WHERE user_id = $1 AND ipo_id = $2"#)
        .bind(user.user_id)
        .bind(id)
        .execute(state.db())
        .await?;
    Ok(Json(ApiResponse::ok("removed")))
}

async fn watchlist(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<Vec<IpoListItem>>>> {
    let rows = sqlx::query_as::<_, IpoListItem>(
        r#"
        SELECT i.id, c.name AS company_name, c.symbol, i.board::text, i.status::text,
               i.price_band_low, i.price_band_high, i.issue_price, i.lot_size, i.min_investment,
               i.open_date, i.close_date, i.listing_date, i.exchange, i.subscription_total,
               i.gmp_value, TRUE AS gmp_unofficial, c.logo_url, i.source, i.source_synced_at
        FROM ipo_watchlist w
        JOIN ipos i ON i.id = w.ipo_id
        JOIN companies c ON c.id = i.company_id
        WHERE w.user_id = $1
        ORDER BY i.open_date DESC NULLS LAST
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn ai_summary(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let row = sqlx::query_as::<_, IpoDetailRow>(&format!("{DETAIL_SELECT} WHERE i.id = $1"))
        .bind(id)
        .fetch_optional(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("IPO not found".into()))?;

    if let Some(summary) = &row.ai_summary {
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "summary": summary,
            "disclaimer": crate::infra::ai::INVESTMENT_DISCLAIMER,
            "cached": true
        }))));
    }

    let ctx = format!(
        "IPO: {} | Board: {} | Price band: {:?} - {:?} | Registrar: {:?} | Financials: {} | Issue info keys: {}",
        row.company_name,
        row.board,
        row.price_band_low,
        row.price_band_high,
        row.registrar,
        row.financials,
        row.issue_info
            .as_object()
            .map(|o| o.keys().cloned().collect::<Vec<_>>().join(", "))
            .unwrap_or_default()
    );

    let summary = state
        .ai()
        .chat(
            vec![crate::infra::ai::ChatMessage {
                role: "user".into(),
                content: "Provide a concise educational summary of this IPO from exchange-published issue details. Do not invent financials. Do not guarantee returns. If data is missing, say so.".into(),
            }],
            Some(ctx),
        )
        .await?;

    sqlx::query(r#"UPDATE ipos SET ai_summary = $2, ai_summary_at = NOW() WHERE id = $1"#)
        .bind(id)
        .bind(&summary)
        .execute(state.db())
        .await?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "summary": summary,
        "disclaimer": crate::infra::ai::INVESTMENT_DISCLAIMER,
        "cached": false
    }))))
}

async fn allotment_check(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AllotmentRequest>,
) -> AppResult<Json<ApiResponse<AllotmentResult>>> {
    #[derive(sqlx::FromRow)]
    struct IpoAllotMeta {
        status: String,
        lot_size: Option<i32>,
        subscription_total: Option<rust_decimal::Decimal>,
    }

    let meta = sqlx::query_as::<_, IpoAllotMeta>(
        r#"
        SELECT status::text AS status, lot_size, subscription_total
        FROM ipos WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("IPO not found".into()))?;

    if let Some(ref pan) = body.pan_last4 {
        if pan.len() != 4 || !pan.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(AppError::Validation(
                "pan_last4 must be exactly 4 alphanumeric characters".into(),
            ));
        }
    }

    let result = crate::modules::ipo::allotment::compute_allotment(
        id,
        user.user_id,
        &meta.status,
        meta.lot_size,
        meta.subscription_total,
        body.pan_last4.as_deref(),
        body.application_number.as_deref(),
    );

    sqlx::query(
        r#"
        INSERT INTO allotment_checks (user_id, ipo_id, pan_last4, application_number, status, shares)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(user.user_id)
    .bind(id)
    .bind(&body.pan_last4)
    .bind(&body.application_number)
    .bind(&result.status)
    .bind(result.shares)
    .execute(state.db())
    .await?;

    if result.status == "allotted" || result.status == "not_allotted" {
        let title = if result.status == "allotted" {
            "IPO allotment (indicative)"
        } else {
            "IPO not allotted (indicative)"
        };
        let _ = sqlx::query(
            r#"
            INSERT INTO notifications (user_id, notif_type, title, body, payload)
            VALUES ($1, 'allotment', $2, $3, $4)
            "#,
        )
        .bind(user.user_id)
        .bind(title)
        .bind(&result.message)
        .bind(serde_json::json!({ "ipo_id": id, "status": result.status, "shares": result.shares }))
        .execute(state.db())
        .await;
    }

    Ok(Json(ApiResponse::ok(result)))
}
