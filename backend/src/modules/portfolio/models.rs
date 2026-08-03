use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
pub struct PortfolioRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub base_currency: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct HoldingRow {
    pub id: Uuid,
    pub portfolio_id: Uuid,
    pub asset_class: String,
    pub symbol: Option<String>,
    pub name: String,
    pub isin: Option<String>,
    pub quantity: Decimal,
    pub avg_cost: Decimal,
    pub currency: String,
    pub sector: Option<String>,
    pub exchange: Option<String>,
    pub last_price: Option<Decimal>,
    pub prev_close: Option<Decimal>,
    pub price_as_of: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateHolding {
    pub asset_class: String,
    pub symbol: Option<String>,
    #[validate(length(min = 1))]
    pub name: String,
    pub isin: Option<String>,
    pub quantity: Decimal,
    pub avg_cost: Decimal,
    pub currency: Option<String>,
    pub sector: Option<String>,
    pub exchange: Option<String>,
    /// Optional current market price; defaults to avg_cost
    pub last_price: Option<Decimal>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateHoldingPrice {
    pub last_price: Decimal,
    pub prev_close: Option<Decimal>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTransaction {
    pub holding_id: Option<Uuid>,
    pub txn_type: String,
    pub trade_date: NaiveDate,
    pub quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub fees: Option<Decimal>,
    pub amount: Decimal,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub asset_class: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TransactionRow {
    pub id: Uuid,
    pub portfolio_id: Uuid,
    pub holding_id: Option<Uuid>,
    pub txn_type: String,
    pub trade_date: NaiveDate,
    pub quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub fees: Option<Decimal>,
    pub amount: Decimal,
    pub currency: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AllocationSlice {
    pub key: String,
    pub value: Decimal,
    pub pct: f64,
}

#[derive(Debug, Serialize)]
pub struct PortfolioAnalytics {
    pub total_value: Decimal,
    pub total_cost: Decimal,
    pub unrealized_pnl: Decimal,
    pub today_pnl: Decimal,
    pub today_pnl_pct: f64,
    pub overall_return_pct: f64,
    pub xirr: Option<f64>,
    pub cagr: Option<f64>,
    pub allocation_by_class: Vec<AllocationSlice>,
    pub allocation_by_sector: Vec<AllocationSlice>,
}

#[derive(Debug, Serialize)]
pub struct PortfolioDashboard {
    pub portfolio: PortfolioRow,
    pub analytics: PortfolioAnalytics,
    pub holdings: Vec<HoldingRow>,
}
