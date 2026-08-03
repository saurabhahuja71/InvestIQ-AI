use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::middleware::AuthUser;
use crate::modules::common::ApiResponse;
use crate::modules::portfolio::analytics::{
    cagr, decimal_to_f64, years_between, xirr, CashFlow,
};
use crate::modules::portfolio::models::*;
use crate::state::AppState;

const HOLDING_SELECT: &str = r#"
    SELECT id, portfolio_id, asset_class::text, symbol, name, isin,
           quantity, avg_cost, currency, sector, exchange,
           last_price, prev_close, price_as_of
    FROM holdings
"#;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_portfolios).post(create_portfolio))
        .route("/{id}", get(dashboard))
        .route("/{id}/holdings", get(list_holdings).post(add_holding))
        .route(
            "/{id}/holdings/{hid}/price",
            post(update_holding_price),
        )
        .route("/{id}/transactions", get(list_txns).post(add_txn))
        .route("/{id}/analytics", get(analytics))
        .route("/{id}/ai-review", post(ai_review))
}

async fn ensure_owner(
    state: &AppState,
    user_id: Uuid,
    portfolio_id: Uuid,
) -> AppResult<PortfolioRow> {
    sqlx::query_as::<_, PortfolioRow>(
        r#"
        SELECT id, user_id, name, base_currency, is_default, created_at
        FROM portfolios WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(portfolio_id)
    .bind(user_id)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("portfolio not found".into()))
}

