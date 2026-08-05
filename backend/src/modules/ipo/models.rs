use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
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
    pub min_investment: Option<Decimal>,
    pub open_date: Option<NaiveDate>,
    pub close_date: Option<NaiveDate>,
    pub listing_date: Option<NaiveDate>,
    pub exchange: Option<String>,
    pub subscription_total: Option<Decimal>,
    pub gmp_value: Option<Decimal>,
    pub gmp_unofficial: bool,
    pub logo_url: Option<String>,
    pub source: Option<String>,
    pub source_synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct IpoDetailRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub symbol: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub website: Option<String>,
    pub board: String,
    pub status: String,
    pub issue_type: Option<String>,
    pub price_band_low: Option<Decimal>,
    pub price_band_high: Option<Decimal>,
    pub issue_price: Option<Decimal>,
    pub lot_size: Option<i32>,
    pub face_value: Option<Decimal>,
    pub min_investment: Option<Decimal>,
    pub issue_size_cr: Option<Decimal>,
    pub shares_offered: Option<i64>,
    pub open_date: Option<NaiveDate>,
    pub close_date: Option<NaiveDate>,
    pub allotment_date: Option<NaiveDate>,
    pub refund_date: Option<NaiveDate>,
    pub listing_date: Option<NaiveDate>,
    pub exchange: Option<String>,
    pub registrar: Option<String>,
    pub lead_managers: serde_json::Value,
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
    pub issue_info: serde_json::Value,
    pub source: Option<String>,
    pub source_synced_at: Option<DateTime<Utc>>,
    pub nse_symbol: Option<String>,
    pub nse_series: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GmpInfo {
    pub value: Option<Decimal>,
    pub updated_at: Option<DateTime<Utc>>,
    pub unofficial: bool,
    pub disclaimer: String,
    /// Always true for NSE-sourced data — NSE does not publish GMP.
    pub available: bool,
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
    pub industry: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub website: Option<String>,
    pub board: String,
    pub status: String,
    pub issue_type: Option<String>,
    pub price_band_low: Option<Decimal>,
    pub price_band_high: Option<Decimal>,
    pub issue_price: Option<Decimal>,
    pub lot_size: Option<i32>,
    pub face_value: Option<Decimal>,
    pub min_investment: Option<Decimal>,
    pub issue_size_cr: Option<Decimal>,
    pub shares_offered: Option<i64>,
    pub open_date: Option<NaiveDate>,
    pub close_date: Option<NaiveDate>,
    pub allotment_date: Option<NaiveDate>,
    pub refund_date: Option<NaiveDate>,
    pub listing_date: Option<NaiveDate>,
    pub exchange: Option<String>,
    pub registrar: Option<String>,
    pub lead_managers: serde_json::Value,
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
    pub prospectus_url: Option<String>,
    pub issue_info: serde_json::Value,
    pub source: Option<String>,
    pub source_synced_at: Option<DateTime<Utc>>,
}

impl From<IpoDetailRow> for IpoDetail {
    fn from(row: IpoDetailRow) -> Self {
        let gmp_available = row.gmp_value.is_some();
        let prospectus = row.rhp_url.clone().or_else(|| row.drhp_url.clone());
        Self {
            gmp: GmpInfo {
                value: row.gmp_value,
                updated_at: row.gmp_updated_at,
                unofficial: true,
                disclaimer: row.gmp_disclaimer.clone(),
                available: gmp_available,
            },
            row: IpoDetailResponse {
                id: row.id,
                company_id: row.company_id,
                company_name: row.company_name,
                symbol: row.symbol,
                sector: row.sector,
                industry: row.industry,
                description: row.description,
                logo_url: row.logo_url,
                website: row.website,
                board: row.board,
                status: row.status,
                issue_type: row.issue_type,
                price_band_low: row.price_band_low,
                price_band_high: row.price_band_high,
                issue_price: row.issue_price,
                lot_size: row.lot_size,
                face_value: row.face_value,
                min_investment: row.min_investment,
                issue_size_cr: row.issue_size_cr,
                shares_offered: row.shares_offered,
                open_date: row.open_date,
                close_date: row.close_date,
                allotment_date: row.allotment_date,
                refund_date: row.refund_date,
                listing_date: row.listing_date,
                exchange: row.exchange,
                registrar: row.registrar,
                lead_managers: row.lead_managers,
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
                prospectus_url: prospectus,
                issue_info: row.issue_info,
                source: row.source,
                source_synced_at: row.source_synced_at,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct IpoQuery {
    pub status: Option<String>,
    pub board: Option<String>,
    pub q: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    /// When true, trigger a best-effort NSE sync before listing (rate-limited by sync lock).
    pub refresh: Option<bool>,
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
