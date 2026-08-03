//! NSE India public IPO feed client + Postgres upsert sync.

use std::str::FromStr;
use std::time::Duration;

use chrono::{NaiveDate, Utc};
use redis::AsyncCommands;
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AppError, AppResult};

const NSE_HOME: &str = "https://www.nseindia.com";
const NSE_IPO_PAGE: &str = "https://www.nseindia.com/market-data/all-upcoming-issues-ipo";
const NSE_CURRENT: &str = "https://www.nseindia.com/api/ipo-current-issue";
const NSE_PAST: &str = "https://www.nseindia.com/api/public-past-issues";
const SOURCE: &str = "nse";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const SYNC_LOCK_KEY: &str = "ipo:sync:lock";
const SYNC_META_KEY: &str = "ipo:sync:last_ok";

#[derive(Debug, Clone)]
pub struct NseIpoRecord {
    pub company_name: String,
    pub symbol: String,
    pub board: String,
    pub status: String,
    pub price_band_low: Option<Decimal>,
    pub price_band_high: Option<Decimal>,
    pub issue_price: Option<Decimal>,
    pub open_date: Option<NaiveDate>,
    pub close_date: Option<NaiveDate>,
    pub listing_date: Option<NaiveDate>,
    pub exchange: Option<String>,
    pub subscription_total: Option<Decimal>,
    pub shares_offered: Option<i64>,
}

#[derive(Clone)]
pub struct NseIpoClient {
    http: reqwest::Client,
}

