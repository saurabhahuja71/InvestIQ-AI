use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct IpoListItem {
    pub id: Uuid,
    pub company_name: String,
    pub symbol: Option<String>,
    pub board: String,
    pub status: String,
    pub price_band_low: Option<Decimal>,
    pub price_band_high: Option<Decimal>,
    pub issue_price: Option<Decimal>,
    pub lot_size: Option<i32>,
    pub open_date: Option<NaiveDate>,
    pub close_date: Option<NaiveDate>,
    pub listing_date: Option<NaiveDate>,
    pub subscription_total: Option<Decimal>,
    pub gmp_value: Option<Decimal>,
    pub gmp_unofficial: bool,
    pub logo_url: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct IpoDetailRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub symbol: Option<String>,
    pub sector: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub board: String,
    pub status: String,
    pub issue_type: Option<String>,
    pub price_band_low: Option<Decimal>,
    pub price_band_high: Option<Decimal>,
    pub issue_price: Option<Decimal>,
    pub lot_size: Option<i32>,
    pub issue_size_cr: Option<Decimal>,
    pub open_date: Option<NaiveDate>,
    pub close_date: Option<NaiveDate>,
    pub allotment_date: Option<NaiveDate>,
    pub refund_date: Option<NaiveDate>,
    pub listing_date: Option<NaiveDate>,
    pub exchange: Option<String>,
    pub registrar: Option<String>,
    pub subscription_total: Option<Decimal>,
    pub subscription_retail: Option<Decimal>,
    pub subscription_qib: Option<Decimal>,
    pub subscription_nii: Option<Decimal>,
    pub listing_open: Option<Decimal>,
    pub listing_close: Option<Decimal>,
    pub gmp_value: Option<Decimal>,
    pub gmp_updated_at: Option<DateTime<Utc>>,
    pub gmp_disclaimer: String,
    pub financials: serde_json::Value,
    pub pros: serde_json::Value,
    pub risks: serde_json::Value,
    pub ai_summary: Option<String>,
    pub drhp_url: Option<String>,
    pub rhp_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GmpInfo {
    pub value: Option<Decimal>,
    pub updated_at: Option<DateTime<Utc>>,
    pub unofficial: bool,
    pub disclaimer: String,
}

#[derive(Debug, Serialize)]
pub struct IpoDetail {
    #[serde(flatten)]
    pub row: IpoDetailResponse,
    pub gmp: GmpInfo,
}

#[derive(Debug, Serialize)]
pub struct IpoDetailResponse {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub symbol: Option<String>,
    pub sector: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub board: String,
    pub status: String,
    pub issue_type: Option<String>,
    pub price_band_low: Option<Decimal>,
    pub price_band_high: Option<Decimal>,
    pub issue_price: Option<Decimal>,
    pub lot_size: Option<i32>,
    pub issue_size_cr: Option<Decimal>,
    pub open_date: Option<NaiveDate>,
    pub close_date: Option<NaiveDate>,
    pub allotment_date: Option<NaiveDate>,
    pub refund_date: Option<NaiveDate>,
    pub listing_date: Option<NaiveDate>,
    pub exchange: Option<String>,
    pub registrar: Option<String>,
    pub subscription_total: Option<Decimal>,
    pub subscription_retail: Option<Decimal>,
    pub subscription_qib: Option<Decimal>,
    pub subscription_nii: Option<Decimal>,
    pub listing_open: Option<Decimal>,
    pub listing_close: Option<Decimal>,
    pub financials: serde_json::Value,
    pub pros: serde_json::Value,
    pub risks: serde_json::Value,
    pub ai_summary: Option<String>,
    pub drhp_url: Option<String>,
    pub rhp_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IpoQuery {
    pub status: Option<String>,
    pub board: Option<String>,
    pub q: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AllotmentRequest {
    pub pan_last4: Option<String>,
    pub application_number: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AllotmentResult {
    pub status: String,
    pub shares: Option<i32>,
    pub message: String,
}
