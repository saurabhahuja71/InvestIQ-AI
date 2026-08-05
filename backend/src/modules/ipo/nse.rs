//! NSE India public IPO endpoints (website JSON APIs).
//!
//! These are the same JSON feeds used by https://www.nseindia.com/market-data/
//! public-issues pages. They require a browser-like session cookie obtained from
//! the NSE homepage. They are **not** a formally licensed commercial API;
//! see `docs/11-ipo-data-provider.md`.

use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDate;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, REFERER, USER_AGENT};
use reqwest::{Client, cookie::Jar};
use rust_decimal::Decimal;
use serde_json::Value;
use tokio::sync::Mutex;

const NSE_HOME: &str = "https://www.nseindia.com/";
const NSE_BASE: &str = "https://www.nseindia.com/api";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Clone)]
pub struct NseClient {
    client: Client,
    jar: Arc<Jar>,
    session_ready: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub struct NseIpoSummary {
    pub company_name: String,
    pub symbol: String,
    pub series: String,
    pub status_raw: String,
    pub open_date: Option<NaiveDate>,
    pub close_date: Option<NaiveDate>,
    pub listing_date: Option<NaiveDate>,
    pub price_band_low: Option<Decimal>,
    pub price_band_high: Option<Decimal>,
    pub issue_price: Option<Decimal>,
    pub lot_size: Option<i32>,
    pub issue_size_shares: Option<Decimal>,
    pub subscription_total: Option<Decimal>,
    pub is_bse: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NseIpoDetail {
    pub company_name: Option<String>,
    pub symbol: Option<String>,
    pub issue_type: Option<String>,
    pub price_band_low: Option<Decimal>,
    pub price_band_high: Option<Decimal>,
    pub lot_size: Option<i32>,
    pub face_value: Option<Decimal>,
    pub min_order_qty: Option<i32>,
    pub registrar: Option<String>,
    pub lead_managers: Vec<String>,
    pub open_date: Option<NaiveDate>,
    pub close_date: Option<NaiveDate>,
    pub rhp_url: Option<String>,
    pub ratios_url: Option<String>,
    pub issue_size_text: Option<String>,
    pub subscription_total: Option<Decimal>,
    pub subscription_retail: Option<Decimal>,
    pub subscription_qib: Option<Decimal>,
    pub subscription_nii: Option<Decimal>,
    pub issue_info: serde_json::Map<String, Value>,
}

impl NseClient {
    pub fn new() -> anyhow::Result<Self> {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(jar.clone())
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;
        Ok(Self {
            client,
            jar,
            session_ready: Arc::new(Mutex::new(false)),
        })
    }

    async fn ensure_session(&self) -> anyhow::Result<()> {
        let mut ready = self.session_ready.lock().await;
        if *ready {
            return Ok(());
        }
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));
        headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml"));
        let res = self.client.get(NSE_HOME).headers(headers).send().await?;
        if !res.status().is_success() && res.status().as_u16() != 403 {
            tracing::warn!(status = %res.status(), "NSE homepage returned unexpected status");
        }
        // Cookie jar is populated even on some soft failures.
        let _ = self.jar;
        *ready = true;
        Ok(())
    }

    async fn get_json(&self, path_and_query: &str) -> anyhow::Result<Value> {
        self.ensure_session().await?;
        let url = format!("{NSE_BASE}/{path_and_query}");
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            REFERER,
            HeaderValue::from_static(
                "https://www.nseindia.com/market-data/all-upcoming-issues-ipo",
            ),
        );

