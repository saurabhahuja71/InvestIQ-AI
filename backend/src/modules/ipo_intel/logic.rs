//! InvestIQ IPO Score — fundamentals-first, transparent, explainable scoring
//! (pure logic, no LLM involvement, no fabricated data).
//!
//! ## Methodology (v2.0)
//!
//! The score is a weighted composite of eight components. Every component is
//! scored only from real, verifiable data. When the underlying data is
//! unavailable the component is marked `insufficient_data` instead of receiving
//! a fabricated score, and the total is re-normalized to /100 across the
//! components that could actually be scored.
//!
//! | Component                  | Max | Data required |
//! |----------------------------|-----|---------------|
//! | Revenue Growth             | 15  | revenue YoY growth (%) |
//! | EPS / PAT Growth           | 20  | EPS and/or PAT YoY growth (%) |
//! | Profitability & Margins    | 15  | PAT/EBITDA margin, ROE, ROCE |
//! | Balance Sheet / Debt       | 10  | debt-to-equity ratio |
//! | Valuation                  | 15  | P/E (vendor or issue price ÷ EPS) |
//! | IPO Subscription           | 15  | overall subscription multiple (official exchange data) |
//! | Industry / Business Quality| 5   | sector benchmark dataset (not available in v1) |
//! | Risk Factors               | 5   | board type, leverage, disclosed risks |
//! | **Total**                  | **100** | |
//!
//! ### Score bands
//!
//! Growth components (revenue / EPS-PAT):
//! `>= 25% → full, >= 15% → 80%, >= 10% → 60%, >= 5% → 40%, >= 0% → 20%, < 0% → 0`.
//!
//! Profitability: the average of every available signal
//! (PAT margin, EBITDA margin, ROE, ROCE), each scaled linearly with a ceiling
//! of 35% → full marks; negative signals score 0.
//!
//! Balance sheet: `D/E <= 0.5 → full, <= 1.0 → 80%, <= 2.0 → 60%, else → 30%`;
//! negative D/E (negative net worth) scores 0.
//!
//! Valuation: `P/E <= 15 → full, <= 25 → 80%, <= 35 → 60%, <= 50 → 40%, else → 20%`;
//! a loss-making company (P/E <= 0) scores 0 with an explanatory note.
//!
//! Subscription: `>= 100× → full, >= 50× → 87%, >= 25× → 73%, >= 10× → 60%,
//! >= 3× → 47%, >= 1× → 33%, else → 20%`.
//!
//! Risk: starts at 5; `-2` SME board, `-2` high leverage (D/E > 2), `-1`
//! elevated leverage (1 < D/E <= 2), `-1` disclosed risk factors; floored at 0.
//! Only scored when at least one real risk signal exists.
//!
//! Growth figures (YoY and CAGR) are computed deterministically from the
//! financial period series; a negative or zero base yields "Not available"
//! rather than a meaningless percentage. YoY/CAGR are never invented.
//!
//! This is NOT investment advice and never a guarantee of returns.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::modules::ipo_intel::models::MetricAnalysis;

pub const SCORE_METHODOLOGY_VERSION: &str = "2.0";

pub const SCORE_DISCLAIMER: &str =
    "The InvestIQ IPO Score is an educational, transparent composite of available \
     data. It is not investment advice and does not guarantee returns.";

/// A single financial period used for growth analytics. `period_end` is used
/// to compute the time span for CAGR.
#[derive(Debug, Clone, Default)]
pub struct SeriesPoint {
    pub period: String,
    pub value: Option<Decimal>,
    pub period_end: Option<NaiveDate>,
}

impl SeriesPoint {
    pub fn new(period: impl Into<String>, value: Option<Decimal>) -> Self {
        Self {
            period: period.into(),
            value,
            period_end: None,
        }
    }
}

/// Everything the scoring logic needs about an IPO. All fields are optional
/// except the board; missing values yield "insufficient data" components.
#[derive(Debug, Clone, Default)]
pub struct ScoreInputs {
    pub revenue_growth_pct: Option<Decimal>,
    pub pat_growth_pct: Option<Decimal>,
    pub eps_growth_pct: Option<Decimal>,
    pub ebitda_margin_pct: Option<Decimal>,
    pub pat_margin_pct: Option<Decimal>,
    pub roe_pct: Option<Decimal>,
    pub roce_pct: Option<Decimal>,
    pub pe_ratio: Option<Decimal>,
    pub issue_price: Option<Decimal>,
    pub eps: Option<Decimal>,
    pub debt_to_equity: Option<Decimal>,
    pub subscription_overall: Option<Decimal>,
    pub sector: Option<String>,
    pub board: String,
    pub has_risks: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentScore {
    pub key: &'static str,
    pub label: &'static str,
    pub max_points: u32,
    pub score: Option<f64>,
    pub explanation: String,
}

impl ComponentScore {
    fn insufficient(key: &'static str, label: &'static str, max: u32, why: &str) -> Self {
        Self {
            key,
            label,
            max_points: max,
            score: None,
            explanation: why.to_string(),
        }
    }

