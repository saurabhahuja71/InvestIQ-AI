//! Milestone 5 — IPO Investment Decision Engine.
//!
//! A deterministic, reproducible analysis of an IPO from structured data only.
//! No LLM is involved and no value is ever invented: every score, view and
//! factor below is a pure function of the inputs passed in. The exact rules,
//! thresholds and missing-data handling are documented in
//! `docs/IPO_DECISION_ENGINE.md` and versioned via [`ANALYSIS_METHODOLOGY_VERSION`].
//!
//! The engine produces two distinct investment horizons that must never be
//! conflated:
//!
//! * **Long-term view** — driven by fundamentals (growth, profitability,
//!   balance sheet) tempered by valuation.
//! * **Listing / short-term view** — driven by official subscription demand,
//!   valuation and risk. GMP is intentionally excluded; `market_sentiment` is
//!   reported as "Not Evaluated" until a real market-sentiment provider exists.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::modules::ipo_intel::logic::{
    analyze_series, balance_sheet, compute_score, earnings_growth, profitability,
    revenue_growth, risk, subscription, valuation, ComponentScore, ScoreInputs, SeriesPoint,
};

pub const ANALYSIS_METHODOLOGY_VERSION: &str = "1.0";

pub const ANALYSIS_DISCLAIMER: &str = "InvestIQ Analysis is based on available public data and \
     can be wrong. It is not investment advice and does not guarantee returns.";

/// View thresholds (documented in docs/IPO_DECISION_ENGINE.md).
const VIEW_STRONG: f64 = 70.0;
const VIEW_POSITIVE: f64 = 55.0;
const VIEW_NEUTRAL: f64 = 40.0;
const VIEW_CAUTION: f64 = 25.0;

/// Risk penalty applied to the long-term score when the risk component scores
/// below this threshold (SME board and/or high leverage and/or disclosed risks).
const LONG_TERM_RISK_CAP: f64 = 40.0;
const LONG_TERM_RISK_PENALTY: f64 = 6.0;

/// Everything the decision engine needs about one IPO. All fields optional
/// except the board; missing values are treated as "not available", never as
/// fabricated numbers.
#[derive(Debug, Clone, Default)]
pub struct AnalysisInputs {
    pub board: String,
    pub sector: Option<String>,
    // Subscription (official exchange data).
    pub subscription_overall: Option<Decimal>,
    pub subscription_qib: Option<Decimal>,
    pub subscription_nii: Option<Decimal>,
    pub subscription_retail: Option<Decimal>,
    // Issue / valuation.
    pub issue_price: Option<Decimal>,
    // Latest-period fundamentals.
    pub eps: Option<Decimal>,
    pub pe_ratio: Option<Decimal>,
    pub debt_to_equity: Option<Decimal>,
    pub ebitda_margin_pct: Option<Decimal>,
    pub pat_margin_pct: Option<Decimal>,
    pub roe_pct: Option<Decimal>,
    pub roce_pct: Option<Decimal>,
    pub revenue_growth_pct: Option<Decimal>,
    pub pat_growth_pct: Option<Decimal>,
    pub eps_growth_pct: Option<Decimal>,
    // Chronological series used for CAGR / consistency.
    pub revenue_series: Vec<SeriesPoint>,
    pub pat_series: Vec<SeriesPoint>,
    pub eps_series: Vec<SeriesPoint>,
    pub period_labels: Vec<String>,
    // Retrieval timestamps (reported so decisions stay auditable).
    pub financials_retrieved_at: Option<DateTime<Utc>>,
    pub subscription_updated_at: Option<DateTime<Utc>>,
    pub ipo_synced_at: Option<DateTime<Utc>>,
    pub has_risks: bool,
    pub risk_factors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpoView {
    StrongPositive,
    Positive,
    Neutral,
    Caution,
    Negative,
    InsufficientData,
}

impl IpoView {
    pub fn as_str(&self) -> &'static str {
        match self {
            IpoView::StrongPositive => "STRONG POSITIVE",
            IpoView::Positive => "POSITIVE",
            IpoView::Neutral => "NEUTRAL",
            IpoView::Caution => "CAUTION",
            IpoView::Negative => "NEGATIVE",
            IpoView::InsufficientData => "INSUFFICIENT DATA",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
    InsufficientData,
}

impl Confidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Confidence::High => "HIGH",
            Confidence::Medium => "MEDIUM",
            Confidence::Low => "LOW",
            Confidence::InsufficientData => "INSUFFICIENT DATA",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub overall_score: Option<f64>,
    pub fundamental_score: Option<f64>,
    pub growth_score: Option<f64>,
    pub profitability_score: Option<f64>,
    pub balance_sheet_score: Option<f64>,
    pub valuation_score: Option<f64>,
    pub subscription_score: Option<f64>,
    pub risk_score: Option<f64>,
    pub long_term_view: IpoView,
    pub listing_view: IpoView,
    pub confidence: Confidence,
    pub data_completeness: f64,
    pub positive_factors: Vec<(String, Option<String>)>,
    pub negative_factors: Vec<(String, Option<String>)>,
    pub missing_data: Vec<String>,
    pub financial_periods: Option<String>,
    pub financials_retrieved_at: Option<DateTime<Utc>>,
    pub subscription_updated_at: Option<DateTime<Utc>>,
    pub ipo_synced_at: Option<DateTime<Utc>>,
}

fn f64v(d: Option<Decimal>) -> Option<f64> {
    d.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0))
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn norm(pts: Option<f64>, max: f64) -> Option<f64> {
    pts.filter(|_| max > 0.0).map(|p| round1((p / max) * 100.0))
}

