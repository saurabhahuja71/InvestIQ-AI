use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use rust_decimal::Decimal;
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::middleware::AuthUser;
use crate::modules::common::ApiResponse;
use crate::modules::journal::models::*;
use crate::modules::portfolio::analytics::decimal_to_f64;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trades", get(list_trades).post(create_trade))
        .route("/trades/{id}", get(get_trade).patch(update_trade).delete(delete_trade))
        .route("/analytics", get(analytics))
        .route("/ai/mistakes", post(ai_mistakes))
}

fn compute_pnl(side: &str, entry: Decimal, exit: Option<Decimal>, qty: Decimal, fees: Decimal) -> Option<Decimal> {
    let exit = exit?;
    let gross = if side == "long" {
        (exit - entry) * qty
    } else {
        (entry - exit) * qty
    };
    Some(gross - fees)
}

async fn list_trades(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<Vec<TradeRow>>>> {
    let rows = sqlx::query_as::<_, TradeRow>(
        r#"
        SELECT id, user_id, symbol, side::text, strategy_name, entry_price, exit_price, quantity,
               entry_at, exit_at, stop_loss, take_profit, risk_reward, fees, pnl,
               emotion_before::text, emotion_after::text, notes, tags, created_at
        FROM journal_trades
        WHERE user_id = $1 AND deleted_at IS NULL
        ORDER BY entry_at DESC
        LIMIT 500
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn create_trade(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateTrade>,
) -> AppResult<Json<ApiResponse<TradeRow>>> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let fees = body.fees.unwrap_or(Decimal::ZERO);
    let pnl = body
        .pnl
        .or_else(|| compute_pnl(&body.side, body.entry_price, body.exit_price, body.quantity, fees));
    let tags = body.tags.unwrap_or_default();

    let row = sqlx::query_as::<_, TradeRow>(
        r#"
        INSERT INTO journal_trades (
            user_id, symbol, side, strategy_name, entry_price, exit_price, quantity,
            entry_at, exit_at, stop_loss, take_profit, risk_reward, fees, pnl,
            emotion_before, emotion_after, notes, tags
        ) VALUES (
            $1, $2, $3::trade_side, $4, $5, $6, $7,
            $8, $9, $10, $11, $12, $13, $14,
            $15::emotion_tag, $16::emotion_tag, $17, $18
        )
        RETURNING id, user_id, symbol, side::text, strategy_name, entry_price, exit_price, quantity,
                  entry_at, exit_at, stop_loss, take_profit, risk_reward, fees, pnl,
                  emotion_before::text, emotion_after::text, notes, tags, created_at
        "#,
    )
    .bind(user.user_id)
    .bind(&body.symbol)
    .bind(&body.side)
    .bind(&body.strategy_name)
    .bind(body.entry_price)
    .bind(body.exit_price)
    .bind(body.quantity)
    .bind(body.entry_at)
    .bind(body.exit_at)
    .bind(body.stop_loss)
    .bind(body.take_profit)
    .bind(body.risk_reward)
    .bind(fees)
    .bind(pnl)
    .bind(&body.emotion_before)
    .bind(&body.emotion_after)
    .bind(&body.notes)
    .bind(&tags)
    .fetch_one(state.db())
    .await?;

    Ok(Json(ApiResponse::ok(row)))
}

async fn get_trade(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<TradeRow>>> {
    let row = sqlx::query_as::<_, TradeRow>(
        r#"
        SELECT id, user_id, symbol, side::text, strategy_name, entry_price, exit_price, quantity,
               entry_at, exit_at, stop_loss, take_profit, risk_reward, fees, pnl,
               emotion_before::text, emotion_after::text, notes, tags, created_at
        FROM journal_trades WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .bind(user.user_id)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("trade not found".into()))?;
    Ok(Json(ApiResponse::ok(row)))
}

async fn update_trade(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateTrade>,
) -> AppResult<Json<ApiResponse<TradeRow>>> {
    let fees = body.fees.unwrap_or(Decimal::ZERO);
    let pnl = body
        .pnl
        .or_else(|| compute_pnl(&body.side, body.entry_price, body.exit_price, body.quantity, fees));
    let tags = body.tags.clone().unwrap_or_default();

    let row = sqlx::query_as::<_, TradeRow>(
        r#"
        UPDATE journal_trades SET
            symbol = $3, side = $4::trade_side, strategy_name = $5,
            entry_price = $6, exit_price = $7, quantity = $8,
            entry_at = $9, exit_at = $10, stop_loss = $11, take_profit = $12,
            risk_reward = $13, fees = $14, pnl = $15,
            emotion_before = $16::emotion_tag, emotion_after = $17::emotion_tag,
            notes = $18, tags = $19, updated_at = NOW()
        WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
        RETURNING id, user_id, symbol, side::text, strategy_name, entry_price, exit_price, quantity,
                  entry_at, exit_at, stop_loss, take_profit, risk_reward, fees, pnl,
                  emotion_before::text, emotion_after::text, notes, tags, created_at
        "#,
    )
    .bind(id)
    .bind(user.user_id)
    .bind(&body.symbol)
    .bind(&body.side)
    .bind(&body.strategy_name)
    .bind(body.entry_price)
    .bind(body.exit_price)
    .bind(body.quantity)
    .bind(body.entry_at)
    .bind(body.exit_at)
    .bind(body.stop_loss)
    .bind(body.take_profit)
    .bind(body.risk_reward)
    .bind(fees)
    .bind(pnl)
    .bind(&body.emotion_before)
    .bind(&body.emotion_after)
    .bind(&body.notes)
    .bind(&tags)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("trade not found".into()))?;

    Ok(Json(ApiResponse::ok(row)))
}

async fn delete_trade(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    let res = sqlx::query(
        r#"UPDATE journal_trades SET deleted_at = NOW() WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL"#,
    )
    .bind(id)
    .bind(user.user_id)
    .execute(state.db())
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("trade not found".into()));
    }
    Ok(Json(ApiResponse::ok("deleted")))
}