        let res = self.client.get(&url).headers(headers).send().await?;
        if res.status().as_u16() == 401 || res.status().as_u16() == 403 {
            // Refresh session once and retry.
            {
                let mut ready = self.session_ready.lock().await;
                *ready = false;
            }
            self.ensure_session().await?;
            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, HeaderValue::from_static(UA));
            headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
            headers.insert(
                REFERER,
                HeaderValue::from_static(
                    "https://www.nseindia.com/market-data/all-upcoming-issues-ipo",
                ),
            );
            let retry = self.client.get(&url).headers(headers).send().await?;
            if !retry.status().is_success() {
                anyhow::bail!("NSE request failed after retry: {} {}", retry.status(), url);
            }
            return Ok(retry.json().await?);
        }
        if !res.status().is_success() {
            anyhow::bail!("NSE request failed: {} {}", res.status(), url);
        }
        Ok(res.json().await?)
    }

    pub async fn list_current(&self) -> anyhow::Result<Vec<NseIpoSummary>> {
        let v = self.get_json("ipo-current-issue").await?;
        Ok(parse_summary_list(&v, "current"))
    }

    pub async fn list_upcoming(&self) -> anyhow::Result<Vec<NseIpoSummary>> {
        let v = self.get_json("all-upcoming-issues?category=ipo").await?;
        Ok(parse_summary_list(&v, "upcoming"))
    }

    pub async fn list_past(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> anyhow::Result<Vec<NseIpoSummary>> {
        let from_s = from.format("%d-%m-%Y").to_string();
        let to_s = to.format("%d-%m-%Y").to_string();
        let path = format!("public-past-issues?from_date={from_s}&to_date={to_s}");
        let v = self.get_json(&path).await?;
        Ok(parse_summary_list(&v, "past"))
    }

    pub async fn get_detail(&self, symbol: &str, series: &str) -> anyhow::Result<NseIpoDetail> {
        let path = format!(
            "ipo-detail?symbol={}&series={}",
            urlencoding_lite(symbol),
            urlencoding_lite(series)
        );
        let v = self.get_json(&path).await?;
        Ok(parse_detail(&v))
    }
}

