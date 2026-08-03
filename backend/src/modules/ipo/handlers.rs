use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::middleware::AuthUser;
use crate::modules::common::{ApiResponse, Meta};
use crate::modules::ipo::models::*;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_ipos))
        .route("/calendar", get(calendar))
        .route("/watchlist", get(watchlist))
        .route("/{id}", get(get_ipo))
        .route("/{id}/watch", post(add_watch).delete(remove_watch))
        .route("/{id}/ai-summary", get(ai_summary))
        .route("/{id}/allotment-check", post(allotment_check))
}

async fn list_ipos(
    State(state): State<AppState>,
    Query(q): Query<IpoQuery>,
) -> AppResult<Json<ApiResponse<Vec<IpoListItem>>>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let status = q.status.as_deref();
    let board = q.board.as_deref();
    let search = q.q.as_deref();

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
               i.open_date,
               i.close_date,
               i.listing_date,
               i.subscription_total,
               i.gmp_value,
               TRUE AS gmp_unofficial,
               c.logo_url
        FROM ipos i
        JOIN companies c ON c.id = i.company_id
        WHERE ($1::text IS NULL OR i.status::text = $1)
          AND ($2::text IS NULL OR i.board::text = $2)
          AND ($3::text IS NULL OR c.name ILIKE '%' || $3 || '%' OR COALESCE(c.symbol,'') ILIKE '%' || $3 || '%')
        ORDER BY i.open_date DESC NULLS LAST
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

    Ok(Json(ApiResponse::ok_with_meta(
        rows,
        Meta {
            page,
            per_page,
            total,
        },
    )))
}

async fn get_ipo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<IpoDetail>>> {
    let row = sqlx::query_as::<_, IpoDetailRow>(
        r#"
        SELECT i.id, i.company_id, c.name AS company_name, c.symbol, c.sector, c.description,
               c.logo_url, i.board::text, i.status::text, i.issue_type,
               i.price_band_low, i.price_band_high, i.issue_price, i.lot_size, i.issue_size_cr,
               i.open_date, i.close_date, i.allotment_date, i.refund_date, i.listing_date,
               i.exchange, i.registrar,
               i.subscription_total, i.subscription_retail, i.subscription_qib, i.subscription_nii,
               i.listing_open, i.listing_close,
               i.gmp_value, i.gmp_updated_at, i.gmp_disclaimer,
               i.financials, i.pros, i.risks, i.ai_summary, i.drhp_url, i.rhp_url
        FROM ipos i
        JOIN companies c ON c.id = i.company_id
        WHERE i.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("IPO not found".into()))?;

    let detail = IpoDetail {
        gmp: GmpInfo {
            value: row.gmp_value,
            updated_at: row.gmp_updated_at,
            unofficial: true,
            disclaimer: row.gmp_disclaimer.clone(),
        },
        row: IpoDetailResponse {
            id: row.id,
            company_id: row.company_id,
            company_name: row.company_name,
            symbol: row.symbol,
            sector: row.sector,
            description: row.description,
            logo_url: row.logo_url,
            board: row.board,
            status: row.status,
            issue_type: row.issue_type,
            price_band_low: row.price_band_low,
            price_band_high: row.price_band_high,
            issue_price: row.issue_price,
            lot_size: row.lot_size,
            issue_size_cr: row.issue_size_cr,
            open_date: row.open_date,
            close_date: row.close_date,
            allotment_date: row.allotment_date,
            refund_date: row.refund_date,
            listing_date: row.listing_date,
            exchange: row.exchange,
            registrar: row.registrar,
            subscription_total: row.subscription_total,
            subscription_retail: row.subscription_retail,
            subscription_qib: row.subscription_qib,
            subscription_nii: row.subscription_nii,
            listing_open: row.listing_open,
            listing_close: row.listing_close,
            financials: row.financials,
            pros: row.pros,
            risks: row.risks,
            ai_summary: row.ai_summary,
            drhp_url: row.drhp_url,
            rhp_url: row.rhp_url,
        },
    };

    Ok(Json(ApiResponse::ok(detail)))
}

async fn calendar(
    State(state): State<AppState>,
    Query(q): Query<IpoQuery>,
) -> AppResult<Json<ApiResponse<Vec<IpoListItem>>>> {
    // Reuse list with date-oriented ordering
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
               i.price_band_low, i.price_band_high, i.issue_price, i.lot_size,
               i.open_date, i.close_date, i.listing_date, i.subscription_total,
               i.gmp_value, TRUE AS gmp_unofficial, c.logo_url
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
    let row = sqlx::query_as::<_, IpoDetailRow>(
        r#"
        SELECT i.id, i.company_id, c.name AS company_name, c.symbol, c.sector, c.description,
               c.logo_url, i.board::text, i.status::text, i.issue_type,
               i.price_band_low, i.price_band_high, i.issue_price, i.lot_size, i.issue_size_cr,
               i.open_date, i.close_date, i.allotment_date, i.refund_date, i.listing_date,
               i.exchange, i.registrar,
               i.subscription_total, i.subscription_retail, i.subscription_qib, i.subscription_nii,
               i.listing_open, i.listing_close,
               i.gmp_value, i.gmp_updated_at, i.gmp_disclaimer,
               i.financials, i.pros, i.risks, i.ai_summary, i.drhp_url, i.rhp_url
        FROM ipos i JOIN companies c ON c.id = i.company_id WHERE i.id = $1
        "#,
    )
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
        "IPO: {} | Board: {} | Price band: {:?} - {:?} | Financials: {} | Pros: {} | Risks: {}",
        row.company_name,
        row.board,
        row.price_band_low,
        row.price_band_high,
        row.financials,
        row.pros,
        row.risks
    );

    let summary = state
        .ai()
        .chat(
            vec![crate::infra::ai::ChatMessage {
                role: "user".into(),
                content: "Provide a concise educational summary of this IPO DRHP highlights, pros, and risks. Do not guarantee returns.".into(),
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

    // In-app notification when allotment is decided
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