/// Weighted composite of scored components, re-normalized to /100 over the
/// components that could actually be scored (missing data never invents a 0).
fn combine(components: &[(&ComponentScore, f64)]) -> Option<f64> {
    let mut sum = 0.0;
    let mut max = 0.0;
    for (c, m) in components {
        if let Some(s) = c.score {
            sum += s;
            max += m;
        }
    }
    norm(Some(sum), max)
}

fn to_view(score: f64) -> IpoView {
    if score >= VIEW_STRONG {
        IpoView::StrongPositive
    } else if score >= VIEW_POSITIVE {
        IpoView::Positive
    } else if score >= VIEW_NEUTRAL {
        IpoView::Neutral
    } else if score >= VIEW_CAUTION {
        IpoView::Caution
    } else {
        IpoView::Negative
    }
}

/// Long-term score: fundamentals (75%) + valuation (25%). A long-term view is
/// only meaningful with fundamentals; valuation-only yields INSUFFICIENT DATA.
fn long_term_score(fundamental: Option<f64>, valuation: Option<f64>) -> Option<f64> {
    match (fundamental, valuation) {
        (Some(f), Some(v)) => Some(round1(0.75 * f + 0.25 * v)),
        (Some(f), None) => Some(f),
        (None, _) => None,
    }
}

/// Listing score: subscription demand (55%) + valuation (35%) + risk (10%),
/// re-normalized when a component is unavailable.
fn listing_score(
    subscription: Option<f64>,
    valuation: Option<f64>,
    risk: Option<f64>,
) -> Option<f64> {
    let mut weighted = 0.0;
    let mut weight = 0.0;
    for (v, w) in [(subscription, 0.55), (valuation, 0.35), (risk, 0.10)] {
        if let Some(x) = v {
            weighted += x * w;
            weight += w;
        }
    }
    if weight <= 0.0 {
        return None;
    }
    Some(round1(weighted / weight))
}

/// Effective P/E used for valuation factors: vendor P/E, else implied
/// `issue price / EPS` (only when earnings are positive).
fn effective_pe(pe_ratio: Option<Decimal>, issue_price: Option<Decimal>, eps: Option<Decimal>) -> Option<f64> {
    f64v(pe_ratio).or_else(|| match (f64v(issue_price), f64v(eps)) {
        (Some(p), Some(e)) if e > 0.0 => Some(p / e),
        _ => None,
    })
}

