//! IPO Intelligence HTTP handlers.
//!
//! Subscription snapshots + history, financials (with growth & valuation
//! analysis), the transparent fundamentals-first InvestIQ IPO Score, and the
//! data-sources/metadata endpoint. GMP is intentionally excluded from all
//! production API responses in v1; the architecture remains extensible so a
//! future market-sentiment provider can be added without touching the score.

use axum::extract::{Path, State};
use axum::{Json, Router};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::modules::common::ApiResponse;
use crate::modules::ipo_intel::logic::{
    analyze_series, compute_score, latest_yoy, ScoreInputs, SeriesPoint, SCORE_DISCLAIMER,
    SCORE_METHODOLOGY_VERSION,
};
use crate::modules::ipo_intel::models::*;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}/subscription", axum::routing::get(get_subscription))
        .route(
            "/{id}/subscription/history",
            axum::routing::get(get_subscription_history),
        )
        .route("/{id}/financials", axum::routing::get(get_financials))
        .route("/{id}/score", axum::routing::get(get_score))
        .route("/{id}/data-sources", axum::routing::get(get_data_sources))
}

async fn ipo_exists(state: &AppState, id: Uuid) -> AppResult<()> {
    let exists: bool =
        sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM ipos WHERE id = $1)"#)
            .bind(id)
            .fetch_one(state.db())
            .await?;
    if !exists {
        return Err(AppError::NotFound("IPO not found".into()));
    }
    Ok(())
}

