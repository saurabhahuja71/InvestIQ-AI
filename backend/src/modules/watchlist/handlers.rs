//! Authenticated IPO watchlist APIs (Milestone 3).
//!
//! - GET    /watchlist
//! - POST   /watchlist           body: { "ipo_id": "<uuid>" }
//! - DELETE /watchlist/{id}      id = ipo_id

use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::middleware::AuthUser;
use crate::modules::common::ApiResponse;
use crate::modules::ipo::models::IpoListItem;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_watchlist).post(add_to_watchlist))
        .route("/{id}", delete(remove_from_watchlist))
}

#[derive(Debug, Deserialize)]
pub struct AddWatchBody {
    pub ipo_id: Uuid,
}

async fn list_watchlist(
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
        ORDER BY w.created_at DESC, i.open_date DESC NULLS LAST
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn add_to_watchlist(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<AddWatchBody>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let exists: bool = sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM ipos WHERE id = $1)"#)
        .bind(body.ipo_id)
        .fetch_one(state.db())
        .await?;
    if !exists {
        return Err(AppError::NotFound("IPO not found".into()));
    }

    sqlx::query(
        r#"
        INSERT INTO ipo_watchlist (user_id, ipo_id) VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(user.user_id)
    .bind(body.ipo_id)
    .execute(state.db())
    .await?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "ipo_id": body.ipo_id,
        "watched": true
    }))))
}

async fn remove_from_watchlist(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    let res = sqlx::query(r#"DELETE FROM ipo_watchlist WHERE user_id = $1 AND ipo_id = $2"#)
        .bind(user.user_id)
        .bind(id)
        .execute(state.db())
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("watchlist entry not found".into()));
    }
    Ok(Json(ApiResponse::ok("removed")))
}