async fn analytics(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<JournalAnalytics>>> {
    let trades = sqlx::query_as::<_, TradeRow>(
        r#"
        SELECT id, user_id, symbol, side::text, strategy_name, entry_price, exit_price, quantity,
               entry_at, exit_at, stop_loss, take_profit, risk_reward, fees, pnl,
               emotion_before::text, emotion_after::text, notes, tags, created_at
        FROM journal_trades WHERE user_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;

    let total_trades = trades.len() as i64;
    let closed: Vec<_> = trades.iter().filter(|t| t.pnl.is_some()).collect();
    let closed_trades = closed.len() as i64;

    let wins: Vec<Decimal> = closed
        .iter()
        .filter_map(|t| t.pnl)
        .filter(|p| *p > Decimal::ZERO)
        .collect();
    let losses: Vec<Decimal> = closed
        .iter()
        .filter_map(|t| t.pnl)
        .filter(|p| *p < Decimal::ZERO)
        .collect();

    let win_rate = if closed_trades > 0 {
        wins.len() as f64 / closed_trades as f64 * 100.0
    } else {
        0.0
    };

    let average_profit = if wins.is_empty() {
        Decimal::ZERO
    } else {
        wins.iter().copied().sum::<Decimal>() / Decimal::from(wins.len() as i64)
    };
    let average_loss = if losses.is_empty() {
        Decimal::ZERO
    } else {
        losses.iter().copied().sum::<Decimal>() / Decimal::from(losses.len() as i64)
    };

    let largest_winner = wins.into_iter().max().unwrap_or(Decimal::ZERO);
    let largest_loser = losses.into_iter().min().unwrap_or(Decimal::ZERO);
    let total_pnl = closed.iter().filter_map(|t| t.pnl).sum();

    let gross_profit: f64 = closed
        .iter()
        .filter_map(|t| t.pnl)
        .filter(|p| *p > Decimal::ZERO)
        .map(decimal_to_f64)
        .sum();
    let gross_loss: f64 = closed
        .iter()
        .filter_map(|t| t.pnl)
        .filter(|p| *p < Decimal::ZERO)
        .map(|p| decimal_to_f64(p).abs())
        .sum();
    let profit_factor = if gross_loss > 0.0 {
        Some(gross_profit / gross_loss)
    } else {
        None
    };

    Ok(Json(ApiResponse::ok(JournalAnalytics {
        total_trades,
        closed_trades,
        win_rate,
        average_profit,
        average_loss,
        largest_winner,
        largest_loser,
        total_pnl,
        profit_factor,
    })))
}

async fn ai_mistakes(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let trades = sqlx::query_as::<_, TradeRow>(
        r#"
        SELECT id, user_id, symbol, side::text, strategy_name, entry_price, exit_price, quantity,
               entry_at, exit_at, stop_loss, take_profit, risk_reward, fees, pnl,
               emotion_before::text, emotion_after::text, notes, tags, created_at
        FROM journal_trades
        WHERE user_id = $1 AND deleted_at IS NULL
        ORDER BY entry_at DESC LIMIT 50
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;

    let ctx = serde_json::to_string(&trades).unwrap_or_default();
    let reply = state
        .ai()
        .chat(
            vec![crate::infra::ai::ChatMessage {
                role: "user".into(),
                content: "Analyze these journal trades for common mistakes (FOMO, revenge trading, poor R:R, skipping stops). Be educational, not judgmental. No guaranteed returns.".into(),
            }],
            Some(ctx),
        )
        .await?;

    sqlx::query(
        r#"
        INSERT INTO journal_ai_insights (user_id, insight_type, content)
        VALUES ($1, 'mistake', $2)
        "#,
    )
    .bind(user.user_id)
    .bind(&reply)
    .execute(state.db())
    .await?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "insights": reply,
        "disclaimer": crate::infra::ai::INVESTMENT_DISCLAIMER
    }))))
}
