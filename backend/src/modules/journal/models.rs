use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
pub struct TradeRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub strategy_name: Option<String>,
    pub entry_price: Decimal,
    pub exit_price: Option<Decimal>,
    pub quantity: Decimal,
    pub entry_at: DateTime<Utc>,
    pub exit_at: Option<DateTime<Utc>>,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub risk_reward: Option<Decimal>,
    pub fees: Option<Decimal>,
    pub pnl: Option<Decimal>,
    pub emotion_before: Option<String>,
    pub emotion_after: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTrade {
    #[validate(length(min = 1))]
    pub symbol: String,
    pub side: String,
    pub strategy_name: Option<String>,
    pub entry_price: Decimal,
    pub exit_price: Option<Decimal>,
    pub quantity: Decimal,
    pub entry_at: DateTime<Utc>,
    pub exit_at: Option<DateTime<Utc>>,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub risk_reward: Option<Decimal>,
    pub fees: Option<Decimal>,
    pub pnl: Option<Decimal>,
    pub emotion_before: Option<String>,
    pub emotion_after: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct JournalAnalytics {
    pub total_trades: i64,
    pub closed_trades: i64,
    pub win_rate: f64,
    pub average_profit: Decimal,
    pub average_loss: Decimal,
    pub largest_winner: Decimal,
    pub largest_loser: Decimal,
    pub total_pnl: Decimal,
    pub profit_factor: Option<f64>,
}