async fn get_subscription(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<SubscriptionResponse>>> {
    ipo_exists(&state, id).await?;

    #[derive(sqlx::FromRow)]
    struct SnapshotRow {
        retail: Option<Decimal>,
        nii: Option<Decimal>,
        qib: Option<Decimal>,
        employee: Option<Decimal>,
        shareholder: Option<Decimal>,
        overall: Option<Decimal>,
        is_final: bool,
        source: String,
        source_type: String,
        updated_at: chrono::DateTime<Utc>,
    }

    let snap = sqlx::query_as::<_, SnapshotRow>(
        r#"
        SELECT retail, nii, qib, employee, shareholder, overall, is_final,
               source, source_type, updated_at
        FROM ipo_subscription_snapshots WHERE ipo_id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(state.db())
    .await?;

    let history = subscription_history(&state, id).await?;

    Ok(Json(ApiResponse::ok(match snap {
        Some(s) => SubscriptionResponse {
            available: true,
            retail: s.retail,
            nii: s.nii,
            qib: s.qib,
            employee: s.employee,
            shareholder: s.shareholder,
            overall: s.overall,
            is_final: s.is_final,
            source: Some(s.source),
            source_type: Some(s.source_type),
            updated_at: Some(s.updated_at),
            history,
        },
        None => {
            // No intel snapshot yet — fall back to the live values synced
            // from NSE on the ipos row so open issues show real demand.
            #[derive(sqlx::FromRow)]
            struct LiveRow {
                retail: Option<Decimal>,
                nii: Option<Decimal>,
                qib: Option<Decimal>,
                overall: Option<Decimal>,
                synced_at: Option<chrono::DateTime<Utc>>,
            }
            let live = sqlx::query_as::<_, LiveRow>(
                r#"
                SELECT subscription_retail AS retail, subscription_nii AS nii,
                       subscription_qib AS qib, subscription_total AS overall,
                       source_synced_at AS synced_at
                FROM ipos WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_one(state.db())
            .await?;
            let has_live = live.overall.is_some()
                || live.retail.is_some()
                || live.nii.is_some()
                || live.qib.is_some();
            SubscriptionResponse {
                available: has_live,
                retail: live.retail,
                nii: live.nii,
                qib: live.qib,
                employee: None,
                shareholder: None,
                overall: live.overall,
                is_final: false,
                source: Some("nse".to_string()),
                source_type: Some("live".to_string()),
                updated_at: live.synced_at,
                history,
            }
        }
    })))
}

async fn subscription_history(
    state: &AppState,
    id: Uuid,
) -> AppResult<Vec<SubscriptionPointRow>> {
    let rows = sqlx::query_as::<_, SubscriptionPointRow>(
        r#"
        SELECT day, retail, nii, qib, employee, shareholder, overall, is_final,
               source, source_type, captured_at
        FROM ipo_subscription_history WHERE ipo_id = $1 ORDER BY day ASC
        "#,
    )
    .bind(id)
    .fetch_all(state.db())
    .await?;
    Ok(rows)
}

async fn get_subscription_history(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<SubscriptionPointRow>>>> {
    ipo_exists(&state, id).await?;
    let rows = subscription_history(&state, id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn get_financials(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<FinancialsResponse>>> {
    ipo_exists(&state, id).await?;

    let issue_price: Option<Decimal> = sqlx::query_scalar::<_, Option<Decimal>>(
        r#"SELECT issue_price FROM ipos WHERE id = $1"#,
    )
    .bind(id)
    .fetch_one(state.db())
    .await?;

    let mut periods = sqlx::query_as::<_, FinancialPeriodRow>(
        r#"
        SELECT period, period_start, period_end, revenue, revenue_growth_pct,
               ebitda, ebitda_margin_pct, pat, pat_growth_pct, eps, pe_ratio,
               roe_pct, roce_pct, debt, debt_to_equity, audited,
               source, source_type, updated_at
        FROM ipo_financials WHERE ipo_id = $1 ORDER BY period ASC
        "#,
    )
    .bind(id)
    .fetch_all(state.db())
    .await?;

    let series = |f: &dyn Fn(&FinancialPeriodRow) -> Option<Decimal>| -> Vec<SeriesPoint> {
        periods
            .iter()
            .map(|p| SeriesPoint {
                period: p.period.clone(),
                value: f(p),
                period_end: p.period_end,
            })
            .collect()
    };

    let revenue_series = series(&|p: &FinancialPeriodRow| p.revenue);
    let pat_series = series(&|p: &FinancialPeriodRow| p.pat);
    let eps_series = series(&|p: &FinancialPeriodRow| p.eps);

    let growth = FinancialGrowth {
        revenue: analyze_series("Revenue", &revenue_series),
        pat: analyze_series("PAT", &pat_series),
        eps: analyze_series("EPS", &eps_series),
    };

    let latest = periods.last();
    let eps = latest.and_then(|p| p.eps);
    let pe_ratio = latest.and_then(|p| p.pe_ratio);
    let implied_pe = match (issue_price, eps) {
        (Some(price), Some(e)) if e > Decimal::ZERO => Some(price / e),
        _ => None,
    };
    let available = pe_ratio.is_some() || implied_pe.is_some();

    let valuation = ValuationResponse {
        available,
        pe_ratio,
        eps,
        issue_price,
        implied_pe,
        sector_pe: None,
        premium_discount_pct: None,
        note: if available {
            "P/E is computed from the latest available financial period and the \
             issue price. A reliable sector/peer benchmark is not available yet."
                .to_string()
        } else {
            "Valuation is not available: no P/E ratio or EPS figure is present."
                .to_string()
        },
    };

    Ok(Json(ApiResponse::ok(FinancialsResponse {
        available: !periods.is_empty(),
        periods,
        growth,
        valuation,
    })))
}

async fn get_score(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<ScoreResponse>>> {
    #[derive(sqlx::FromRow)]
    struct IpoMeta {
        issue_price: Option<Decimal>,
        subscription_total: Option<Decimal>,
        board: String,
        risks: serde_json::Value,
    }

    let meta = sqlx::query_as::<_, IpoMeta>(
        r#"SELECT issue_price, subscription_total, board::text AS board, risks FROM ipos WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("IPO not found".into()))?;

    let sector: Option<String> = sqlx::query_scalar::<_, Option<String>>(
        r#"SELECT c.sector FROM companies c JOIN ipos i ON i.company_id = c.id WHERE i.id = $1"#,
    )
    .bind(id)
    .fetch_one(state.db())
    .await?;

    // Load every financial period (chronological) so growth figures are derived
    // deterministically by the backend from the underlying series.
    #[derive(sqlx::FromRow)]
    struct FinRow {
        period: String,
        period_end: Option<chrono::NaiveDate>,
        revenue: Option<Decimal>,
        revenue_growth_pct: Option<Decimal>,
        pat: Option<Decimal>,
        pat_growth_pct: Option<Decimal>,
        eps: Option<Decimal>,
        pe_ratio: Option<Decimal>,
        ebitda_margin_pct: Option<Decimal>,
        roe_pct: Option<Decimal>,
        roce_pct: Option<Decimal>,
        debt_to_equity: Option<Decimal>,
    }

    let rows = sqlx::query_as::<_, FinRow>(
        r#"
        SELECT period, period_end, revenue, revenue_growth_pct,
               pat, pat_growth_pct, eps, pe_ratio,
               ebitda_margin_pct, roe_pct, roce_pct, debt_to_equity
        FROM ipo_financials WHERE ipo_id = $1 ORDER BY period ASC
        "#,
    )
    .bind(id)
    .fetch_all(state.db())
    .await?;

    let to_series = |f: &dyn Fn(&FinRow) -> Option<Decimal>| -> Vec<SeriesPoint> {
        rows.iter()
            .map(|r| SeriesPoint {
                period: r.period.clone(),
                value: f(r),
                period_end: r.period_end,
            })
            .collect()
    };

    let revenue_growth_pct = rows
        .last()
        .and_then(|r| r.revenue_growth_pct)
        .or_else(|| latest_yoy(&to_series(&|r| r.revenue)));
    let pat_growth_pct = rows
        .last()
        .and_then(|r| r.pat_growth_pct)
        .or_else(|| latest_yoy(&to_series(&|r| r.pat)));
    let eps_growth_pct = latest_yoy(&to_series(&|r| r.eps));

    let latest = rows.last();
    let pat_margin_pct = match (latest.and_then(|r| r.pat), latest.and_then(|r| r.revenue)) {
        (Some(pat), Some(rev)) if rev > Decimal::ZERO => Some(pat / rev * Decimal::from(100)),
        _ => None,
    };

    let has_risks = meta
        .risks
        .as_array()
        .map(|a| !a.is_empty())
        .unwrap_or(false);

    let inputs = ScoreInputs {
        revenue_growth_pct,
        pat_growth_pct,
        eps_growth_pct,
        ebitda_margin_pct: latest.and_then(|r| r.ebitda_margin_pct),
        pat_margin_pct,
        roe_pct: latest.and_then(|r| r.roe_pct),
        roce_pct: latest.and_then(|r| r.roce_pct),
        pe_ratio: latest.and_then(|r| r.pe_ratio),
        issue_price: meta.issue_price,
        eps: latest.and_then(|r| r.eps),
        debt_to_equity: latest.and_then(|r| r.debt_to_equity),
        subscription_overall: meta.subscription_total,
        sector,
        board: meta.board,
        has_risks,
    };

    let result = compute_score(inputs);

    let components: Vec<ScoreComponentResponse> = result
        .components
        .iter()
        .map(|c| ScoreComponentResponse {
            key: c.key.to_string(),
            label: c.label.to_string(),
            max_points: c.max_points,
            score: c
                .score
                .and_then(|s| Decimal::from_f64_retain((s * 10.0).round() / 10.0)),
            status: if c.score.is_some() { "scored" } else { "insufficient_data" },
            explanation: c.explanation.clone(),
        })
        .collect();

    let total = result
        .total
        .and_then(|t| Decimal::from_f64_retain((t * 10.0).round() / 10.0));

    // Cache the score for audit/reproducibility (see migration v2 for the
    // components_json/data_quality_json schema).
    let components_json = serde_json::to_value(&components).unwrap_or_else(|_| serde_json::json!([]));
    let positives_json =
        serde_json::to_value(&result.positive_factors).unwrap_or_else(|_| serde_json::json!([]));
    let concerns_json =
        serde_json::to_value(&result.concerns).unwrap_or_else(|_| serde_json::json!([]));
    let data_quality_json = serde_json::json!({
        "overall": result.data_quality.overall,
        "missing": result.data_quality.missing,
    });

    let _ = sqlx::query(
        r#"
        INSERT INTO ipo_scores (
            ipo_id, total, components_json, data_quality_json,
            positive_factors, concerns, methodology_version, disclaimer, computed_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
        ON CONFLICT (ipo_id) DO UPDATE SET
            total = EXCLUDED.total,
            components_json = EXCLUDED.components_json,
            data_quality_json = EXCLUDED.data_quality_json,
            positive_factors = EXCLUDED.positive_factors,
            concerns = EXCLUDED.concerns,
            methodology_version = EXCLUDED.methodology_version,
            disclaimer = EXCLUDED.disclaimer,
            computed_at = NOW()
        "#,
    )
    .bind(id)
    .bind(total)
    .bind(components_json)
    .bind(data_quality_json)
    .bind(positives_json)
    .bind(concerns_json)
    .bind(SCORE_METHODOLOGY_VERSION)
    .bind(SCORE_DISCLAIMER)
    .execute(state.db())
    .await
    .ok();

    Ok(Json(ApiResponse::ok(ScoreResponse {
        total,
        max_points: result.max_points,
        methodology_version: SCORE_METHODOLOGY_VERSION.to_string(),
        data_quality: DataQualityResponse {
            overall: result.data_quality.overall.to_string(),
            missing: result.data_quality.missing,
        },
        components,
        positive_factors: result.positive_factors,
        concerns: result.concerns,
        disclaimer: SCORE_DISCLAIMER.to_string(),
        computed_at: Utc::now(),
    })))
}

/// Document every externally sourced data category for this IPO: provider,
/// official-vs-unofficial, licensing, refresh frequency, rate limits, plus the
/// per-IPO freshness timestamps actually stored. This makes the app's
/// "Data source & data quality" claims auditable.
///
/// The `gmp` category row is intentionally filtered out of the response in v1
/// (GMP is not integrated); the `data_sources` table keeps it for a future
/// market-sentiment provider.
async fn get_data_sources(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<IntelMetaResponse>>> {
    #[derive(sqlx::FromRow)]
    struct Meta {
        id: Uuid,
        company_name: String,
        source_synced_at: Option<chrono::DateTime<Utc>>,
    }

    let meta = sqlx::query_as::<_, Meta>(
        r#"
        SELECT i.id, c.name AS company_name, i.source_synced_at
        FROM ipos i JOIN companies c ON c.id = i.company_id
        WHERE i.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("IPO not found".into()))?;

    let data_sources = sqlx::query_as::<_, DataSourceMeta>(
        r#"
        SELECT provider, category, official, api_url, refresh_frequency_secs,
               licensing, rate_limits, notes
        FROM data_sources
        WHERE enabled = TRUE AND category <> 'gmp'
        ORDER BY category
        "#,
    )
    .fetch_all(state.db())
    .await?;

    let sub_updated: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar(r#"SELECT updated_at FROM ipo_subscription_snapshots WHERE ipo_id = $1"#)
            .bind(id)
            .fetch_optional(state.db())
            .await?;

    let fin_updated: Option<chrono::DateTime<Utc>> = sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
        r#"SELECT MAX(updated_at) FROM ipo_financials WHERE ipo_id = $1"#,
    )
    .bind(id)
    .fetch_one(state.db())
    .await?;

    let score_at: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar(r#"SELECT computed_at FROM ipo_scores WHERE ipo_id = $1"#)
            .bind(id)
            .fetch_optional(state.db())
            .await?;

    Ok(Json(ApiResponse::ok(IntelMetaResponse {
        ipo_id: meta.id,
        company_name: meta.company_name,
        data_sources,
        freshness: DataSourceFreshness {
            ipo_synced_at: meta.source_synced_at,
            subscription_updated_at: sub_updated,
            financials_updated_at: fin_updated,
            score_computed_at: score_at,
        },
    })))
}
