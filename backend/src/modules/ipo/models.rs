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

/// Minimum investment required to participate, per investor category.
/// Retail/employee figures come from NSE issue metadata (lot size × price, and
/// the NSE-published maximum subscription caps). The NII/HNI figure is the
/// SEBI regulatory minimum application value (₹2,00,000) for the mainboard NII
/// category, clearly labelled as such — never fabricated per-IPO data.
#[derive(Debug, Serialize)]
pub struct InvestmentRequirements {
    pub bid_lot: Option<i32>,
    pub retail_min_amount: Option<Decimal>,
    pub retail_max_amount: Option<Decimal>,
    pub employee_max_amount: Option<Decimal>,
    pub nii_min_amount: Option<Decimal>,
    pub nii_min_note: String,
}

impl InvestmentRequirements {
    fn unavailable() -> Self {
        Self {
            bid_lot: None,
            retail_min_amount: None,
            retail_max_amount: None,
            employee_max_amount: None,
            nii_min_amount: None,
            nii_min_note: String::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct IpoDetail {
    #[serde(flatten)]
    pub row: IpoDetailResponse,
    pub investment_requirements: InvestmentRequirements,
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

/// Extract a positive INR amount from NSE text such as `Rs. 2,00,000` or
/// `"Rs. 5,00,000"`. Digits only, so `Rs. 2,00,000` → 200000.
fn parse_rupee_amount(raw: Option<&str>) -> Option<Decimal> {
    let digits: String = raw?
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if digits.is_empty() {
        return None;
    }
    digits
        .trim_matches('.')
        .parse::<Decimal>()
        .ok()
        .filter(|d| *d > Decimal::ZERO)
}

/// Leading integer from an NSE lot-size string, e.g. `94 Equity Shares …` → 94.
fn parse_lot_int(s: &str) -> Option<i32> {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok().filter(|n| *n > 0)
    }
}

fn issue_info_value(info: &serde_json::Value, key: &str) -> Option<String> {
    info.as_object()?
        .get(key)?
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn investment_requirements(row: &IpoDetailRow) -> InvestmentRequirements {
    let info = &row.issue_info;
    let info_get = |key: &str| issue_info_value(info, key);

    let bid_lot = row
        .lot_size
        .or_else(|| info_get("Bid Lot").as_deref().and_then(parse_lot_int))
        .or_else(|| info_get("Minimum Order Quantity").as_deref().and_then(parse_lot_int));

    let price = row.price_band_high.or(row.price_band_low);
    let retail_min_amount = match (bid_lot, price) {
        (Some(lot), Some(px)) => Some(Decimal::from(lot) * px),
        _ => None,
    };

    let retail_max_amount = parse_rupee_amount(
        info_get("Maximum Subscription Amount for Retail Investor").as_deref(),
    );
    let employee_max_amount = parse_rupee_amount(
        info_get("Maximum Subscription Amount for Eligible Employee").as_deref(),
    );

    // SEBI regulatory minimum application value for the mainboard NII/HNI
    // category (₹2,00,000). Not a per-IPO data point — a regulatory constant,
    // labelled as such in the UI. SME issues have no separate NII category.
    let nii_min_amount = if row.board.eq_ignore_ascii_case("mainboard") {
        Some(Decimal::from(200_000))
    } else {
        None
    };

    if bid_lot.is_none()
        && retail_min_amount.is_none()
        && retail_max_amount.is_none()
        && employee_max_amount.is_none()
        && nii_min_amount.is_none()
    {
        return InvestmentRequirements::unavailable();
    }

    InvestmentRequirements {
        bid_lot,
        retail_min_amount,
        retail_max_amount,
        employee_max_amount,
        nii_min_amount,
        nii_min_note: "SEBI regulatory minimum application value for the NII/HNI category (₹2,00,000)."
            .to_string(),
    }
}

impl From<IpoDetailRow> for IpoDetail {
    fn from(row: IpoDetailRow) -> Self {
        let prospectus = row.rhp_url.clone().or_else(|| row.drhp_url.clone());
        let investment_requirements = investment_requirements(&row);
        Self {
            investment_requirements,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rupee_amounts() {
        assert_eq!(
            parse_rupee_amount(Some("Rs. 2,00,000")),
            Some(Decimal::from(200_000))
        );
        assert_eq!(
            parse_rupee_amount(Some("\"Rs. 5,00,000\"")),
            Some(Decimal::from(500_000))
        );
        assert_eq!(parse_rupee_amount(Some("-")), None);
        assert_eq!(parse_rupee_amount(Some("")), None);
        assert_eq!(parse_rupee_amount(None), None);
    }

    #[test]
    fn parses_lot_ints() {
        assert_eq!(parse_lot_int("94 Equity Shares and in multiples thereof"), Some(94));
        assert_eq!(parse_lot_int("50 Equity Shares"), Some(50));
        assert_eq!(parse_lot_int("Equity Shares"), None);
    }

    #[test]
    fn issue_info_value_reads_clean_strings() {
        let info = serde_json::json!({
            "Bid Lot": "94 Equity Shares and in multiples thereof",
            "Maximum Subscription Amount for Retail Investor": "Rs. 2,00,000",
        });
        assert_eq!(
            issue_info_value(&info, "Bid Lot"),
            Some("94 Equity Shares and in multiples thereof".to_string())
        );
        assert_eq!(issue_info_value(&info, "Missing"), None);
    }

    fn row_with(issue_info: serde_json::Value) -> IpoDetailRow {
        IpoDetailRow {
            id: Uuid::new_v4(),
            company_id: Uuid::new_v4(),
            company_name: "Test Co".into(),
            symbol: Some("TEST".into()),
            sector: None,
            industry: None,
            description: None,
            logo_url: None,
            website: None,
            board: "mainboard".into(),
            status: "open".into(),
            issue_type: None,
            price_band_low: Some(Decimal::from(151)),
            price_band_high: Some(Decimal::from(159)),
            issue_price: None,
            lot_size: Some(94),
            face_value: None,
            min_investment: None,
            issue_size_cr: None,
            shares_offered: None,
            open_date: None,
            close_date: None,
            allotment_date: None,
            refund_date: None,
            listing_date: None,
            exchange: None,
            registrar: None,
            lead_managers: serde_json::json!([]),
            subscription_total: None,
            subscription_retail: None,
            subscription_qib: None,
            subscription_nii: None,
            listing_open: None,
            listing_close: None,
            financials: serde_json::json!({}),
            pros: serde_json::json!([]),
            risks: serde_json::json!([]),
            ai_summary: None,
            drhp_url: None,
            rhp_url: None,
            issue_info,
            source: None,
            source_synced_at: None,
            nse_symbol: None,
            nse_series: None,
        }
    }

    #[test]
    fn investment_requirements_computed_from_real_data() {
        let info = serde_json::json!({
            "Bid Lot": "94 Equity Shares and in multiples thereof",
            "Maximum Subscription Amount for Retail Investor": "Rs. 2,00,000",
            "Maximum Subscription Amount for Eligible Employee": "Rs. 5,00,000",
        });
        let req = investment_requirements(&row_with(info));
        // lot 94 × upper band 159
        assert_eq!(req.retail_min_amount, Some(Decimal::from(94 * 159)));
        assert_eq!(req.bid_lot, Some(94));
        assert_eq!(req.retail_max_amount, Some(Decimal::from(200_000)));
        assert_eq!(req.employee_max_amount, Some(Decimal::from(500_000)));
        assert_eq!(req.nii_min_amount, Some(Decimal::from(200_000)));
        assert!(!req.nii_min_note.is_empty());
    }

    #[test]
    fn investment_requirements_unavailable_when_no_data() {
        let mut row = row_with(serde_json::json!({}));
        row.lot_size = None;
        row.price_band_low = None;
        row.price_band_high = None;
        let req = investment_requirements(&row);
        assert_eq!(req.bid_lot, None);
        assert_eq!(req.retail_min_amount, None);
        assert_eq!(req.retail_max_amount, None);
        // mainboard still reports the regulatory NII floor even when the rest
        // is missing — that is a regulatory constant, not fabricated data.
        assert_eq!(req.nii_min_amount, Some(Decimal::from(200_000)));
    }

    #[test]
    fn sme_has_no_nii_floor() {
        let mut row = row_with(serde_json::json!({}));
        row.board = "sme".into();
        let req = investment_requirements(&row);
        assert_eq!(req.nii_min_amount, None);
    }
}