async fn list_portfolios(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<Vec<PortfolioRow>>>> {
    let rows = sqlx::query_as::<_, PortfolioRow>(
        r#"
        SELECT id, user_id, name, base_currency, is_default, created_at
        FROM portfolios WHERE user_id = $1 ORDER BY is_default DESC, created_at
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[derive(serde::Deserialize)]
pub struct CreatePortfolio {
    pub name: String,
    pub base_currency: Option<String>,
}

async fn create_portfolio(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreatePortfolio>,
) -> AppResult<Json<ApiResponse<PortfolioRow>>> {
    let row = sqlx::query_as::<_, PortfolioRow>(
        r#"
        INSERT INTO portfolios (user_id, name, base_currency, is_default)
        VALUES ($1, $2, COALESCE($3, 'INR'), false)
        RETURNING id, user_id, name, base_currency, is_default, created_at
        "#,
    )
    .bind(user.user_id)
    .bind(&body.name)
    .bind(&body.base_currency)
    .fetch_one(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(row)))
}

async fn list_holdings(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<HoldingRow>>>> {
    ensure_owner(&state, user.user_id, id).await?;
    let sql = format!("{HOLDING_SELECT} WHERE portfolio_id = $1 ORDER BY name");
    let rows = sqlx::query_as::<_, HoldingRow>(&sql)
        .bind(id)
        .fetch_all(state.db())
        .await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn add_holding(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateHolding>,
) -> AppResult<Json<ApiResponse<HoldingRow>>> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    ensure_owner(&state, user.user_id, id).await?;

    let last = body.last_price.unwrap_or(body.avg_cost);
    let prev = last * Decimal::new(995, 3); // 0.995x as prior close default

    let row = sqlx::query_as::<_, HoldingRow>(
        r#"
        INSERT INTO holdings (
            portfolio_id, asset_class, symbol, name, isin, quantity, avg_cost,
            currency, sector, exchange, last_price, prev_close, price_as_of
        )
        VALUES (
            $1, $2::asset_class, $3, $4, $5, $6, $7,
            COALESCE($8, 'INR'), $9, $10, $11, $12, NOW()
        )
        RETURNING id, portfolio_id, asset_class::text, symbol, name, isin,
                  quantity, avg_cost, currency, sector, exchange,
                  last_price, prev_close, price_as_of
        "#,
    )
    .bind(id)
    .bind(&body.asset_class)
    .bind(&body.symbol)
    .bind(&body.name)
    .bind(&body.isin)
    .bind(body.quantity)
    .bind(body.avg_cost)
    .bind(&body.currency)
    .bind(&body.sector)
    .bind(&body.exchange)
    .bind(last)
    .bind(prev)
    .fetch_one(state.db())
    .await?;

    if let Some(sym) = &body.symbol {
        let _ = sqlx::query(
            r#"
            INSERT INTO price_snapshots (symbol, asset_class, price, currency, as_of)
            VALUES ($1, $2::asset_class, $3, COALESCE($4, 'INR'), NOW())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(sym)
        .bind(&body.asset_class)
        .bind(last)
        .bind(&body.currency)
        .execute(state.db())
        .await;
    }

    Ok(Json(ApiResponse::ok(row)))
}

async fn update_holding_price(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, hid)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateHoldingPrice>,
) -> AppResult<Json<ApiResponse<HoldingRow>>> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    ensure_owner(&state, user.user_id, id).await?;

    let row = sqlx::query_as::<_, HoldingRow>(
        r#"
        UPDATE holdings SET
            prev_close = COALESCE($3, last_price, avg_cost),
            last_price = $2,
            price_as_of = NOW(),
            updated_at = NOW()
        WHERE id = $1 AND portfolio_id = $4
        RETURNING id, portfolio_id, asset_class::text, symbol, name, isin,
                  quantity, avg_cost, currency, sector, exchange,
                  last_price, prev_close, price_as_of
        "#,
    )
    .bind(hid)
    .bind(body.last_price)
    .bind(body.prev_close)
    .bind(id)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("holding not found".into()))?;

    Ok(Json(ApiResponse::ok(row)))
}

async fn list_txns(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<TransactionRow>>>> {
    ensure_owner(&state, user.user_id, id).await?;
    let rows = sqlx::query_as::<_, TransactionRow>(
        r#"
        SELECT id, portfolio_id, holding_id, txn_type::text, trade_date,
               quantity, price, fees, amount, currency, notes, created_at
        FROM transactions WHERE portfolio_id = $1
        ORDER BY trade_date DESC, created_at DESC
        LIMIT 500
        "#,
    )
    .bind(id)
    .fetch_all(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn add_txn(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateTransaction>,
) -> AppResult<Json<ApiResponse<TransactionRow>>> {
    ensure_owner(&state, user.user_id, id).await?;

    let mut holding_id = body.holding_id;
    if holding_id.is_none() && body.txn_type == "buy" {
        if let (Some(name), Some(qty), Some(price)) = (&body.name, body.quantity, body.price) {
            let ac = body.asset_class.as_deref().unwrap_or("stock");
            let prev = price * Decimal::new(995, 3);
            let h = sqlx::query_as::<_, HoldingRow>(
                r#"
                INSERT INTO holdings (
                    portfolio_id, asset_class, symbol, name, quantity, avg_cost, currency,
                    last_price, prev_close, price_as_of
                )
                VALUES (
                    $1, $2::asset_class, $3, $4, $5, $6, COALESCE($7, 'INR'),
                    $6, $8, NOW()
                )
                RETURNING id, portfolio_id, asset_class::text, symbol, name, isin,
                          quantity, avg_cost, currency, sector, exchange,
                          last_price, prev_close, price_as_of
                "#,
            )
            .bind(id)
            .bind(ac)
            .bind(&body.symbol)
            .bind(name)
            .bind(qty)
            .bind(price)
            .bind(&body.currency)
            .bind(prev)
            .fetch_one(state.db())
            .await?;
            holding_id = Some(h.id);
        }
    } else if let (Some(hid), Some(qty), Some(price)) =
        (holding_id, body.quantity, body.price)
    {
        // Roll avg cost / qty for buy/sell
        if let Ok(Some(h)) = sqlx::query_as::<_, HoldingRow>(&format!(
            "{HOLDING_SELECT} WHERE id = $1 AND portfolio_id = $2"
        ))
        .bind(hid)
        .bind(id)
        .fetch_optional(state.db())
        .await
        {
            let (new_qty, new_avg) = match body.txn_type.as_str() {
                "buy" => {
                    let cost = h.avg_cost * h.quantity + price * qty;
                    let q = h.quantity + qty;
                    let avg = if q > Decimal::ZERO {
                        cost / q
                    } else {
                        price
                    };
                    (q, avg)
                }
                "sell" => {
                    let q = (h.quantity - qty).max(Decimal::ZERO);
                    (q, h.avg_cost)
                }
                _ => (h.quantity, h.avg_cost),
            };
            let _ = sqlx::query(
                r#"
                UPDATE holdings SET quantity = $2, avg_cost = $3,
                    last_price = COALESCE($4, last_price),
                    prev_close = COALESCE(last_price, prev_close),
                    price_as_of = NOW(), updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(hid)
            .bind(new_qty)
            .bind(new_avg)
            .bind(price)
            .execute(state.db())
            .await;
        }
    }

    let row = sqlx::query_as::<_, TransactionRow>(
        r#"
        INSERT INTO transactions (
            portfolio_id, holding_id, txn_type, trade_date, quantity, price, fees, amount, currency, notes
        )
        VALUES (
            $1, $2, $3::txn_type, $4, $5, $6, COALESCE($7, 0), $8, COALESCE($9, 'INR'), $10
        )
        RETURNING id, portfolio_id, holding_id, txn_type::text, trade_date,
                  quantity, price, fees, amount, currency, notes, created_at
        "#,
    )
    .bind(id)
    .bind(holding_id)
    .bind(&body.txn_type)
    .bind(body.trade_date)
    .bind(body.quantity)
    .bind(body.price)
    .bind(body.fees)
    .bind(body.amount)
    .bind(&body.currency)
    .bind(&body.notes)
    .fetch_one(state.db())
    .await?;

    Ok(Json(ApiResponse::ok(row)))
}

pub async fn compute_analytics(
    state: &AppState,
    portfolio_id: Uuid,
) -> AppResult<PortfolioAnalytics> {
    let sql = format!("{HOLDING_SELECT} WHERE portfolio_id = $1");
    let holdings = sqlx::query_as::<_, HoldingRow>(&sql)
        .bind(portfolio_id)
        .fetch_all(state.db())
        .await?;

    let mut total_value = Decimal::ZERO;
    let mut total_cost = Decimal::ZERO;
    let mut today_pnl = Decimal::ZERO;
    let mut by_class: std::collections::HashMap<String, Decimal> = Default::default();
    let mut by_sector: std::collections::HashMap<String, Decimal> = Default::default();

    for h in &holdings {
        let px = h.last_price.unwrap_or(h.avg_cost);
        let prev = h.prev_close.unwrap_or(px);
        let value = h.quantity * px;
        let cost = h.quantity * h.avg_cost;
        total_value += value;
        total_cost += cost;
        today_pnl += h.quantity * (px - prev);
        *by_class.entry(h.asset_class.clone()).or_default() += value;
        let sector = h.sector.clone().unwrap_or_else(|| "Unknown".into());
        *by_sector.entry(sector).or_default() += value;
    }

    let unrealized_pnl = total_value - total_cost;
    let overall_return_pct = if total_cost > Decimal::ZERO {
        decimal_to_f64(unrealized_pnl / total_cost * Decimal::from(100))
    } else {
        0.0
    };

    let yesterday_value = total_value - today_pnl;
    let today_pnl_pct = if yesterday_value > Decimal::ZERO {
        decimal_to_f64(today_pnl / yesterday_value * Decimal::from(100))
    } else {
        0.0
    };

    let to_slices = |map: std::collections::HashMap<String, Decimal>, total: Decimal| {
        map.into_iter()
            .map(|(key, value)| AllocationSlice {
                pct: if total > Decimal::ZERO {
                    decimal_to_f64(value / total * Decimal::from(100))
                } else {
                    0.0
                },
                key,
                value,
            })
            .collect::<Vec<_>>()
    };

    let txns = sqlx::query_as::<_, TransactionRow>(
        r#"
        SELECT id, portfolio_id, holding_id, txn_type::text, trade_date,
               quantity, price, fees, amount, currency, notes, created_at
        FROM transactions WHERE portfolio_id = $1 ORDER BY trade_date
        "#,
    )
    .bind(portfolio_id)
    .fetch_all(state.db())
    .await?;

    let mut flows: Vec<CashFlow> = txns
        .iter()
        .map(|t| CashFlow {
            date: t.trade_date,
            amount: decimal_to_f64(t.amount),
        })
        .collect();

    if total_value > Decimal::ZERO {
        flows.push(CashFlow {
            date: Utc::now().date_naive(),
            amount: decimal_to_f64(total_value),
        });
    }

    let xirr_v = xirr(&flows, 0.1);

    let cagr_v = if let Some(first) = txns.first() {
        let years = years_between(first.trade_date, Utc::now().date_naive());
        // invested capital as sum of negative cash flows magnitude
        let invested: f64 = txns
            .iter()
            .filter(|t| t.amount < Decimal::ZERO)
            .map(|t| decimal_to_f64(t.amount).abs())
            .sum();
        if invested > 0.0 && years > 0.0 {
            cagr(invested, decimal_to_f64(total_value), years)
        } else if total_cost > Decimal::ZERO && years > 0.0 {
            cagr(
                decimal_to_f64(total_cost),
                decimal_to_f64(total_value),
                years,
            )
        } else {
            None
        }
    } else if total_cost > Decimal::ZERO && total_value > Decimal::ZERO {
        // No transactions: use portfolio created_at via holdings price_as_of fallback 1y
        cagr(
            decimal_to_f64(total_cost),
            decimal_to_f64(total_value),
            1.0,
        )
    } else {
        None
    };

    Ok(PortfolioAnalytics {
        total_value,
        total_cost,
        unrealized_pnl,
        today_pnl,
        today_pnl_pct,
        overall_return_pct,
        xirr: xirr_v,
        cagr: cagr_v,
        allocation_by_class: to_slices(by_class, total_value),
        allocation_by_sector: to_slices(by_sector, total_value),
    })
}

async fn analytics(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<PortfolioAnalytics>>> {
    ensure_owner(&state, user.user_id, id).await?;
    let a = compute_analytics(&state, id).await?;
    Ok(Json(ApiResponse::ok(a)))
}

async fn dashboard(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<PortfolioDashboard>>> {
    let portfolio = ensure_owner(&state, user.user_id, id).await?;
    let sql = format!("{HOLDING_SELECT} WHERE portfolio_id = $1 ORDER BY name");
    let holdings = sqlx::query_as::<_, HoldingRow>(&sql)
        .bind(id)
        .fetch_all(state.db())
        .await?;
    let analytics = compute_analytics(&state, id).await?;
    Ok(Json(ApiResponse::ok(PortfolioDashboard {
        portfolio,
        analytics,
        holdings,
    })))
}

async fn ai_review(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    ensure_owner(&state, user.user_id, id).await?;
    tracing::info!(email = %user.email, portfolio = %id, "portfolio ai review");
    let a = compute_analytics(&state, id).await?;
    let ctx = serde_json::to_string(&a).unwrap_or_default();
    let reply = state
        .ai()
        .chat(
            vec![crate::infra::ai::ChatMessage {
                role: "user".into(),
                content: "Review this portfolio for concentration risk, diversification, and educational rebalancing ideas. Do not guarantee returns.".into(),
            }],
            Some(ctx),
        )
        .await?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "review": reply,
        "disclaimer": crate::infra::ai::INVESTMENT_DISCLAIMER
    }))))
}