impl NseIpoClient {
    pub fn new() -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| AppError::Internal(format!("nse client: {e}")))?;
        Ok(Self { http })
    }

    async fn bootstrap_session(&self) -> AppResult<()> {
        // NSE requires a cookie jar warmed by a browser-like page hit.
        let _ = self
            .http
            .get(NSE_HOME)
            .header("Accept", "text/html,application/xhtml+xml")
            .send()
            .await;
        let res = self
            .http
            .get(NSE_IPO_PAGE)
            .header("Accept", "text/html,application/xhtml+xml")
            .header("Referer", NSE_HOME)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("nse session: {e}")))?;
        if !res.status().is_success() && res.status().as_u16() != 403 {
            tracing::warn!(status = %res.status(), "nse session page returned non-success");
        }
        Ok(())
    }

    async fn get_json(&self, url: &str) -> AppResult<serde_json::Value> {
        let res = self
            .http
            .get(url)
            .header("Accept", "application/json")
            .header("Referer", NSE_IPO_PAGE)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("nse request: {e}")))?;

        let status = res.status();
        let body = res
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("nse body: {e}")))?;

        if !status.is_success() {
            return Err(AppError::Internal(format!(
                "nse {url} status {status}: {}",
                body.chars().take(200).collect::<String>()
            )));
        }

        serde_json::from_str(&body)
            .map_err(|e| AppError::Internal(format!("nse json parse: {e}")))
    }

    pub async fn fetch_all(&self) -> AppResult<Vec<NseIpoRecord>> {
        self.bootstrap_session().await?;

        let mut out = Vec::new();

        let current = self.get_json(NSE_CURRENT).await?;
        if let Some(arr) = current.as_array() {
            for item in arr {
                if let Some(rec) = parse_current(item) {
                    out.push(rec);
                }
            }
        }

        match self.get_json(NSE_PAST).await {
            Ok(past) => {
                if let Some(arr) = past.as_array() {
                    for item in arr {
                        if let Some(rec) = parse_past(item) {
                            // Prefer current/active record if same symbol already present.
                            if !out.iter().any(|r| r.symbol == rec.symbol) {
                                out.push(rec);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "nse past issues fetch failed; continuing with current only");
            }
        }

        Ok(out)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct _Unused;

fn parse_current(v: &serde_json::Value) -> Option<NseIpoRecord> {
    let company_name = v
        .get("companyName")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())?
        .to_string();
    let symbol = v
        .get("symbol")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())?
        .to_uppercase();

    let series = v
        .get("series")
        .and_then(|x| x.as_str())
        .unwrap_or("EQ")
        .to_uppercase();
    let board = if series.contains("SME") {
        "sme".to_string()
    } else {
        "mainboard".to_string()
    };

    let open_date = parse_date(v.get("issueStartDate").and_then(|x| x.as_str()));
    let close_date = parse_date(v.get("issueEndDate").and_then(|x| x.as_str()));
    let status = map_status(
        v.get("status").and_then(|x| x.as_str()),
        open_date,
        close_date,
        false,
    );

    let (low, high, fixed) = parse_price_fields(
        v.get("issuePrice")
            .and_then(|x| x.as_str())
            .or_else(|| v.get("priceRange").and_then(|x| x.as_str())),
    );

    let is_bse = v
        .get("isBse")
        .and_then(|x| x.as_str())
        .map(|s| s == "1")
        .unwrap_or(false);
    let exchange = Some(if is_bse {
        "BSE".to_string()
    } else {
        "NSE".to_string()
    });

    let subscription_total = parse_decimal_loose(
        v.get("noOfTime")
            .map(|x| match x {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                _ => String::new(),
            })
            .as_deref(),
    );

    let shares_offered = parse_i64_loose(
        v.get("noOfSharesOffered")
            .or_else(|| v.get("issueSize"))
            .map(|x| match x {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                _ => String::new(),
            })
            .as_deref(),
    );

    Some(NseIpoRecord {
        company_name,
        symbol,
        board,
        status,
        price_band_low: low,
        price_band_high: high,
        issue_price: fixed,
        open_date,
        close_date,
        listing_date: None,
        exchange,
        subscription_total,
        shares_offered,
    })
}

fn parse_past(v: &serde_json::Value) -> Option<NseIpoRecord> {
    let company_name = v
        .get("company")
        .or_else(|| v.get("companyName"))
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())?
        .to_string();
    let symbol = v
        .get("symbol")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())?
        .to_uppercase();

    let security = v
        .get("securityType")
        .and_then(|x| x.as_str())
        .unwrap_or("EQ")
        .to_uppercase();
    let board = if security.contains("SME") {
        "sme".to_string()
    } else {
        "mainboard".to_string()
    };

    let open_date = parse_date(v.get("ipoStartDate").and_then(|x| x.as_str()));
    let close_date = parse_date(v.get("ipoEndDate").and_then(|x| x.as_str()));
    let listing_date = parse_date(v.get("listingDate").and_then(|x| x.as_str()));
    let status = map_status(Some("Closed"), open_date, close_date, true);

    let (low, high, fixed) = parse_price_fields(
        v.get("priceRange")
            .and_then(|x| x.as_str())
            .or_else(|| v.get("issuePrice").and_then(|x| x.as_str())),
    );

    Some(NseIpoRecord {
        company_name,
        symbol,
        board,
        status,
        price_band_low: low,
        price_band_high: high,
        issue_price: fixed,
        open_date,
        close_date,
        listing_date,
        exchange: Some("NSE".to_string()),
        subscription_total: None,
        shares_offered: None,
    })
}

fn map_status(
    raw: Option<&str>,
    open: Option<NaiveDate>,
    close: Option<NaiveDate>,
    force_closed: bool,
) -> String {
    if force_closed {
        return "closed".into();
    }
    let today = Utc::now().date_naive();
    let lower = raw.unwrap_or("").to_ascii_lowercase();
    if lower.contains("active") || lower.contains("open") {
        return "open".into();
    }
    if lower.contains("upcoming") || lower.contains("forthcoming") {
        return "upcoming".into();
    }
    if lower.contains("close") || lower.contains("list") {
        return "closed".into();
    }
    match (open, close) {
        (Some(o), _) if o > today => "upcoming".into(),
        (_, Some(c)) if c < today => "closed".into(),
        (Some(o), Some(c)) if o <= today && today <= c => "open".into(),
        _ => "upcoming".into(),
    }
}