/// Build the deterministic analysis. Pure: same inputs → same outputs.
pub fn compute_analysis(i: &AnalysisInputs) -> AnalysisResult {
    // ---- Component scores (single source of truth = logic.rs thresholds) ----
    let rev = revenue_growth(i.revenue_growth_pct);
    let earn = earnings_growth(i.eps_growth_pct, i.pat_growth_pct);
    let prof = profitability(
        i.pat_margin_pct,
        i.ebitda_margin_pct,
        i.roe_pct,
        i.roce_pct,
    );
    let bal = balance_sheet(i.debt_to_equity);
    let val = valuation(i.pe_ratio, i.issue_price, i.eps);
    let sub = subscription(i.subscription_overall);
    let rsk = risk(&i.board, i.debt_to_equity, i.has_risks);

    // ---- Normalized dimension scores (0–100) ----
    let growth_score = combine(&[(&rev, 15.0), (&earn, 20.0)]);
    let profitability_score = norm(prof.score, 15.0);
    let balance_sheet_score = norm(bal.score, 10.0);
    let fundamental_score = combine(&[(&rev, 15.0), (&earn, 20.0), (&prof, 15.0), (&bal, 10.0)]);
    let valuation_score = norm(val.score, 15.0);
    let subscription_score = norm(sub.score, 15.0);
    let risk_score = norm(rsk.score, 5.0);

    // InvestIQ Score (identical to GET /ipos/{id}/score total).
    let score_inputs = ScoreInputs {
        revenue_growth_pct: i.revenue_growth_pct,
        pat_growth_pct: i.pat_growth_pct,
        eps_growth_pct: i.eps_growth_pct,
        ebitda_margin_pct: i.ebitda_margin_pct,
        pat_margin_pct: i.pat_margin_pct,
        roe_pct: i.roe_pct,
        roce_pct: i.roce_pct,
        pe_ratio: i.pe_ratio,
        issue_price: i.issue_price,
        eps: i.eps,
        debt_to_equity: i.debt_to_equity,
        subscription_overall: i.subscription_overall,
        sector: i.sector.clone(),
        board: i.board.clone(),
        has_risks: i.has_risks,
    };
    let overall_score = compute_score(score_inputs).total.map(round1);

    // ---- Views ----
    let mut lt_score = long_term_score(fundamental_score, valuation_score);
    if let (Some(rs), Some(s)) = (risk_score, lt_score) {
        if rs < LONG_TERM_RISK_CAP {
            lt_score = Some(round1((s - LONG_TERM_RISK_PENALTY).max(0.0)));
        }
    }
    let long_term_view = match lt_score {
        Some(s) => to_view(s),
        None => IpoView::InsufficientData,
    };

    let listing_view = match listing_score(subscription_score, valuation_score, risk_score) {
        Some(s) => to_view(s),
        None => IpoView::InsufficientData,
    };

    // ---- Data completeness ----
    let (data_completeness, missing_data) = completeness(i);

    let confidence = {
        let sub_present = i.subscription_overall.is_some();
        let val_present = valuation_score.is_some();
        let periods = i.period_labels.len();
        if data_completeness >= 85.0 && sub_present && val_present && periods >= 3 {
            Confidence::High
        } else if data_completeness >= 60.0 && periods >= 2 && (sub_present || val_present) {
            Confidence::Medium
        } else if data_completeness >= 30.0 {
            Confidence::Low
        } else {
            Confidence::InsufficientData
        }
    };

    // ---- Factors ----
    let rev_a = analyze_series("Revenue", &i.revenue_series);
    let pat_a = analyze_series("PAT", &i.pat_series);
    let eps_a = analyze_series("EPS", &i.eps_series);

    let mut positive_factors = Vec::new();
    let mut negative_factors = Vec::new();

    // Long-term / fundamental factors.
    let rev_cagr_pct = f64v(rev_a.cagr_pct);
    let pat_cagr_pct = f64v(pat_a.cagr_pct);
    let eps_cagr_pct = f64v(eps_a.cagr_pct);
    let rev_yoy = f64v(rev_a.yoy_growth_pct);
    let pat_yoy = f64v(pat_a.yoy_growth_pct);
    let eps_yoy = f64v(eps_a.yoy_growth_pct);

    let span = |a: &crate::modules::ipo_intel::models::MetricAnalysis| {
        format!(
            "{}–{}",
            a.cagr_start_period.as_deref().unwrap_or("earliest period"),
            a.latest_period.as_deref().unwrap_or("latest period")
        )
    };

    match rev_cagr_pct {
        Some(p) if p >= 15.0 => positive_factors.push((
            "Strong revenue growth".into(),
            Some(format!("Revenue CAGR of {p:.0}% over {}", span(&rev_a))),
        )),
        Some(p) if p < 0.0 => negative_factors.push((
            "Declining revenue".into(),
            Some(format!("Revenue CAGR of {p:.1}% over {}", span(&rev_a))),
        )),
        _ => {
            if let Some(y) = rev_yoy {
                if y >= 15.0 {
                    positive_factors.push((
                        "Strong revenue growth".into(),
                        Some(format!(
                            "Revenue grew {y:.1}% YoY in {}",
                            rev_a.latest_period.as_deref().unwrap_or("latest period")
                        )),
                    ));
                } else if y < 0.0 {
                    negative_factors.push((
                        "Declining revenue".into(),
                        Some(format!(
                            "Revenue declined {y:.1}% YoY in {}",
                            rev_a.latest_period.as_deref().unwrap_or("latest period")
                        )),
                    ));
                }
            }
        }
    }
    if let Some(p) = pat_cagr_pct {
        if p >= 15.0 {
            positive_factors.push((
                "Strong profit growth".into(),
                Some(format!("PAT CAGR of {p:.0}% over {}", span(&pat_a))),
            ));
        }
    }
    if let Some(p) = eps_cagr_pct {
        if p >= 15.0 {
            positive_factors.push((
                "Strong EPS growth".into(),
                Some(format!("EPS CAGR of {p:.0}% over {}", span(&eps_a))),
            ));
        }
    }
    if let Some(m) = f64v(i.pat_margin_pct) {
        if m >= 10.0 {
            positive_factors.push((
                "Healthy profitability".into(),
                Some(format!("PAT margin of {m:.1}%")),
            ));
        }
    }
    if let Some(m) = f64v(i.ebitda_margin_pct) {
        if m >= 15.0 {
            positive_factors.push((
                "Strong operating margins".into(),
                Some(format!("EBITDA margin of {m:.1}%")),
            ));
        }
    }
    if let Some(roe) = f64v(i.roe_pct) {
        if roe >= 15.0 {
            positive_factors.push((
                "Efficient capital use".into(),
                Some(format!("ROE of {roe:.1}%")),
            ));
        }
    }
    if let Some(roce) = f64v(i.roce_pct) {
        if roce >= 15.0 {
            positive_factors.push((
                "Strong capital returns".into(),
                Some(format!("ROCE of {roce:.1}%")),
            ));
        }
    }
    if let Some(de) = f64v(i.debt_to_equity) {
        if (0.0..=0.5).contains(&de) {
            positive_factors.push((
                "Low leverage".into(),
                Some(format!("Debt-to-equity of {de:.2}")),
            ));
        }
    }
    if i.period_labels.len() >= 3 {
        positive_factors.push((
            "Consistent financial track record".into(),
            Some(format!(
                "{} financial periods available ({})",
                i.period_labels.len(),
                financial_periods_label(&i.period_labels)
            )),
        ));
    }
    if !i.has_risks && !i.board.eq_ignore_ascii_case("sme") {
        if let Some(de) = f64v(i.debt_to_equity) {
            if de <= 1.0 {
                positive_factors.push((
                    "Clean risk profile".into(),
                    Some(format!("No material risk factors disclosed; D/E of {de:.2}")),
                ));
            }
        }
    }

    // Demand / valuation factors.
    if let Some(s) = f64v(i.subscription_overall) {
        if s >= 10.0 {
            positive_factors.push((
                "Strong overall demand".into(),
                Some(format!("Overall subscription of {s:.1}x")),
            ));
        }
    }
    if let Some(q) = f64v(i.subscription_qib) {
        if q >= 50.0 {
            positive_factors.push((
                "Strong institutional demand".into(),
                Some(format!("QIB subscription of {q:.1}x")),
            ));
        }
    }
    if let Some(rt) = f64v(i.subscription_retail) {
        if rt >= 10.0 {
            positive_factors.push((
                "Strong retail demand".into(),
                Some(format!("Retail subscription of {rt:.1}x")),
            ));
        }
    }
    if let Some(pe) = effective_pe(i.pe_ratio, i.issue_price, i.eps) {
        if pe > 0.0 && pe <= 25.0 {
            positive_factors.push((
                "Reasonable valuation".into(),
                Some(format!("IPO P/E of {pe:.1}")),
            ));
        }
    }

    // ---- Negative factors ----
    if let Some(y) = pat_yoy {
        if y < 0.0 {
            negative_factors.push((
                "Declining profitability".into(),
                Some(format!(
                    "PAT declined {y:.1}% YoY in {}",
                    pat_a.latest_period.as_deref().unwrap_or("latest period")
                )),
            ));
        }
    }
    if let Some(y) = eps_yoy {
        if y < 0.0 {
            negative_factors.push((
                "Declining earnings".into(),
                Some(format!(
                    "EPS declined {y:.1}% YoY in {}",
                    eps_a.latest_period.as_deref().unwrap_or("latest period")
                )),
            ));
        }
    }
    if let Some(m) = f64v(i.pat_margin_pct) {
        if m <= 0.0 {
            negative_factors.push((
                "Loss-making".into(),
                Some(format!("PAT margin of {m:.1}% (non-positive earnings)")),
            ));
        }
    }
    if let Some(de) = f64v(i.debt_to_equity) {
        if de < 0.0 {
            negative_factors.push((
                "Negative net worth".into(),
                Some(format!("Debt-to-equity of {de:.2}")),
            ));
        } else if de > 2.0 {
            negative_factors.push((
                "High leverage".into(),
                Some(format!("Debt-to-equity of {de:.2}")),
            ));
        }
    }
    if let Some(pe) = effective_pe(i.pe_ratio, i.issue_price, i.eps) {
        if pe > 35.0 {
            negative_factors.push((
                "Elevated valuation".into(),
                Some(format!(
                    "IPO P/E of {pe:.1} (a peer/sector benchmark is not yet available)"
                )),
            ));
        }
    }
    if let Some(s) = f64v(i.subscription_overall) {
        if s < 1.0 {
            negative_factors.push((
                "Weak demand".into(),
                Some(format!("Overall subscription of {s:.1}x")),
            ));
        }
    }
    if let Some(q) = f64v(i.subscription_qib) {
        if q < 1.0 {
            negative_factors.push((
                "Weak institutional interest".into(),
                Some(format!("QIB subscription of {q:.1}x")),
            ));
        }
    }
    if i.board.eq_ignore_ascii_case("sme") {
        negative_factors.push((
            "SME board liquidity risk".into(),
            Some("Listed on the SME board — lower liquidity and higher volatility".into()),
        ));
    }
    if i.has_risks {
        negative_factors.push((
            "Disclosed risk factors".into(),
            Some(format!(
                "{} risk factor{} disclosed in the prospectus",
                i.risk_factors.len(),
                if i.risk_factors.len() == 1 { "" } else { "s" }
            )),
        ));
    }
    if i.risk_factors.iter().any(|r| {
        let rl = r.to_lowercase();
        ["customer", "concentrat", "clientele", "supplier", "geographic", "dependent", "reliance"]
            .iter()
            .any(|k| rl.contains(k))
    }) {
        negative_factors.push((
            "Customer/supplier concentration risk disclosed".into(),
            Some("Disclosed in the prospectus risk factors".into()),
        ));
    }
    if i.period_labels.len() == 1 {
        negative_factors.push((
            "Limited financial track record".into(),
            Some("Only one financial period is available for evaluation".into()),
        ));
    }

    // ---- Financial period label ----
    let financial_periods = financial_periods_label(&i.period_labels);
    let financial_periods = if financial_periods.is_empty() {
        None
    } else {
        Some(financial_periods)
    };

    AnalysisResult {
        overall_score,
        fundamental_score,
        growth_score,
        profitability_score,
        balance_sheet_score,
        valuation_score,
        subscription_score,
        risk_score,
        long_term_view,
        listing_view,
        confidence,
        data_completeness: round1(data_completeness),
        positive_factors,
        negative_factors,
        missing_data,
        financial_periods,
        financials_retrieved_at: i.financials_retrieved_at,
        subscription_updated_at: i.subscription_updated_at,
        ipo_synced_at: i.ipo_synced_at,
    }
}

