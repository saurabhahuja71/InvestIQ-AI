//! Portfolio analytics: XIRR (Newton-Raphson), CAGR, and P&L helpers.

use chrono::NaiveDate;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

#[derive(Clone, Debug)]
pub struct CashFlow {
    pub date: NaiveDate,
    /// Negative = money invested; positive = money returned
    pub amount: f64,
}

/// Annualized XIRR; returns None if cannot converge or insufficient data.
pub fn xirr(flows: &[CashFlow], guess: f64) -> Option<f64> {
    if flows.len() < 2 {
        return None;
    }
    let mut rate = guess;
    let d0 = flows[0].date;

    for _ in 0..100 {
        let mut f = 0.0;
        let mut df = 0.0;
        for cf in flows {
            let days = (cf.date - d0).num_days() as f64;
            let t = days / 365.0;
            let denom = (1.0 + rate).powf(t);
            if !denom.is_finite() || denom == 0.0 {
                return None;
            }
            f += cf.amount / denom;
            df += -t * cf.amount / (denom * (1.0 + rate));
        }
        if df.abs() < 1e-12 {
            break;
        }
        let next = rate - f / df;
        if (next - rate).abs() < 1e-7 {
            return Some(next);
        }
        rate = next;
        if !rate.is_finite() || rate <= -0.999999 {
            return None;
        }
    }
    None
}

pub fn cagr(beginning: f64, ending: f64, years: f64) -> Option<f64> {
    if beginning <= 0.0 || ending <= 0.0 || years <= 0.0 {
        return None;
    }
    Some((ending / beginning).powf(1.0 / years) - 1.0)
}

pub fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

/// Years between two dates using day count / 365.25
pub fn years_between(start: NaiveDate, end: NaiveDate) -> f64 {
    let days = (end - start).num_days().max(0) as f64;
    days / 365.25
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn xirr_simple() {
        let flows = vec![
            CashFlow {
                date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                amount: -1000.0,
            },
            CashFlow {
                date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                amount: 1100.0,
            },
        ];
        let r = xirr(&flows, 0.1).unwrap();
        assert!((r - 0.1).abs() < 0.001);
    }

    #[test]
    fn cagr_doubles_in_one_year() {
        let r = cagr(100.0, 200.0, 1.0).unwrap();
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cagr_rejects_bad_inputs() {
        assert!(cagr(0.0, 100.0, 1.0).is_none());
        assert!(cagr(100.0, 100.0, 0.0).is_none());
    }
}