fn parse_date(s: Option<&str>) -> Option<NaiveDate> {
    let s = s?.trim();
    if s.is_empty() || s == "-" {
        return None;
    }
    // Formats seen: 03-Aug-2026, 31-JUL-2026, 03-Aug-2026
    let formats = ["%d-%b-%Y", "%d-%B-%Y", "%d/%m/%Y", "%Y-%m-%d", "%d-%b-%y"];
    for f in formats {
        if let Ok(d) = NaiveDate::parse_from_str(&s.to_ascii_lowercase(), &f.to_ascii_lowercase()) {
            return Some(d);
        }
        // chrono month abbreviations are case-sensitive in some versions — try as-is
        if let Ok(d) = NaiveDate::parse_from_str(s, f) {
            return Some(d);
        }
    }
    // Title-case month: 03-Aug-2026 already covered; try upper month
    let titled = s
        .split('-')
        .enumerate()
        .map(|(i, part)| {
            if i == 1 && part.len() >= 3 {
                let mut c = part.to_ascii_lowercase();
                if let Some(first) = c.get_mut(0..1) {
                    first.make_ascii_uppercase();
                }
                c
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("-");
    NaiveDate::parse_from_str(&titled, "%d-%b-%Y").ok()
}

fn parse_price_fields(raw: Option<&str>) -> (Option<Decimal>, Option<Decimal>, Option<Decimal>) {
    let raw = match raw {
        Some(s) => s.trim(),
        None => return (None, None, None),
    };
    if raw.is_empty() || raw == "-" {
        return (None, None, None);
    }
    // e.g. "Rs.400 to Rs.425" or "Rs.100"
    let cleaned = raw
        .replace("Rs.", "")
        .replace("Rs", "")
        .replace("₹", "")
        .replace(',', "");
    // Extract all decimal-like tokens from price strings like "Rs.400 to Rs.425"
    let re = regex::Regex::new(r"\d+(?:\.\d+)?").ok();
    let nums: Vec<Decimal> = if let Some(re) = re {
        re.find_iter(&cleaned)
            .filter_map(|m| Decimal::from_str(m.as_str()).ok())
            .collect()
    } else {
        Vec::new()
    };

    match nums.as_slice() {
        [a, b, ..] => (Some(*a), Some(*b), None),
        [a] => (Some(*a), Some(*a), Some(*a)),
        _ => (None, None, None),
    }
}

fn parse_decimal_loose(s: Option<&str>) -> Option<Decimal> {
    let s = s?.trim();
    if s.is_empty() || s == "-" {
        return None;
    }
    // scientific notation
    if let Ok(f) = s.parse::<f64>() {
        return Decimal::from_str(&format!("{f:.6}")).ok();
    }
    Decimal::from_str(s).ok()
}

fn parse_i64_loose(s: Option<&str>) -> Option<i64> {
    let s = s?.trim();
    if s.is_empty() || s == "-" {
        return None;
    }
    if let Ok(v) = s.parse::<i64>() {
        return Some(v);
    }
    if let Ok(f) = s.parse::<f64>() {
        return Some(f.round() as i64);
    }
    None
}

pub async fn sync_ipos(
    db: &PgPool,
    redis: &mut redis::aio::ConnectionManager,
    client: &NseIpoClient,
) -> AppResult<usize> {
    // Distributed lock (best-effort)
    let locked: bool = redis
        .set_nx(SYNC_LOCK_KEY, "1")
        .await
        .unwrap_or(false);
    if locked {
        let _: Result<(), _> = redis.expire(SYNC_LOCK_KEY, 120).await;
    } else {
        // Another sync in progress
        return Ok(0);
    }

    let result = sync_ipos_inner(db, client).await;

    let _: Result<(), _> = redis.del(SYNC_LOCK_KEY).await;

    match &result {
        Ok(n) => {
            let _: Result<(), _> = redis
                .set_ex(
                    SYNC_META_KEY,
                    format!("{}:{}", Utc::now().timestamp(), n),
                    3600,
                )
                .await;
            tracing::info!(count = n, "nse ipo sync completed");
        }
        Err(e) => tracing::error!(error = %e, "nse ipo sync failed"),
    }

    result
}

async fn sync_ipos_inner(db: &PgPool, client: &NseIpoClient) -> AppResult<usize> {
    let records = client.fetch_all().await?;
    let mut count = 0usize;

    for rec in records {
        let company_id = if let Some(id) = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            SELECT id FROM companies
            WHERE external_source = $1 AND external_id = $2
            "#,
        )
        .bind(SOURCE)
        .bind(&rec.symbol)
        .fetch_optional(db)
        .await?
        {
            sqlx::query(r#"UPDATE companies SET name = $2, symbol = $3 WHERE id = $1"#)
                .bind(id)
                .bind(&rec.company_name)
                .bind(&rec.symbol)
                .execute(db)
                .await?;
            id
        } else {
            sqlx::query_scalar(
                r#"
                INSERT INTO companies (name, symbol, external_source, external_id)
                VALUES ($1, $2, $3, $4)
                RETURNING id
                "#,
            )
            .bind(&rec.company_name)
            .bind(&rec.symbol)
            .bind(SOURCE)
            .bind(&rec.symbol)
            .fetch_one(db)
            .await?
        };

        let existing: Option<uuid::Uuid> = sqlx::query_scalar(
            r#"
            SELECT id FROM ipos
            WHERE external_source = $1 AND external_id = $2
            "#,
        )
        .bind(SOURCE)
        .bind(&rec.symbol)
        .fetch_optional(db)
        .await?;

        if let Some(id) = existing {
            sqlx::query(
                r#"
                UPDATE ipos SET
                    company_id = $2,
                    board = $3::ipo_board,
                    status = $4::ipo_status,
                    price_band_low = $5,
                    price_band_high = $6,
                    issue_price = $7,
                    open_date = $8,
                    close_date = $9,
                    listing_date = COALESCE($10, listing_date),
                    exchange = $11,
                    subscription_total = COALESCE($12, subscription_total),
                    shares_offered = COALESCE($13, shares_offered),
                    gmp_value = NULL,
                    gmp_updated_at = NULL,
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(id)
            .bind(company_id)
            .bind(&rec.board)
            .bind(&rec.status)
            .bind(rec.price_band_low)
            .bind(rec.price_band_high)
            .bind(rec.issue_price)
            .bind(rec.open_date)
            .bind(rec.close_date)
            .bind(rec.listing_date)
            .bind(&rec.exchange)
            .bind(rec.subscription_total)
            .bind(rec.shares_offered)
            .execute(db)
            .await?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO ipos (
                    company_id, board, status,
                    price_band_low, price_band_high, issue_price,
                    open_date, close_date, listing_date, exchange,
                    subscription_total, shares_offered,
                    external_source, external_id,
                    gmp_value, gmp_updated_at
                ) VALUES (
                    $1, $2::ipo_board, $3::ipo_status,
                    $4, $5, $6,
                    $7, $8, $9, $10,
                    $11, $12,
                    $13, $14,
                    NULL, NULL
                )
                "#,
            )
            .bind(company_id)
            .bind(&rec.board)
            .bind(&rec.status)
            .bind(rec.price_band_low)
            .bind(rec.price_band_high)
            .bind(rec.issue_price)
            .bind(rec.open_date)
            .bind(rec.close_date)
            .bind(rec.listing_date)
            .bind(&rec.exchange)
            .bind(rec.subscription_total)
            .bind(rec.shares_offered)
            .bind(SOURCE)
            .bind(&rec.symbol)
            .execute(db)
            .await?;
        }

        count += 1;
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_price_band() {
        let (l, h, f) = parse_price_fields(Some("Rs.400 to Rs.425"));
        assert_eq!(l.unwrap().to_string(), "400");
        assert_eq!(h.unwrap().to_string(), "425");
        assert!(f.is_none());
    }

    #[test]
    fn parses_dates() {
        assert_eq!(
            parse_date(Some("03-Aug-2026")).unwrap().to_string(),
            "2026-08-03"
        );
        assert_eq!(
            parse_date(Some("31-JUL-2026")).unwrap().to_string(),
            "2026-07-31"
        );
        assert!(parse_date(Some("-")).is_none());
    }
}
