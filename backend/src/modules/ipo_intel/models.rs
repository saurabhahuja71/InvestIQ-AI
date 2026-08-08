//! Response DTOs for IPO Intelligence endpoints.
//!
//! Data-quality contract (Milestone 4 revision):
//! every financial and subscription value is accompanied by its source,
//! the financial period it refers to (financials), and the timestamp it was
//! retrieved (`fetched_at` / `captured_at` / `updated_at`). Values are never
//! invented; missing values serialize as `null` and the UI shows "Not Available".

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SubscriptionPointRow {
    pub day: NaiveDate,
    pub retail: Option<Decimal>,
    pub nii: Option<Decimal>,
    pub qib: Option<Decimal>,
    pub employee: Option<Decimal>,
    pub shareholder: Option<Decimal>,
    pub overall: Option<Decimal>,
    pub is_final: bool,
    /// Provider of the figure, e.g. `nse`.
    pub source: String,
    /// `official` (exchange/regulator/company filing) or `unofficial`.
    pub source_type: String,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub available: bool,
    pub retail: Option<Decimal>,
    pub nii: Option<Decimal>,
    pub qib: Option<Decimal>,
    pub employee: Option<Decimal>,
    pub shareholder: Option<Decimal>,
    pub overall: Option<Decimal>,
    pub is_final: bool,
    pub source: Option<String>,
    pub source_type: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub history: Vec<SubscriptionPointRow>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FinancialPeriodRow {
    /// Financial period label, e.g. `FY2024` or `H1 FY2025`.
    pub period: String,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub revenue: Option<Decimal>,
    pub revenue_growth_pct: Option<Decimal>,
    pub ebitda: Option<Decimal>,
    pub ebitda_margin_pct: Option<Decimal>,
    pub pat: Option<Decimal>,
    pub pat_growth_pct: Option<Decimal>,
    pub eps: Option<Decimal>,
    pub pe_ratio: Option<Decimal>,
    pub roe_pct: Option<Decimal>,
    pub roce_pct: Option<Decimal>,
    pub debt: Option<Decimal>,
    pub debt_to_equity: Option<Decimal>,
    pub audited: bool,
    /// Provider of the figure, e.g. `rhp` (official company filing).
    pub source: String,
    /// `official` (audited/company filing) or `unofficial`.
    pub source_type: String,
    pub updated_at: DateTime<Utc>,
}

/// Deterministic growth analysis for a single metric (revenue / PAT / EPS).
#[derive(Debug, Serialize, Clone)]
pub struct MetricAnalysis {
    pub label: String,
    pub latest_value: Option<Decimal>,
    pub latest_period: Option<String>,
    pub yoy_growth_pct: Option<Decimal>,
    pub cagr_pct: Option<Decimal>,
    pub cagr_start_period: Option<String>,
    pub cagr_years: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct FinancialGrowth {
    pub revenue: MetricAnalysis,
    pub pat: MetricAnalysis,
    pub eps: MetricAnalysis,
}

/// Valuation analysis for the IPO. `sector_pe` / `premium_discount_pct` are
/// only populated when a reliable sector/peer benchmark exists; in v1 they are
/// always `null` and the note says so rather than inventing a comparison.
#[derive(Debug, Serialize)]
pub struct ValuationResponse {
    pub available: bool,
    pub pe_ratio: Option<Decimal>,
    pub eps: Option<Decimal>,
    pub issue_price: Option<Decimal>,
    pub implied_pe: Option<Decimal>,
    pub sector_pe: Option<Decimal>,
    pub premium_discount_pct: Option<Decimal>,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct FinancialsResponse {
    pub available: bool,
    pub periods: Vec<FinancialPeriodRow>,
    pub growth: FinancialGrowth,
    pub valuation: ValuationResponse,
}

#[derive(Debug, Serialize)]
pub struct ScoreComponentResponse {
    pub key: String,
    pub label: String,
    pub max_points: u32,
    pub score: Option<Decimal>,
    pub status: &'static str,
    pub explanation: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DataSourceMeta {
    pub provider: String,
    pub category: String,
    pub official: bool,
    pub api_url: Option<String>,
    pub refresh_frequency_secs: Option<i32>,
    pub licensing: Option<String>,
    pub rate_limits: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoreResponse {
    pub total: Option<Decimal>,
    pub max_points: u32,
    pub methodology_version: String,
    pub data_quality: DataQualityResponse,
    pub components: Vec<ScoreComponentResponse>,
    pub positive_factors: Vec<String>,
    pub concerns: Vec<String>,
    pub disclaimer: String,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DataQualityResponse {
    pub overall: String,
    pub missing: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DataSourceFreshness {
    pub ipo_synced_at: Option<DateTime<Utc>>,
    pub subscription_updated_at: Option<DateTime<Utc>>,
    pub financials_updated_at: Option<DateTime<Utc>>,
    pub score_computed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct IntelMetaResponse {
    pub ipo_id: Uuid,
    pub company_name: String,
    pub data_sources: Vec<DataSourceMeta>,
    pub freshness: DataSourceFreshness,
}
