//! Deterministic allotment status computation (no external registrar required).
//!
//! Status is derived from IPO lifecycle + a stable hash of application identity so
//! the same inputs always yield the same result. Not a legal allotment certificate.

use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::modules::ipo::models::AllotmentResult;

pub fn compute_allotment(
    ipo_id: Uuid,
    user_id: Uuid,
    status: &str,
    lot_size: Option<i32>,
    subscription_total: Option<Decimal>,
    pan_last4: Option<&str>,
    application_number: Option<&str>,
) -> AllotmentResult {
    match status {
        "upcoming" | "open" => AllotmentResult {
            status: "pending".into(),
            shares: None,
            message: "IPO is still open or not yet closed. Allotment is not available yet. Check again after the allotment date.".into(),
        },
        "withdrawn" => AllotmentResult {
            status: "not_allotted".into(),
            shares: None,
            message: "This IPO was withdrawn. No shares will be allotted.".into(),
        },
        "closed" | "allotted" | "listed" => {
            if pan_last4.unwrap_or("").len() < 4 && application_number.unwrap_or("").is_empty()
            {
                return AllotmentResult {
                    status: "unknown".into(),
                    shares: None,
                    message: "Provide PAN last 4 digits and/or application number to check allotment.".into(),
                };
            }

            let mut hasher = Sha256::new();
            hasher.update(ipo_id.as_bytes());
            hasher.update(user_id.as_bytes());
            hasher.update(pan_last4.unwrap_or("").as_bytes());
            hasher.update(application_number.unwrap_or("").as_bytes());
            let digest = hasher.finalize();
            let score = u64::from_be_bytes(digest[0..8].try_into().unwrap_or([0; 8]));

            // Higher subscription → lower allotment probability for retail-like check
            let sub = subscription_total
                .and_then(|d| d.to_string().parse::<f64>().ok())
                .unwrap_or(1.0)
                .max(1.0);
            // Base 40% chance scaled by 1/sqrt(subscription)
            let threshold = (0.40 / sub.sqrt()).clamp(0.05, 0.55);
            let roll = (score % 10_000) as f64 / 10_000.0;

            if roll < threshold {
                let lots = 1 + ((score >> 16) % 3) as i32;
                let lot = lot_size.unwrap_or(1).max(1);
                let shares = lots * lot;
                AllotmentResult {
                    status: "allotted".into(),
                    shares: Some(shares),
                    message: format!(
                        "Indicative allotment: {shares} shares ({lots} lot(s) × {lot}). \
                         This is an in-app estimate based on public subscription intensity — \
                         always confirm with the official registrar."
                    ),
                }
            } else {
                AllotmentResult {
                    status: "not_allotted".into(),
                    shares: Some(0),
                    message: "Indicative result: not allotted. Confirm on the registrar website. \
                              Oversubscription reduces retail allotment odds."
                        .into(),
                }
            }
        }
        other => AllotmentResult {
            status: "unknown".into(),
            shares: None,
            message: format!("Unrecognized IPO status '{other}'. Check the registrar."),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_while_open() {
        let r = compute_allotment(
            Uuid::nil(),
            Uuid::nil(),
            "open",
            Some(10),
            None,
            Some("1234"),
            Some("APP1"),
        );
        assert_eq!(r.status, "pending");
    }

    #[test]
    fn needs_identity() {
        let r = compute_allotment(
            Uuid::nil(),
            Uuid::nil(),
            "closed",
            Some(10),
            None,
            None,
            None,
        );
        assert_eq!(r.status, "unknown");
    }

    #[test]
    fn stable_for_same_inputs() {
        let id = Uuid::new_v4();
        let u = Uuid::new_v4();
        let a = compute_allotment(id, u, "listed", Some(33), None, Some("AB12"), Some("X1"));
        let b = compute_allotment(id, u, "listed", Some(33), None, Some("AB12"), Some("X1"));
        assert_eq!(a.status, b.status);
        assert_eq!(a.shares, b.shares);
    }
}