    fn scored(key: &'static str, label: &'static str, max: u32, score: f64, why: String) -> Self {
        Self {
            key,
            label,
            max_points: max,
            score: Some(score.clamp(0.0, max as f64)),
            explanation: why,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoreResult {
    pub total: Option<f64>,
    pub max_points: u32,
    pub components: Vec<ComponentScore>,
    pub positive_factors: Vec<String>,
    pub concerns: Vec<String>,
    pub data_quality: DataQuality,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataQuality {
    pub overall: &'static str,
    pub missing: Vec<String>,
}

pub(crate) fn as_f64(d: Option<Decimal>) -> Option<f64> {
    d.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0))
}

fn round_dec(v: f64) -> Decimal {
    Decimal::from_f64_retain((v * 100.0).round() / 100.0).unwrap_or_default()
}

/// Year-over-year growth percentage `(current - previous) / previous * 100`.
///
/// Returns `None` when either figure is missing, the previous period is zero
/// (division by zero / undefined), or the previous base is negative (a
/// percentage change on a loss base is not meaningful). Negative/zero current
/// values are handled correctly: a decline yields a negative percentage.
pub fn yoy_growth(current: Option<Decimal>, previous: Option<Decimal>) -> Option<Decimal> {
    let c = as_f64(current)?;
    let p = as_f64(previous)?;
    if p <= 0.0 || !c.is_finite() || !p.is_finite() {
        return None;
    }
    Some(round_dec((c - p) / p * 100.0))
}

/// Compound annual growth rate over `years`, `((end / start)^(1/years) - 1) * 100`.
///
/// Returns `None` when the start or end value is missing/non-positive, or when
/// `years` is missing/non-positive — a CAGR on a loss or zero base is never
/// invented.
pub fn cagr(end_value: Option<Decimal>, start_value: Option<Decimal>, years: Option<Decimal>) -> Option<Decimal> {
    let end = as_f64(end_value)?;
    let start = as_f64(start_value)?;
    let years = as_f64(years)?;
    if start <= 0.0 || end <= 0.0 || years <= 0.0 || !end.is_finite() || !start.is_finite() || !years.is_finite() {
        return None;
    }
    let ratio = end / start;
    if ratio <= 0.0 {
        return None;
    }
    Some(round_dec((ratio.powf(1.0 / years) - 1.0) * 100.0))
}

/// YoY growth of the latest period against the previous available period in a
/// chronological series. Returns `None` when there are fewer than two data
/// points or the base is non-positive.
pub fn latest_yoy(points: &[SeriesPoint]) -> Option<Decimal> {
    let with_value: Vec<&SeriesPoint> = points.iter().filter(|p| p.value.is_some()).collect();
    let current = with_value.last()?;
    let previous = with_value.iter().rev().nth(1)?;
    yoy_growth(current.value, previous.value)
}

/// Years between two financial period-end dates. Returns `None` if either date
/// is missing or the span is non-positive.
fn period_years(start: Option<NaiveDate>, end: Option<NaiveDate>) -> Option<Decimal> {
    let days = end.and_then(|e| start.map(|s| (e - s).num_days()))?;
    if days <= 0 {
        return None;
    }
    Some(round_dec(days as f64 / 365.25))
}

/// Deterministic growth analysis for one metric across a chronological series.
/// Never invents figures: latest value, latest YoY and the CAGR over the
/// available span are each reported independently as `Option`.
pub fn analyze_series(label: &'static str, points: &[SeriesPoint]) -> MetricAnalysis {
    let with_value: Vec<&SeriesPoint> = points.iter().filter(|p| p.value.is_some()).collect();

    let latest = with_value.last();
    let yoy = latest_yoy(points);

    let cagr = if with_value.len() >= 2 {
        let start = with_value.first();
        let end = with_value.last();
        let years = period_years(start.and_then(|s| s.period_end), end.and_then(|e| e.period_end));
        match years {
            Some(y) => cagr(
                end.and_then(|e| e.value),
                start.and_then(|s| s.value),
                Some(y),
            )
            .map(|c| (c, start.expect("start exists").period.clone(), y)),
            None => None,
        }
    } else {
        None
    };

    MetricAnalysis {
        label: label.to_string(),
        latest_value: latest.and_then(|p| p.value),
        latest_period: latest.map(|p| p.period.clone()),
        yoy_growth_pct: yoy,
        cagr_pct: cagr.as_ref().map(|(c, _, _)| *c),
        cagr_start_period: cagr.as_ref().map(|(_, p, _)| p.clone()),
        cagr_years: cagr.as_ref().map(|(_, _, y)| *y),
    }
}

/// Revenue Growth (0–15). Negative growth scores 0.
pub(crate) fn revenue_growth(growth_pct: Option<Decimal>) -> ComponentScore {
    let (key, label, max) = ("revenue_growth", "Revenue Growth", 15u32);
    let Some(g) = as_f64(growth_pct) else {
        return ComponentScore::insufficient(
            key,
            label,
            max,
            "Insufficient data: revenue growth is not available.",
        );
    };
    let score = if g >= 25.0 {
        15.0
    } else if g >= 15.0 {
        12.0
    } else if g >= 10.0 {
        9.0
    } else if g >= 5.0 {
        6.0
    } else if g >= 0.0 {
        3.0
    } else {
        0.0
    };
    let why = if g < 0.0 {
        format!("Revenue declined {g:.1}% YoY.")
    } else {
        format!("Revenue growth of {g:.1}% YoY.")
    };
    ComponentScore::scored(key, label, max, score, why)
}

/// EPS / PAT Growth (0–20). EPS growth is primary; PAT growth is used when EPS
/// is unavailable, and the two are blended 50/50 when both are present.
pub(crate) fn earnings_growth(
    eps_growth_pct: Option<Decimal>,
    pat_growth_pct: Option<Decimal>,
) -> ComponentScore {
    let (key, label, max) = ("earnings_growth", "EPS / PAT Growth", 20u32);
    fn pts(g: f64) -> f64 {
        if g >= 25.0 {
            20.0
        } else if g >= 15.0 {
            16.0
        } else if g >= 10.0 {
            12.0
        } else if g >= 5.0 {
            8.0
        } else if g >= 0.0 {
            4.0
        } else {
            0.0
        }
    }
    let eg = as_f64(eps_growth_pct);
    let pg = as_f64(pat_growth_pct);
    match (eg, pg) {
        (None, None) => ComponentScore::insufficient(
            key,
            label,
            max,
            "Insufficient data: no EPS or PAT growth figures available.",
        ),
        (Some(e), Some(p)) => ComponentScore::scored(
            key,
            label,
            max,
            (pts(e) + pts(p)) / 2.0,
            format!("EPS growth {e:.1}% and PAT growth {p:.1}% are blended 50/50."),
        ),
        (Some(e), None) => ComponentScore::scored(
            key,
            label,
            max,
            pts(e),
            format!("EPS growth of {e:.1}% YoY."),
        ),
        (None, Some(p)) => ComponentScore::scored(
            key,
            label,
            max,
            pts(p),
            format!("PAT growth of {p:.1}% YoY (EPS growth unavailable)."),
        ),
    }
}

/// Profitability & Margins (0–15): average of every available profitability
/// signal (PAT margin, EBITDA margin, ROE, ROCE), each scaled with a 35%
/// ceiling for full marks and 0 for non-positive values.
pub(crate) fn profitability(
    pat_margin_pct: Option<Decimal>,
    ebitda_margin_pct: Option<Decimal>,
    roe_pct: Option<Decimal>,
    roce_pct: Option<Decimal>,
) -> ComponentScore {
    let (key, label, max) = ("profitability", "Profitability & Margins", 15u32);
    let signals: Vec<(&str, f64)> = [
        ("PAT margin", pat_margin_pct),
        ("EBITDA margin", ebitda_margin_pct),
        ("ROE", roe_pct),
        ("ROCE", roce_pct),
    ]
    .into_iter()
    .filter_map(|(name, v)| as_f64(v).map(|x| (name, x)))
    .collect();

    if signals.is_empty() {
        return ComponentScore::insufficient(
            key,
            label,
            max,
            "Insufficient data: no margin or ROE/ROCE figures available.",
        );
    }
    let avg = signals.iter().map(|(_, s)| s.clamp(0.0, 35.0) / 35.0 * 15.0).sum::<f64>()
        / signals.len() as f64;
    let why = format!(
        "Average of available profitability signals ({}).",
        signals
            .iter()
            .map(|(n, s)| format!("{n} {s:.1}%"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    ComponentScore::scored(key, label, max, avg, why)
}

/// Balance Sheet / Debt (0–10) from the debt-to-equity ratio.
pub(crate) fn balance_sheet(debt_to_equity: Option<Decimal>) -> ComponentScore {
    let (key, label, max) = ("balance_sheet", "Balance Sheet / Debt", 10u32);
    let Some(de) = as_f64(debt_to_equity) else {
        return ComponentScore::insufficient(
            key,
            label,
            max,
            "Insufficient data: debt-to-equity ratio is not available.",
        );
    };
    let (score, why) = if de < 0.0 {
        (0.0, "Negative debt-to-equity indicates negative net worth.".to_string())
    } else if de <= 0.5 {
        (10.0, format!("Debt-to-equity of {de:.2} is low."))
    } else if de <= 1.0 {
        (8.0, format!("Debt-to-equity of {de:.2} is moderate."))
    } else if de <= 2.0 {
        (6.0, format!("Debt-to-equity of {de:.2} is elevated."))
    } else {
        (3.0, format!("Debt-to-equity of {de:.2} is high."))
    };
    ComponentScore::scored(key, label, max, score, why)
}

/// Valuation (0–15). Uses the vendor P/E when present, otherwise derives it as
/// `issue price / EPS`. Loss-making companies (P/E <= 0) score 0.
pub(crate) fn valuation(
    pe_ratio: Option<Decimal>,
    issue_price: Option<Decimal>,
    eps: Option<Decimal>,
) -> ComponentScore {
    let (key, label, max) = ("valuation", "Valuation", 15u32);
    let pe = as_f64(pe_ratio).or_else(|| match (as_f64(issue_price), as_f64(eps)) {
        (Some(p), Some(e)) if e > 0.0 => Some(p / e),
        _ => None,
    });
    let Some(pe) = pe else {
        return ComponentScore::insufficient(
            key,
            label,
            max,
            "Insufficient data: no P/E ratio or EPS available.",
        );
    };
    let (score, why) = if pe <= 0.0 {
        (0.0, "Loss-making: P/E is not meaningful for non-positive earnings.".to_string())
    } else if pe <= 15.0 {
        (15.0, format!("Price-to-earnings ratio of {pe:.1} is low."))
    } else if pe <= 25.0 {
        (12.0, format!("Price-to-earnings ratio of {pe:.1} is reasonable."))
    } else if pe <= 35.0 {
        (9.0, format!("Price-to-earnings ratio of {pe:.1} is elevated."))
    } else if pe <= 50.0 {
        (6.0, format!("Price-to-earnings ratio of {pe:.1} is high."))
    } else {
        (3.0, format!("Price-to-earnings ratio of {pe:.1} is very high."))
    };
    ComponentScore::scored(key, label, max, score, why)
}

/// IPO Subscription (0–15) from the official exchange subscription multiple.
pub(crate) fn subscription(overall: Option<Decimal>) -> ComponentScore {
    let (key, label, max) = ("subscription", "IPO Subscription", 15u32);
    let Some(s) = as_f64(overall) else {
        return ComponentScore::insufficient(
            key,
            label,
            max,
            "Insufficient data: subscription multiple is not available.",
        );
    };
    let (score, why) = if s >= 100.0 {
        (15.0, format!("Overall subscription of {s:.1}x is exceptional."))
    } else if s >= 50.0 {
        (13.0, format!("Overall subscription of {s:.1}x is very strong."))
    } else if s >= 25.0 {
        (11.0, format!("Overall subscription of {s:.1}x is strong."))
    } else if s >= 10.0 {
        (9.0, format!("Overall subscription of {s:.1}x is healthy."))
    } else if s >= 3.0 {
        (7.0, format!("Overall subscription of {s:.1}x is moderate."))
    } else if s >= 1.0 {
        (5.0, format!("Overall subscription of {s:.1}x is subdued."))
    } else {
        (0.0, format!("Overall subscription of {s:.1}x is below 1x."))
    };
    ComponentScore::scored(key, label, max, score, why)
}

/// Industry / Business Quality (0–5). Requires a sector benchmark dataset which
/// is NOT available in v1, so this component is always "insufficient data".
fn industry_business(_sector: Option<String>) -> ComponentScore {
    ComponentScore::insufficient(
        "industry",
        "Industry / Business Quality",
        5,
        "Insufficient data: no sector benchmark dataset is available \
         (a paid industry-data provider would be required).",
    )
}

/// Risk Factors (0–5, higher = lower risk). Penalties for SME board, high
/// leverage, and disclosed risk items. Only scored when at least one real risk
/// signal exists; a mainboard with no data yields "insufficient data".
pub(crate) fn risk(board: &str, debt_to_equity: Option<Decimal>, has_risks: bool) -> ComponentScore {
    let (key, label, max) = ("risk", "Risk Factors", 5u32);
    let sme = board.eq_ignore_ascii_case("sme");
    if debt_to_equity.is_none() && !has_risks && !sme {
        return ComponentScore::insufficient(
            key,
            label,
            max,
            "Insufficient data: no leverage or risk disclosures available.",
        );
    }
    let mut score = 5.0;
    let mut notes = Vec::new();
    if sme {
        score -= 2.0;
        notes.push("SME board (liquidity risk)".to_string());
    }
    if let Some(de) = as_f64(debt_to_equity) {
        if de > 2.0 {
            score -= 2.0;
            notes.push(format!("High leverage (D/E {de:.1})"));
        } else if de > 1.0 {
            score -= 1.0;
            notes.push(format!("Elevated leverage (D/E {de:.1})"));
        }
    }
    if has_risks {
        score -= 1.0;
        notes.push("Disclosed risk factors".to_string());
    }
    if notes.is_empty() {
        notes.push("No material risk signals in available data".to_string());
    }
    ComponentScore::scored(key, label, max, score, notes.join("; "))
}

fn positive_factors(inputs: &ScoreInputs) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(g) = as_f64(inputs.revenue_growth_pct) {
        if g > 0.0 {
            out.push("Positive revenue growth".to_string());
        }
    }
    if let Some(g) = as_f64(inputs.eps_growth_pct).or(as_f64(inputs.pat_growth_pct)) {
        if g > 0.0 {
            out.push("Positive earnings growth".to_string());
        }
    }
    if let Some(m) = as_f64(inputs.pat_margin_pct) {
        if m > 0.0 {
            out.push("Profitable (positive PAT margin)".to_string());
        }
    }
    if let Some(em) = as_f64(inputs.ebitda_margin_pct) {
        if em >= 15.0 {
            out.push("Strong EBITDA margin".to_string());
        }
    }
    if let Some(s) = as_f64(inputs.subscription_overall) {
        if s >= 10.0 {
            out.push("Strong overall subscription".to_string());
        } else if s >= 3.0 {
            out.push("Healthy subscription interest".to_string());
        }
    }
    if let Some(pe) = as_f64(inputs.pe_ratio) {
        if pe > 0.0 && pe <= 25.0 {
            out.push("Reasonable valuation".to_string());
        }
    }
    if let Some(de) = as_f64(inputs.debt_to_equity) {
        if (0.0..=1.0).contains(&de) {
            out.push("Low leverage".to_string());
        }
    }
    out
}

fn concerns(inputs: &ScoreInputs) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(g) = as_f64(inputs.revenue_growth_pct) {
        if g < 0.0 {
            out.push("Declining revenue".to_string());
        }
    }
    if let Some(g) = as_f64(inputs.eps_growth_pct).or(as_f64(inputs.pat_growth_pct)) {
        if g < 0.0 {
            out.push("Declining earnings".to_string());
        }
    }
    if let Some(m) = as_f64(inputs.pat_margin_pct) {
        if m <= 0.0 {
            out.push("Not profitable (non-positive PAT margin)".to_string());
        }
    }
    if let Some(pe) = as_f64(inputs.pe_ratio) {
        if pe > 35.0 {
            out.push("High valuation (P/E > 35)".to_string());
        }
    }
    if let Some(s) = as_f64(inputs.subscription_overall) {
        if s < 1.0 {
            out.push("Weak subscription".to_string());
        }
    }
    if let Some(de) = as_f64(inputs.debt_to_equity) {
        if de < 0.0 {
            out.push("Negative net worth (negative debt-to-equity)".to_string());
        } else if de > 2.0 {
            out.push("High debt-to-equity".to_string());
        }
    }
    if inputs.board.eq_ignore_ascii_case("sme") {
        out.push("SME board liquidity risk".to_string());
    }
    if inputs.has_risks {
        out.push("Disclosed risk factors present".to_string());
    }
    out
}

/// Compute the full InvestIQ IPO Score from available inputs.
///
/// The industry component requires a sector benchmark dataset that is not yet
/// available, so it never contributes a score and is excluded from the data
/// quality report to keep the score comparable over time. The total is
/// re-normalized to /100 across all other components that could be scored.
pub fn compute_score(inputs: ScoreInputs) -> ScoreResult {
    let components = vec![
        revenue_growth(inputs.revenue_growth_pct),
        earnings_growth(inputs.eps_growth_pct, inputs.pat_growth_pct),
        profitability(
            inputs.pat_margin_pct,
            inputs.ebitda_margin_pct,
            inputs.roe_pct,
            inputs.roce_pct,
        ),
        balance_sheet(inputs.debt_to_equity),
        valuation(inputs.pe_ratio, inputs.issue_price, inputs.eps),
        subscription(inputs.subscription_overall),
        industry_business(inputs.sector.clone()),
        risk(&inputs.board, inputs.debt_to_equity, inputs.has_risks),
    ];

    let mut scored_max = 0.0;
    let mut scored_sum = 0.0;
    let mut missing = Vec::new();
    for c in &components {
        if c.key == "industry" {
            continue;
        }
        match c.score {
            Some(s) => {
                scored_max += c.max_points as f64;
                scored_sum += s;
            }
            None => missing.push(c.key.to_string()),
        }
    }

    let data_quality = if missing.is_empty() {
        DataQuality {
            overall: "complete",
            missing,
        }
    } else if scored_max == 0.0 {
        DataQuality {
            overall: "insufficient",
            missing,
        }
    } else {
        DataQuality {
            overall: "partial",
            missing,
        }
    };

    let total = if scored_max > 0.0 {
        Some((scored_sum / scored_max) * 100.0)
    } else {
        None
    };

    ScoreResult {
        total,
        max_points: 100,
        components,
        positive_factors: positive_factors(&inputs),
        concerns: concerns(&inputs),
        data_quality,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(v: f64) -> Decimal {
        Decimal::from_str_exact(&format!("{v}")).unwrap_or_else(|_| Decimal::from(v as i64))
    }

    fn inputs() -> ScoreInputs {
        ScoreInputs {
            revenue_growth_pct: Some(dec(30.0)),
            pat_growth_pct: Some(dec(25.0)),
            eps_growth_pct: Some(dec(20.0)),
            ebitda_margin_pct: Some(dec(20.0)),
            pat_margin_pct: Some(dec(12.0)),
            roe_pct: Some(dec(15.0)),
            roce_pct: Some(dec(18.0)),
            pe_ratio: Some(dec(22.0)),
            debt_to_equity: Some(dec(0.8)),
            subscription_overall: Some(dec(40.0)),
            sector: Some("IT".to_string()),
            board: "mainboard".into(),
            has_risks: false,
            ..Default::default()
        }
    }

    // ---- YoY growth ----

    #[test]
    fn yoy_growth_calculated_and_guarded() {
        assert_eq!(yoy_growth(Some(dec(120.0)), Some(dec(100.0))), Some(dec(20.0)));
        assert_eq!(yoy_growth(Some(dec(80.0)), Some(dec(100.0))), Some(dec(-20.0)));
        assert_eq!(yoy_growth(Some(dec(100.0)), Some(dec(100.0))), Some(dec(0.0)));
        assert_eq!(yoy_growth(Some(dec(100.0)), None), None);
        assert_eq!(yoy_growth(None, Some(dec(100.0))), None);
        assert_eq!(yoy_growth(Some(dec(100.0)), Some(dec(0.0))), None);
        // Loss base: percentage change is meaningless.
        assert_eq!(yoy_growth(Some(dec(100.0)), Some(dec(-50.0))), None);
    }

    // ---- CAGR ----

    #[test]
    fn cagr_calculated_and_guarded() {
        // (121/100)^(1/2) - 1 = 10%
        let c = cagr(Some(dec(121.0)), Some(dec(100.0)), Some(dec(2.0))).unwrap();
        assert!((c.to_string().parse::<f64>().unwrap() - 10.0).abs() < 0.01);
        assert_eq!(cagr(None, Some(dec(100.0)), Some(dec(2.0))), None);
        assert_eq!(cagr(Some(dec(100.0)), None, Some(dec(2.0))), None);
        assert_eq!(cagr(Some(dec(100.0)), Some(dec(100.0)), None), None);
        assert_eq!(cagr(Some(dec(100.0)), Some(dec(0.0)), Some(dec(2.0))), None);
        assert_eq!(cagr(Some(dec(-10.0)), Some(dec(100.0)), Some(dec(2.0))), None);
        assert_eq!(cagr(Some(dec(100.0)), Some(dec(-50.0)), Some(dec(2.0))), None);
        assert_eq!(cagr(Some(dec(100.0)), Some(dec(100.0)), Some(dec(0.0))), None);
    }

    // ---- Series analysis ----

    #[test]
    fn analyze_series_with_full_history() {
        let start = NaiveDate::from_ymd_opt(2023, 3, 31).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 3, 31).unwrap();
        let points = vec![
            SeriesPoint {
                period: "FY2023".into(),
                value: Some(dec(100.0)),
                period_end: Some(start),
            },
            SeriesPoint {
                period: "FY2024".into(),
                value: Some(dec(110.0)),
                period_end: start.checked_add_months(chrono::Months::new(12)),
            },
            SeriesPoint {
                period: "FY2025".into(),
                value: Some(dec(121.0)),
                period_end: Some(end),
            },
        ];
        let m = analyze_series("Revenue", &points);
        assert_eq!(m.label, "Revenue");
        assert_eq!(m.latest_value, Some(dec(121.0)));
        assert_eq!(m.latest_period.as_deref(), Some("FY2025"));
        assert_eq!(m.yoy_growth_pct, Some(dec(10.0)));
        assert!(m.cagr_pct.is_some());
        assert_eq!(m.cagr_start_period.as_deref(), Some("FY2023"));
        assert!(m.cagr_years.is_some());
    }

    #[test]
    fn analyze_series_insufficient_history() {
        let m = analyze_series("Revenue", &[]);
        assert_eq!(m.latest_value, None);
        assert_eq!(m.yoy_growth_pct, None);
        assert_eq!(m.cagr_pct, None);

        let single = vec![SeriesPoint::new("FY2025", Some(dec(100.0)))];
        let m2 = analyze_series("Revenue", &single);
        assert_eq!(m2.latest_value, Some(dec(100.0)));
        assert_eq!(m2.yoy_growth_pct, None); // no previous period
        assert_eq!(m2.cagr_pct, None); // fewer than two data points
    }

    #[test]
    fn analyze_series_negative_base_is_not_available() {
        let points = vec![
            SeriesPoint::new("FY2024", Some(dec(-50.0))),
            SeriesPoint::new("FY2025", Some(dec(25.0))),
        ];
        let m = analyze_series("PAT", &points);
        assert_eq!(m.yoy_growth_pct, None); // loss base
        assert_eq!(m.cagr_pct, None);
    }

    #[test]
    fn analyze_series_zero_base_is_not_available() {
        let points = vec![
            SeriesPoint::new("FY2024", Some(dec(0.0))),
            SeriesPoint::new("FY2025", Some(dec(100.0))),
        ];
        let m = analyze_series("Revenue", &points);
        assert_eq!(m.yoy_growth_pct, None);
        assert_eq!(m.cagr_pct, None);
    }

    #[test]
    fn analyze_series_ignores_missing_periods() {
        let points = vec![
            SeriesPoint::new("FY2024", None),
            SeriesPoint::new("FY2025", Some(dec(110.0))),
            SeriesPoint::new("FY2026", Some(dec(121.0))),
        ];
        let m = analyze_series("Revenue", &points);
        assert_eq!(m.latest_value, Some(dec(121.0)));
        assert_eq!(m.yoy_growth_pct, Some(dec(10.0)));
    }

    // ---- Component: Revenue Growth ----

    #[test]
    fn revenue_growth_scoring() {
        assert_eq!(revenue_growth(Some(dec(30.0))).score, Some(15.0));
        assert_eq!(revenue_growth(Some(dec(15.0))).score, Some(12.0));
        assert_eq!(revenue_growth(Some(dec(10.0))).score, Some(9.0));
        assert_eq!(revenue_growth(Some(dec(5.0))).score, Some(6.0));
        assert_eq!(revenue_growth(Some(dec(0.0))).score, Some(3.0));
        assert_eq!(revenue_growth(Some(dec(-10.0))).score, Some(0.0));
        assert_eq!(revenue_growth(None).score, None);
    }

    // ---- Component: EPS / PAT Growth ----

    #[test]
    fn earnings_growth_uses_eps_and_pat() {
        let both = earnings_growth(Some(dec(30.0)), Some(dec(20.0)));
        assert_eq!(both.score, Some(18.0)); // (20 + 16) / 2
        assert_eq!(earnings_growth(Some(dec(30.0)), None).score, Some(20.0));
        assert_eq!(earnings_growth(None, Some(dec(30.0))).score, Some(20.0));
        assert_eq!(earnings_growth(Some(dec(-5.0)), None).score, Some(0.0));
        assert_eq!(earnings_growth(Some(dec(7.0)), Some(dec(-3.0))).score, Some(4.0));
        assert_eq!(earnings_growth(None, None).score, None);
    }

    // ---- Component: Profitability ----

    #[test]
    fn profitability_averages_signals() {
        let p = profitability(Some(dec(35.0)), None, None, None);
        assert_eq!(p.score, Some(15.0));
        let p2 = profitability(None, None, Some(dec(35.0)), None);
        assert_eq!(p2.score, Some(15.0));
        // ROE 17.5% → half marks
        let p3 = profitability(None, None, Some(dec(17.5)), None);
        assert!((p3.score.unwrap() - 7.5).abs() < 0.001);
        // Negative signals score 0
        let p4 = profitability(Some(dec(-5.0)), None, None, None);
        assert_eq!(p4.score, Some(0.0));
        assert_eq!(profitability(None, None, None, None).score, None);
    }

    // ---- Component: Balance Sheet ----

    #[test]
    fn balance_sheet_scoring() {
        assert_eq!(balance_sheet(Some(dec(0.3))).score, Some(10.0));
        assert_eq!(balance_sheet(Some(dec(0.8))).score, Some(8.0));
        assert_eq!(balance_sheet(Some(dec(1.5))).score, Some(6.0));
        assert_eq!(balance_sheet(Some(dec(5.0))).score, Some(3.0));
        assert_eq!(balance_sheet(Some(dec(-0.5))).score, Some(0.0));
        assert_eq!(balance_sheet(None).score, None);
    }

    // ---- Component: Valuation ----

    #[test]
    fn valuation_scoring() {
        assert_eq!(valuation(Some(dec(12.0)), None, None).score, Some(15.0));
        assert_eq!(valuation(Some(dec(20.0)), None, None).score, Some(12.0));
        assert_eq!(valuation(Some(dec(30.0)), None, None).score, Some(9.0));
        assert_eq!(valuation(Some(dec(40.0)), None, None).score, Some(6.0));
        assert_eq!(valuation(Some(dec(60.0)), None, None).score, Some(3.0));
        // Loss-making: P/E non-positive scores 0, not insufficient.
        let loss = valuation(Some(dec(-8.0)), None, None);
        assert_eq!(loss.score, Some(0.0));
        assert!(loss.explanation.contains("Loss-making"));
        // Implied P/E from issue price / EPS.
        assert_eq!(valuation(None, Some(dec(120.0)), Some(dec(10.0))).score, Some(15.0));
        // EPS zero → no implied P/E.
        assert_eq!(valuation(None, Some(dec(120.0)), Some(dec(0.0))).score, None);
        assert_eq!(valuation(None, None, None).score, None);
    }

    // ---- Component: Subscription ----

    #[test]
    fn subscription_scoring() {
        assert_eq!(subscription(Some(dec(150.0))).score, Some(15.0));
        assert_eq!(subscription(Some(dec(60.0))).score, Some(13.0));
        assert_eq!(subscription(Some(dec(30.0))).score, Some(11.0));
        assert_eq!(subscription(Some(dec(12.0))).score, Some(9.0));
        assert_eq!(subscription(Some(dec(5.0))).score, Some(7.0));
        assert_eq!(subscription(Some(dec(1.0))).score, Some(5.0));
        assert_eq!(subscription(Some(dec(0.5))).score, Some(0.0));
        assert_eq!(subscription(None).score, None);
    }

    // ---- Component: Industry ----

    #[test]
    fn industry_always_insufficient_without_benchmark() {
        let i = industry_business(Some("IT".to_string()));
        assert_eq!(i.score, None);
        assert_eq!(i.max_points, 5);
        let i2 = industry_business(None);
        assert_eq!(i2.score, None);
    }

    // ---- Component: Risk ----

    #[test]
    fn risk_penalizes_sme_debt_and_risks() {
        let r = risk("mainboard", Some(dec(0.5)), false);
        assert_eq!(r.score, Some(5.0));
        let r2 = risk("sme", Some(dec(3.0)), true);
        assert_eq!(r2.score, Some(0.0)); // 5 - 2 - 2 - 1
        let r3 = risk("mainboard", Some(dec(1.5)), false);
        assert_eq!(r3.score, Some(4.0));
        let r4 = risk("mainboard", None, false);
        assert_eq!(r4.score, None); // no risk signals → insufficient
    }

    // ---- Overall score ----

    #[test]
    fn total_complete_when_all_data() {
        let res = compute_score(inputs());
        assert_eq!(res.data_quality.overall, "complete");
        assert!(res.total.unwrap() > 0.0 && res.total.unwrap() <= 100.0);
        assert_eq!(res.max_points, 100);
        assert_eq!(res.components.len(), 8);
    }

    #[test]
    fn total_renormalizes_when_partial() {
        let inputs = ScoreInputs {
            subscription_overall: Some(dec(10.0)), // 9/15
            board: "mainboard".into(),
            ..Default::default()
        };
        let res = compute_score(inputs);
        assert_eq!(res.data_quality.overall, "partial");
        // total = 9/15 * 100 = 60
        assert!((res.total.unwrap() - 60.0).abs() < 0.001);
        assert!(res.data_quality.missing.contains(&"revenue_growth".to_string()));
    }

    #[test]
    fn total_insufficient_when_nothing_scorable() {
        let res = compute_score(ScoreInputs::default());
        assert_eq!(res.data_quality.overall, "insufficient");
        assert_eq!(res.total, None);
    }

    #[test]
    fn negative_earnings_score_deterministic() {
        let inputs = ScoreInputs {
            revenue_growth_pct: Some(dec(10.0)),
            pat_margin_pct: Some(dec(-12.0)),
            eps: Some(dec(-2.0)),
            pe_ratio: Some(dec(-10.0)),
            debt_to_equity: Some(dec(1.0)),
            subscription_overall: Some(dec(5.0)),
            board: "mainboard".into(),
            ..Default::default()
        };
        let res = compute_score(inputs);
        assert!(res.total.unwrap() < 60.0);
        assert!(res.concerns.iter().any(|c| c.contains("Not profitable")));
    }

    #[test]
    fn zero_values_handled_without_invention() {
        let inputs = ScoreInputs {
            revenue_growth_pct: Some(dec(0.0)),
            eps_growth_pct: Some(dec(0.0)),
            pat_margin_pct: Some(dec(0.0)),
            debt_to_equity: Some(dec(0.0)),
            pe_ratio: Some(dec(0.0)),
            subscription_overall: Some(dec(0.0)),
            board: "mainboard".into(),
            ..Default::default()
        };
        let res = compute_score(inputs);
        assert_eq!(res.data_quality.overall, "complete");
        // All components scored; low-growth, break-even, fully-subscribed-below-1x.
        assert_eq!(res.components.iter().filter(|c| c.score.is_some()).count(), 7);
    }

    #[test]
    fn total_never_exceeds_100() {
        let mut i = inputs();
        // Extreme values in every component.
        i.revenue_growth_pct = Some(dec(200.0));
        i.eps_growth_pct = Some(dec(200.0));
        i.pat_growth_pct = Some(dec(200.0));
        i.pat_margin_pct = Some(dec(100.0));
        i.ebitda_margin_pct = Some(dec(100.0));
        i.roe_pct = Some(dec(100.0));
        i.roce_pct = Some(dec(100.0));
        i.pe_ratio = Some(dec(1.0));
        i.debt_to_equity = Some(dec(0.0));
        i.subscription_overall = Some(dec(1000.0));
        let res = compute_score(i);
        assert_eq!(res.total.unwrap(), 100.0);
    }
}