fn financial_periods_label(labels: &[String]) -> String {
    if labels.is_empty() {
        return String::new();
    }
    let first = labels.first().cloned().unwrap_or_default();
    let last = labels.last().cloned().unwrap_or_default();
    if first == last {
        first
    } else {
        format!("{first}–{last}")
    }
}

/// Data completeness percentage plus the list of what is missing. Weighted
/// items sum to 100; informational gaps (cash-flow history, peer benchmark,
/// promoter/customer detail) are reported as missing but carry no weight
/// because the engine cannot evaluate them at all in v1.
fn completeness(i: &AnalysisInputs) -> (f64, Vec<String>) {
    let mut pts: f64 = 0.0;
    let mut missing = Vec::new();

    let periods = i.period_labels.len();
    pts += if periods >= 3 {
        20.0
    } else if periods == 2 {
        14.0
    } else if periods == 1 {
        7.0
    } else {
        0.0
    };
    if periods == 0 {
        missing.push("Financial history".into());
    } else if periods == 1 {
        missing.push("Multi-year financial history".into());
    }

    let latest_rev = i.revenue_series.iter().rev().find(|p| p.value.is_some());
    if latest_rev.is_some() {
        pts += 5.0;
    } else {
        missing.push("Revenue".into());
    }
    if i.pat_series.iter().any(|p| p.value.is_some()) {
        pts += 5.0;
    } else {
        missing.push("PAT".into());
    }
    if i.eps_series.iter().any(|p| p.value.is_some()) {
        pts += 8.0;
    } else {
        missing.push("EPS".into());
    }
    let profitability_ok = i.pat_margin_pct.is_some()
        || i.ebitda_margin_pct.is_some()
        || i.roe_pct.is_some()
        || i.roce_pct.is_some();
    if profitability_ok {
        pts += 8.0;
    } else {
        missing.push("Profitability (margins / ROE / ROCE)".into());
    }
    if i.debt_to_equity.is_some() {
        pts += 8.0;
    } else {
        missing.push("Balance sheet (debt-to-equity)".into());
    }
    let cagr_ok = i
        .revenue_series
        .iter()
        .filter(|p| p.value.is_some() && p.period_end.is_some())
        .count()
        >= 2;
    if cagr_ok {
        pts += 8.0;
    } else {
        missing.push("Multi-period growth trend".into());
    }
    if i.subscription_overall.is_some() {
        pts += 12.0;
    } else {
        missing.push("Subscription".into());
    }
    if i.subscription_qib.is_some() || i.subscription_nii.is_some() {
        pts += 4.0;
    } else {
        missing.push("Subscription category breakdown (QIB/NII)".into());
    }
    let valuation_ok = i.pe_ratio.is_some()
        || (i.issue_price.is_some() && f64v(i.eps).map(|e| e > 0.0).unwrap_or(false));
    if valuation_ok {
        pts += 12.0;
    } else {
        missing.push("Valuation (P/E or EPS with issue price)".into());
    }
    let risk_ok = i.debt_to_equity.is_some() || i.has_risks || i.board.eq_ignore_ascii_case("sme");
    if risk_ok {
        pts += 6.0;
    } else {
        missing.push("Risk disclosures".into());
    }
    if i.issue_price.is_some() {
        pts += 4.0;
    } else {
        missing.push("Issue price".into());
    }

    // Informational gaps (zero weight): never available from current sources.
    missing.push("Cash-flow history".into());
    missing.push("Peer/sector valuation benchmark".into());
    missing.push("Promoter holding details".into());
    if !i.risk_factors.iter().any(|r| {
        let rl = r.to_lowercase();
        ["customer", "concentrat", "clientele", "supplier"].iter().any(|k| rl.contains(k))
    }) {
        missing.push("Customer concentration detail".into());
    }

    (pts.min(100.0), missing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn dec(v: f64) -> Decimal {
        Decimal::from_str_exact(&format!("{v}")).unwrap_or_else(|_| Decimal::from(v as i64))
    }

    fn series(vals: &[(i32, f64)]) -> Vec<SeriesPoint> {
        vals.iter()
            .map(|(year, v)| SeriesPoint {
                period: format!("FY{year}"),
                value: Some(dec(*v)),
                period_end: NaiveDate::from_ymd_opt(*year, 3, 31),
            })
            .collect()
    }

    fn base_inputs() -> AnalysisInputs {
        AnalysisInputs {
            board: "mainboard".into(),
            sector: None,
            subscription_overall: Some(dec(40.0)),
            subscription_qib: Some(dec(60.0)),
            subscription_nii: Some(dec(30.0)),
            subscription_retail: Some(dec(20.0)),
            issue_price: Some(dec(300.0)),
            eps: Some(dec(10.0)),
            pe_ratio: Some(dec(30.0)),
            debt_to_equity: Some(dec(0.4)),
            ebitda_margin_pct: Some(dec(22.0)),
            pat_margin_pct: Some(dec(12.0)),
            roe_pct: Some(dec(18.0)),
            roce_pct: Some(dec(20.0)),
            revenue_growth_pct: Some(dec(30.0)),
            pat_growth_pct: Some(dec(25.0)),
            eps_growth_pct: Some(dec(20.0)),
            revenue_series: series(&[(2023, 100.0), (2024, 130.0), (2025, 169.0)]),
            pat_series: series(&[(2023, 10.0), (2024, 14.0), (2025, 18.0)]),
            eps_series: series(&[(2023, 5.0), (2024, 7.0), (2025, 9.0)]),
            period_labels: vec!["FY2023".into(), "FY2024".into(), "FY2025".into()],
            financials_retrieved_at: None,
            subscription_updated_at: None,
            ipo_synced_at: None,
            has_risks: false,
            risk_factors: Vec::new(),
        }
    }

    #[test]
    fn strong_company_is_strong_positive_long_term() {
        let r = compute_analysis(&base_inputs());
        assert_eq!(r.long_term_view, IpoView::StrongPositive);
        assert!(r.overall_score.unwrap() > 70.0);
        assert!(r.data_completeness > 90.0);
        assert_eq!(r.confidence, Confidence::High);
        assert!(r
            .positive_factors
            .iter()
            .any(|(f, _)| f.contains("Strong revenue growth")),
            "factors = {:?}",
            r.positive_factors);
    }

    #[test]
    fn weak_company_is_negative() {
        let mut i = base_inputs();
        i.revenue_series = series(&[(2023, 100.0), (2024, 80.0), (2025, 60.0)]);
        i.pat_series = series(&[(2023, 10.0), (2024, 4.0), (2025, -2.0)]);
        i.eps_series = series(&[(2023, 5.0), (2024, 2.0), (2025, -1.0)]);
        i.revenue_growth_pct = Some(dec(-25.0));
        i.pat_growth_pct = Some(dec(-50.0));
        i.eps_growth_pct = Some(dec(-50.0));
        i.pat_margin_pct = Some(dec(-3.0));
        i.roe_pct = Some(dec(-5.0));
        i.roce_pct = Some(dec(-2.0));
        i.debt_to_equity = Some(dec(3.0));
        let r = compute_analysis(&i);
        assert_eq!(r.long_term_view, IpoView::Negative);
        assert!(r.overall_score.unwrap() < 40.0);
        assert!(r
            .negative_factors
            .iter()
            .any(|(f, _)| f.contains("Loss-making")));
        assert!(r
            .negative_factors
            .iter()
            .any(|(f, _)| f.contains("High leverage")));
    }

    #[test]
    fn high_valuation_tempered() {
        let mut i = base_inputs();
        i.pe_ratio = Some(dec(120.0));
        i.issue_price = Some(dec(1200.0));
        i.eps = Some(dec(10.0));
        let r = compute_analysis(&i);
        assert!(r.valuation_score.unwrap() < 30.0);
        assert!(r
            .negative_factors
            .iter()
            .any(|(f, _)| f.contains("Elevated valuation")));
    }

    #[test]
    fn low_valuation_scores_well() {
        let mut i = base_inputs();
        i.pe_ratio = Some(dec(10.0));
        i.issue_price = Some(dec(100.0));
        let r = compute_analysis(&i);
        assert_eq!(r.valuation_score.unwrap(), 100.0);
        assert!(r
            .positive_factors
            .iter()
            .any(|(f, _)| f.contains("Reasonable valuation")));
    }

    #[test]
    fn negative_earnings_pe_not_meaningful() {
        let mut i = base_inputs();
        i.eps = Some(dec(-4.0));
        i.pe_ratio = Some(dec(-30.0));
        i.pat_margin_pct = Some(dec(-15.0));
        let r = compute_analysis(&i);
        // Valuation component scores 0 for non-positive earnings, but the
        // implied P/E must never be produced from negative EPS.
        assert_eq!(r.valuation_score.unwrap(), 0.0);
        assert!(r
            .negative_factors
            .iter()
            .any(|(f, _)| f.contains("Loss-making")));
        assert!(!r.positive_factors.iter().any(|(f, _)| f.contains("P/E")));
    }

    #[test]
    fn declining_eps_is_a_negative_factor() {
        let mut i = base_inputs();
        i.eps_series = series(&[(2023, 10.0), (2024, 9.0), (2025, 7.0)]);
        i.eps_growth_pct = Some(dec(-22.0));
        let r = compute_analysis(&i);
        assert!(r
            .negative_factors
            .iter()
            .any(|(f, _)| f.contains("Declining earnings")));
    }

    #[test]
    fn strong_eps_growth_is_a_positive_factor() {
        let mut i = base_inputs();
        i.eps_series = series(&[(2023, 5.0), (2024, 8.0), (2025, 13.0)]);
        i.eps_growth_pct = Some(dec(60.0));
        let r = compute_analysis(&i);
        assert!(r
            .positive_factors
            .iter()
            .any(|(f, _)| f.contains("Strong EPS growth")));
    }

    #[test]
    fn high_debt_is_a_concern() {
        let mut i = base_inputs();
        i.debt_to_equity = Some(dec(3.5));
        let r = compute_analysis(&i);
        assert!(r.balance_sheet_score.unwrap() < 50.0);
        assert!(r
            .negative_factors
            .iter()
            .any(|(f, _)| f.contains("High leverage")));
    }

    #[test]
    fn low_debt_is_a_positive() {
        let mut i = base_inputs();
        i.debt_to_equity = Some(dec(0.2));
        let r = compute_analysis(&i);
        assert_eq!(r.balance_sheet_score.unwrap(), 100.0);
        assert!(r
            .positive_factors
            .iter()
            .any(|(f, _)| f.contains("Low leverage")));
    }

    #[test]
    fn strong_subscription_boosts_listing() {
        let mut i = base_inputs();
        i.subscription_overall = Some(dec(120.0));
        i.subscription_qib = Some(dec(200.0));
        let r = compute_analysis(&i);
        assert_eq!(r.subscription_score.unwrap(), 100.0);
        assert_eq!(r.listing_view, IpoView::StrongPositive);
    }

    #[test]
    fn weak_subscription_hurts_listing() {
        let mut i = base_inputs();
        i.subscription_overall = Some(dec(0.4));
        i.subscription_qib = Some(dec(0.3));
        let r = compute_analysis(&i);
        assert_eq!(r.subscription_score.unwrap(), 0.0);
        // Under-subscribed (0.4x) is at best a caution call; a truly elevated
        // valuation on top of weak demand pushes it to NEGATIVE.
        assert!(matches!(
            r.listing_view,
            IpoView::Caution | IpoView::Negative
        ));
    }

    #[test]
    fn weak_subscription_with_high_valuation_is_negative_listing() {
        let mut i = base_inputs();
        i.subscription_overall = Some(dec(0.4));
        i.subscription_qib = Some(dec(0.3));
        i.pe_ratio = Some(dec(80.0));
        let r = compute_analysis(&i);
        assert_eq!(r.listing_view, IpoView::Negative);
    }

    #[test]
    fn missing_financial_data_gives_insufficient_views() {
        let i = AnalysisInputs {
            board: "mainboard".into(),
            subscription_overall: Some(dec(10.0)),
            ..Default::default()
        };
        let r = compute_analysis(&i);
        assert_eq!(r.long_term_view, IpoView::InsufficientData);
        assert!(r.fundamental_score.is_none());
        assert_eq!(r.confidence, Confidence::InsufficientData);
    }

    #[test]
    fn missing_subscription_still_scores_valuation() {
        let mut i = base_inputs();
        i.subscription_overall = None;
        i.subscription_qib = None;
        i.subscription_nii = None;
        i.subscription_retail = None;
        let r = compute_analysis(&i);
        assert!(r.subscription_score.is_none());
        // Valuation + fundamentals still produce a listing view (renormalized).
        assert_ne!(r.listing_view, IpoView::InsufficientData);
        assert!(r.missing_data.iter().any(|m| m.contains("Subscription")));
    }

    #[test]
    fn insufficient_history_is_low_confidence() {
        let mut i = base_inputs();
        i.period_labels = vec!["FY2025".into()];
        i.revenue_series = series(&[(2025, 100.0)]);
        i.pat_series = series(&[(2025, 10.0)]);
        i.eps_series = series(&[(2025, 5.0)]);
        i.revenue_growth_pct = None;
        i.pat_growth_pct = None;
        i.eps_growth_pct = None;
        let r = compute_analysis(&i);
        assert_eq!(r.confidence, Confidence::Low);
        assert!(r.data_completeness < 85.0);
        assert!(r.missing_data.iter().any(|m| m.contains("Multi-year")));
        assert!(r
            .negative_factors
            .iter()
            .any(|(f, _)| f.contains("Limited financial track record")));
    }

    #[test]
    fn conflicting_data_produces_mixed_views() {
        // Strong growth but very expensive valuation and weak demand:
        // long-term should be more positive than listing.
        let mut i = base_inputs();
        i.revenue_growth_pct = Some(dec(50.0));
        i.eps_growth_pct = Some(dec(40.0));
        i.pe_ratio = Some(dec(95.0));
        i.subscription_overall = Some(dec(0.5));
        i.subscription_qib = Some(dec(0.4));
        let r = compute_analysis(&i);
        assert!(r.overall_score.unwrap() > 55.0);
        assert_eq!(r.long_term_view, IpoView::StrongPositive);
        assert_eq!(r.listing_view, IpoView::Negative);
        assert!(r
            .negative_factors
            .iter()
            .any(|(f, _)| f.contains("Weak demand")));
    }

    #[test]
    fn zero_values_are_scored_not_invented() {
        let mut i = base_inputs();
        i.revenue_growth_pct = Some(dec(0.0));
        i.pat_growth_pct = Some(dec(0.0));
        i.eps_growth_pct = Some(dec(0.0));
        i.pat_margin_pct = Some(dec(0.0));
        i.ebitda_margin_pct = Some(dec(0.0));
        i.roe_pct = Some(dec(0.0));
        i.roce_pct = Some(dec(0.0));
        i.debt_to_equity = Some(dec(0.0));
        i.pe_ratio = Some(dec(0.0));
        i.eps = Some(dec(0.0));
        i.subscription_overall = Some(dec(0.0));
        let r = compute_analysis(&i);
        assert!(r.overall_score.is_some());
        assert!(r.growth_score.is_some());
        assert_eq!(r.profitability_score.unwrap(), 0.0);
        assert_eq!(r.subscription_score.unwrap(), 0.0);
    }

    #[test]
    fn deterministic_same_inputs_same_output() {
        let a = compute_analysis(&base_inputs());
        let b = compute_analysis(&base_inputs());
        assert_eq!(a.overall_score, b.overall_score);
        assert_eq!(a.long_term_view, b.long_term_view);
        assert_eq!(a.listing_view, b.listing_view);
        assert_eq!(a.confidence, b.confidence);
        assert_eq!(a.data_completeness, b.data_completeness);
        assert_eq!(a.positive_factors.len(), b.positive_factors.len());
        assert_eq!(a.negative_factors.len(), b.negative_factors.len());
        assert_eq!(a.missing_data, b.missing_data);
    }

    #[test]
    fn views_never_fabricate_sentiment() {
        let r = compute_analysis(&base_inputs());
        // GMP / market sentiment are not evaluated anywhere in v1.
        assert_eq!(r.listing_view, IpoView::StrongPositive);
        // The engine must never inject GMP into the listing view: with a 40x
        // subscription the listing view is driven purely by official demand.
        assert!(!r.positive_factors.iter().any(|(f, _)| f.contains("GMP")));
    }

    #[test]
    fn sme_board_adds_risk_factor() {
        let mut i = base_inputs();
        i.board = "sme".into();
        let r = compute_analysis(&i);
        assert!(r
            .negative_factors
            .iter()
            .any(|(f, _)| f.contains("SME board")));
        assert!(r.risk_score.unwrap() <= 60.0);
    }

    #[test]
    fn customer_concentration_keyword_detected() {
        let mut i = base_inputs();
        i.risk_factors = vec![
            "Concentration of revenue from a small number of customers".into(),
        ];
        i.has_risks = true;
        let r = compute_analysis(&i);
        assert!(r
            .negative_factors
            .iter()
            .any(|(f, _)| f.contains("concentration")));
    }

    #[test]
    fn financial_periods_label_built() {
        let i = base_inputs();
        let r = compute_analysis(&i);
        assert_eq!(r.financial_periods.as_deref(), Some("FY2023–FY2025"));
    }

    #[test]
    fn no_risk_data_yields_no_risk_score() {
        let mut i = base_inputs();
        i.debt_to_equity = None;
        i.has_risks = false;
        i.board = "mainboard".into();
        let r = compute_analysis(&i);
        assert!(r.risk_score.is_none());
        assert!(r.missing_data.iter().any(|m| m.contains("Risk")));
    }
}