fn urlencoding_lite(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn parse_summary_list(v: &Value, source: &str) -> Vec<NseIpoSummary> {
    let owned: Vec<Value>;
    let arr: &[Value] = match v {
        Value::Array(a) => a.as_slice(),
        Value::Object(o) => {
            owned = o
                .get("data")
                .and_then(|d| d.as_array())
                .cloned()
                .unwrap_or_default();
            owned.as_slice()
        }
        _ => return vec![],
    };

    let mut out = Vec::new();
    for item in arr {
        if let Some(s) = parse_summary_item(item, source) {
            out.push(s);
        }
    }
    out
}

fn parse_summary_item(item: &Value, source: &str) -> Option<NseIpoSummary> {
    let obj = item.as_object()?;
    let company_name = obj
        .get("companyName")
        .or_else(|| obj.get("company"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let symbol = obj
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_uppercase();
    if company_name.is_empty() || symbol.is_empty() {
        return None;
    }

    let series = obj
        .get("series")
        .or_else(|| obj.get("securityType"))
        .and_then(|v| v.as_str())
        .unwrap_or("EQ")
        .trim()
        .to_uppercase();

    let status_raw = obj
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or(match source {
            "current" => "Active",
            "upcoming" => "Forthcoming",
            _ => "Closed",
        })
        .to_string();

    let open_date = parse_nse_date(
        obj.get("issueStartDate")
            .or_else(|| obj.get("ipoStartDate"))
            .and_then(|v| v.as_str()),
    );
    let close_date = parse_nse_date(
        obj.get("issueEndDate")
            .or_else(|| obj.get("ipoEndDate"))
            .and_then(|v| v.as_str()),
    );
    let listing_date = parse_nse_date(obj.get("listingDate").and_then(|v| v.as_str()));

    let price_text = obj
        .get("issuePrice")
        .or_else(|| obj.get("priceBand"))
        .or_else(|| obj.get("priceRange"))
        .and_then(|v| v.as_str());
    let (price_band_low, price_band_high) = parse_price_band(price_text);
    let issue_price = if price_band_low == price_band_high {
        price_band_low
    } else {
        // Fixed issue price when not a range (past IPOs often send a single number)
        parse_decimal_loose(price_text.filter(|s| !s.contains("to") && !s.contains('-')))
    };

    let lot_size = obj
        .get("lotSize")
        .and_then(value_to_i32)
        .or_else(|| parse_int_loose(obj.get("lotSize").and_then(|v| v.as_str())));

    let issue_size_shares = obj
        .get("issueSize")
        .and_then(value_to_decimal)
        .or_else(|| parse_decimal_loose(obj.get("issueSize").and_then(|v| v.as_str())));

    let subscription_total = obj
        .get("noOfTime")
        .and_then(value_to_decimal)
        .or_else(|| parse_decimal_loose(obj.get("noOfTime").and_then(|v| v.as_str())));

    let is_bse = obj
        .get("isBse")
        .map(|v| match v {
            Value::String(s) => s == "1" || s.eq_ignore_ascii_case("true"),
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_i64() == Some(1),
            _ => false,
        })
        .unwrap_or(false);

    Some(NseIpoSummary {
        company_name,
        symbol,
        series,
        status_raw,
        open_date,
        close_date,
        listing_date,
        price_band_low,
        price_band_high,
        issue_price,
        lot_size,
        issue_size_shares,
        subscription_total,
        is_bse,
    })
}

fn parse_detail(v: &Value) -> NseIpoDetail {
    let mut detail = NseIpoDetail {
        company_name: v
            .get("companyName")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        symbol: None,
        ..Default::default()
    };

    if let Some(arr) = v
        .pointer("/issueInfo/dataList")
        .and_then(|x| x.as_array())
    {
        for item in arr {
            let title = item
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .trim();
            let value = item.get("value").cloned().unwrap_or(Value::Null);
            let value_str = json_to_clean_string(&value);
            if title.is_empty() {
                continue;
            }
            detail
                .issue_info
                .insert(title.to_string(), Value::String(value_str.clone()));

            let t = title.to_ascii_lowercase();
            match t.as_str() {
                "symbol" => detail.symbol = Some(value_str.to_uppercase()),
                "issue type" => detail.issue_type = Some(value_str.clone()),
                "price range" => {
                    let (lo, hi) = parse_price_band(Some(&value_str));
                    detail.price_band_low = lo;
                    detail.price_band_high = hi;
                }
                "face value" => detail.face_value = parse_decimal_loose(Some(&value_str)),
                "bid lot" | "lot size" => {
                    detail.lot_size = parse_lot_size(&value_str);
                }
                "minimum order quantity" => {
                    detail.min_order_qty = parse_lot_size(&value_str);
                }
                "name of the registrar" => detail.registrar = Some(value_str.clone()),
                "book running lead managers" => {
                    detail.lead_managers = split_managers(&value_str);
                }
                "issue period" => {
                    let (o, c) = parse_issue_period(&value_str);
                    detail.open_date = o;
                    detail.close_date = c;
                }
                "issue size" => detail.issue_size_text = Some(value_str.clone()),
                "red herring prospectus" => {
                    detail.rhp_url = extract_url(&value_str);
                }
                "ratios / basis of issue price" => {
                    detail.ratios_url = extract_url(&value_str);
                }
                _ => {}
            }
            // Company display name is often the first titled empty-value row
            if detail.company_name.as_deref() == Some("ARDEE")
                || detail
                    .company_name
                    .as_ref()
                    .map(|n| n.eq_ignore_ascii_case(detail.symbol.as_deref().unwrap_or("")))
                    .unwrap_or(false)
            {
                // keep list name preferred
            }
            if !title.is_empty()
                && value_str.is_empty()
                && !title.eq_ignore_ascii_case("symbol")
                && title.chars().any(|c| c.is_alphabetic())
                && title.len() > 3
                && !t.contains("cut-off")
                && !t.contains("remark")
            {
                // First company-name style title with empty value
                if detail
                    .company_name
                    .as_ref()
                    .map(|n| n.len() < 8 || n.chars().all(|c| c.is_ascii_uppercase()))
                    .unwrap_or(true)
                    && title.contains(' ')
                {
                    detail.company_name = Some(title.to_string());
                }
            }
        }
    }

    if let Some(bids) = v.get("bidDetails").and_then(|b| b.as_array()) {
        for bid in bids {
            let cat = bid
                .get("category")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let times = bid
                .get("noOfTime")
                .and_then(value_to_decimal)
                .or_else(|| parse_decimal_loose(bid.get("noOfTime").and_then(|x| x.as_str())));
            if cat == "total" {
                detail.subscription_total = times.or(detail.subscription_total);
            } else if cat.contains("retail") || cat.contains("rii") || cat.contains("individual investors (ind")
            {
                detail.subscription_retail = times.or(detail.subscription_retail);
            } else if cat.starts_with("qualified institutional") || cat == "qibs"
            {
                detail.subscription_qib = times.or(detail.subscription_qib);
            } else if cat.starts_with("non institutional investors") && !cat.contains("bid amount")
            {
                detail.subscription_nii = times.or(detail.subscription_nii);
            }
        }
    }

    detail
}

fn json_to_clean_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.trim().trim_matches('"').to_string(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

pub fn parse_nse_date(raw: Option<&str>) -> Option<NaiveDate> {
    let s = raw?.trim();
    if s.is_empty() || s == "-" {
        return None;
    }
    // Formats: 05-Aug-2026, 05-AUG-2026, 05-08-2026
    let formats = ["%d-%b-%Y", "%d-%B-%Y", "%d-%m-%Y", "%d/%m/%Y", "%Y-%m-%d"];
    for f in formats {
        if let Ok(d) = NaiveDate::parse_from_str(&s.to_ascii_lowercase(), &f.to_ascii_lowercase()) {
            return Some(d);
        }
        // chrono month abbrev is case-sensitive for %b — try title case
        if let Ok(d) = NaiveDate::parse_from_str(s, f) {
            return Some(d);
        }
    }
    // Manual: 05-Aug-2026
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() == 3 {
        let day: u32 = parts[0].trim().parse().ok()?;
        let year: i32 = parts[2].trim().parse().ok()?;
        let mon = month_from_str(parts[1].trim())?;
        return NaiveDate::from_ymd_opt(year, mon, day);
    }
    None
}

fn month_from_str(m: &str) -> Option<u32> {
    match m.to_ascii_lowercase().as_str() {
        "jan" | "january" | "01" | "1" => Some(1),
        "feb" | "february" | "02" | "2" => Some(2),
        "mar" | "march" | "03" | "3" => Some(3),
        "apr" | "april" | "04" | "4" => Some(4),
        "may" | "05" | "5" => Some(5),
        "jun" | "june" | "06" | "6" => Some(6),
        "jul" | "july" | "07" | "7" => Some(7),
        "aug" | "august" | "08" | "8" => Some(8),
        "sep" | "sept" | "september" | "09" | "9" => Some(9),
        "oct" | "october" | "10" => Some(10),
        "nov" | "november" | "11" => Some(11),
        "dec" | "december" | "12" => Some(12),
        _ => None,
    }
}

pub fn parse_price_band(raw: Option<&str>) -> (Option<Decimal>, Option<Decimal>) {
    let s = match raw {
        Some(x) => x,
        None => return (None, None),
    };
    // Extract numbers like 50, 53.5 from "Rs.50 to Rs.53" / "Rs. 560 to Rs. 590 per Equity Share"
    let nums: Vec<Decimal> = extract_decimals(s);
    match nums.as_slice() {
        [a] => (Some(*a), Some(*a)),
        [a, b, ..] => (Some(*a), Some(*b)),
        _ => (None, None),
    }
}

fn extract_decimals(s: &str) -> Vec<Decimal> {
    // Normalize currency markers so "Rs.50" is not read as "0.50"
    let normalized = s
        .replace("Rs.", " ")
        .replace("Rs", " ")
        .replace("Re.", " ")
        .replace("Re", " ")
        .replace('₹', " ")
        .replace(',', "");

    let mut out = Vec::new();
    let mut cur = String::new();
    for c in normalized.chars() {
        if c.is_ascii_digit() || c == '.' {
            cur.push(c);
        } else if !cur.is_empty() {
            push_price_decimal(&mut out, &cur);
            cur.clear();
        }
    }
    if !cur.is_empty() {
        push_price_decimal(&mut out, &cur);
    }
    out
}

fn push_price_decimal(out: &mut Vec<Decimal>, cur: &str) {
    let trimmed = cur.trim_matches('.');
    if trimmed.is_empty() {
        return;
    }
    if let Ok(d) = trimmed.parse::<Decimal>() {
        // Price bands / lots are typically well below 1e6; skip years etc.
        if d > Decimal::ZERO && d < Decimal::from(1_000_000) {
            out.push(d);
        }
    }
}

fn parse_decimal_loose(raw: Option<&str>) -> Option<Decimal> {
    let s = raw?.trim();
    if s.is_empty() || s == "-" {
        return None;
    }
    if let Ok(d) = s.parse::<Decimal>() {
        return Some(d);
    }
    extract_decimals(s).into_iter().next()
}

fn parse_int_loose(raw: Option<&str>) -> Option<i32> {
    parse_decimal_loose(raw)?.to_string().parse().ok()
}

fn parse_lot_size(s: &str) -> Option<i32> {
    // "Minimum 281 Equity shares and in multiples thereof" / "400 Equity Shares" / "800"
    extract_decimals(s)
        .into_iter()
        .find_map(|d| d.to_string().parse::<i32>().ok().filter(|&n| n > 0 && n < 1_000_000))
}

fn parse_issue_period(s: &str) -> (Option<NaiveDate>, Option<NaiveDate>) {
    // "05-Aug-2026 to 07-Aug-2026"
    let lower = s.to_ascii_lowercase();
    let parts: Vec<&str> = if lower.contains(" to ") {
        s.split(" to ").collect()
    } else if lower.contains(" - ") {
        s.split(" - ").collect()
    } else {
        return (parse_nse_date(Some(s)), None);
    };
    (
        parse_nse_date(parts.first().copied()),
        parse_nse_date(parts.get(1).copied()),
    )
}

fn split_managers(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().trim_matches('"').to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

fn extract_url(s: &str) -> Option<String> {
    if s.starts_with("http://") || s.starts_with("https://") {
        return Some(s.trim().to_string());
    }
    // <a href=URL ...>
    if let Some(idx) = s.find("href=") {
        let rest = &s[idx + 5..];
        let rest = rest.trim_start_matches(['"', '\'', ' ']);
        let end = rest
            .find([' ', '"', '\'', '>'])
            .unwrap_or(rest.len());
        let url = &rest[..end];
        if url.starts_with("http") {
            return Some(url.to_string());
        }
    }
    None
}

fn value_to_decimal(v: &Value) -> Option<Decimal> {
    match v {
        Value::Number(n) => n.to_string().parse().ok(),
        Value::String(s) => parse_decimal_loose(Some(s)),
        _ => None,
    }
}

fn value_to_i32(v: &Value) -> Option<i32> {
    match v {
        Value::Number(n) => n.as_i64().map(|i| i as i32),
        Value::String(s) => parse_int_loose(Some(s)),
        _ => None,
    }
}

pub fn map_board(series: &str) -> &'static str {
    match series.to_ascii_uppercase().as_str() {
        "SME" | "SM" => "sme",
        _ => "mainboard",
    }
}

pub fn map_status(
    status_raw: &str,
    open: Option<NaiveDate>,
    close: Option<NaiveDate>,
    listing: Option<NaiveDate>,
    today: NaiveDate,
    bucket: &str,
) -> &'static str {
    let s = status_raw.to_ascii_lowercase();
    if s.contains("active") || s.contains("open") {
        return "open";
    }
    if s.contains("forthcoming") || s.contains("upcoming") {
        return "upcoming";
    }
    if listing.is_some() && listing.unwrap() <= today {
        return "listed";
    }
    if listing.is_some() {
        return "closed"; // closed, awaiting listing
    }
    match bucket {
        "current" => "open",
        "upcoming" => "upcoming",
        "past" => {
            if let Some(c) = close {
                if c >= today {
                    return "closed";
                }
            }
            if open.map(|o| o > today).unwrap_or(false) {
                return "upcoming";
            }
            "closed"
        }
        _ => "closed",
    }
}

pub fn exchange_label(series: &str, is_bse: bool) -> String {
    let board = map_board(series);
    if board == "sme" {
        if is_bse {
            "BSE SME".into()
        } else {
            "NSE SME".into()
        }
    } else if is_bse {
        "BSE".into()
    } else {
        "NSE".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nse_dates() {
        assert_eq!(
            parse_nse_date(Some("05-Aug-2026")),
            Some(NaiveDate::from_ymd_opt(2026, 8, 5).unwrap())
        );
        assert_eq!(
            parse_nse_date(Some("03-AUG-2026")),
            Some(NaiveDate::from_ymd_opt(2026, 8, 3).unwrap())
        );
        assert_eq!(parse_nse_date(Some("-")), None);
    }

    #[test]
    fn parses_price_bands() {
        let (lo, hi) = parse_price_band(Some("Rs.50 to Rs.53"));
        assert_eq!(lo.unwrap().to_string(), "50");
        assert_eq!(hi.unwrap().to_string(), "53");
        let (lo, hi) = parse_price_band(Some("Rs. 560 to Rs. 590 per Equity Share"));
        assert_eq!(lo.unwrap().to_string(), "560");
        assert_eq!(hi.unwrap().to_string(), "590");
    }

    #[test]
    fn maps_board_and_status() {
        assert_eq!(map_board("SME"), "sme");
        assert_eq!(map_board("EQ"), "mainboard");
        let today = NaiveDate::from_ymd_opt(2026, 8, 5).unwrap();
        assert_eq!(
            map_status("Active", None, None, None, today, "current"),
            "open"
        );
        assert_eq!(
            map_status(
                "Closed",
                None,
                None,
                Some(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()),
                today,
                "past"
            ),
            "listed"
        );
    }
}
