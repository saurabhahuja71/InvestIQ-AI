//! Pure IPO alert evaluation (unit-testable).

use chrono::NaiveDate;

/// User-toggleable IPO alert kinds (stored in notification_prefs JSON).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpoAlertKind {
    /// IPO subscription window is open / opened today.
    Open,
    /// IPO closes today.
    ClosesToday,
    /// Allotment announced (status allotted or allotment date is today).
    AllotmentAnnounced,
    /// Listing is tomorrow.
    ListingTomorrow,
    /// Listing is today.
    ListingToday,
}

impl IpoAlertKind {
    pub fn pref_key(self) -> &'static str {
        match self {
            Self::Open => "ipo_open",
            Self::ClosesToday => "ipo_close",
            Self::AllotmentAnnounced => "allotment",
            Self::ListingTomorrow => "listing_tomorrow",
            Self::ListingToday => "listing_day",
        }
    }

    pub fn notif_type(self) -> &'static str {
        match self {
            Self::Open => "ipo_open",
            Self::ClosesToday => "ipo_close",
            Self::AllotmentAnnounced => "allotment",
            // DB enum has listing_day; tomorrow is distinguished via payload.event
            Self::ListingTomorrow | Self::ListingToday => "listing_day",
        }
    }

    pub fn title(self, company: &str) -> String {
        match self {
            Self::Open => format!("{company} IPO is open"),
            Self::ClosesToday => format!("{company} IPO closes today"),
            Self::AllotmentAnnounced => format!("{company} allotment announced"),
            Self::ListingTomorrow => format!("{company} lists tomorrow"),
            Self::ListingToday => format!("{company} lists today"),
        }
    }

    pub fn body(self) -> &'static str {
        match self {
            Self::Open => "Subscription window is open. Review the prospectus and risks before applying.",
            Self::ClosesToday => "Last day to apply (per exchange dates). Confirm with your broker.",
            Self::AllotmentAnnounced => {
                "Allotment status may be available with the registrar. This is not an official allotment result."
            }
            Self::ListingTomorrow => "Listing expected tomorrow. Dates are from the exchange feed.",
            Self::ListingToday => "Listing day. Past unofficial GMP is not predictive of listing performance.",
        }
    }
}

/// Default IPO alert preferences when a user has none stored.
pub fn default_ipo_alert_prefs() -> serde_json::Value {
    serde_json::json!({
        "ipo_open": true,
        "ipo_close": true,
        "allotment": true,
        "listing_tomorrow": true,
        "listing_day": true
    })
}

/// Merge client prefs with defaults; only known IPO keys are normalized to bool.
pub fn merge_ipo_alert_prefs(incoming: &serde_json::Value) -> serde_json::Value {
    let mut out = default_ipo_alert_prefs();
    if let Some(obj) = out.as_object_mut() {
        for key in [
            "ipo_open",
            "ipo_close",
            "allotment",
            "listing_tomorrow",
            "listing_day",
        ] {
            if let Some(v) = incoming.get(key) {
                if let Some(b) = v.as_bool() {
                    obj.insert(key.to_string(), serde_json::Value::Bool(b));
                }
            }
        }
    }
    out
}

pub fn pref_enabled(prefs: &serde_json::Value, kind: IpoAlertKind) -> bool {
    prefs
        .get(kind.pref_key())
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

/// Evaluate which alerts should fire for one IPO on `today` (exchange calendar dates).
pub fn evaluate_ipo_alerts(
    today: NaiveDate,
    status: &str,
    open_date: Option<NaiveDate>,
    close_date: Option<NaiveDate>,
    allotment_date: Option<NaiveDate>,
    listing_date: Option<NaiveDate>,
) -> Vec<IpoAlertKind> {
    let mut out = Vec::new();
    let status = status.to_ascii_lowercase();

    // Opens: open_date is today, or currently open (dedup handled at insert).
    if open_date == Some(today) || status == "open" {
        out.push(IpoAlertKind::Open);
    }

    if close_date == Some(today) {
        out.push(IpoAlertKind::ClosesToday);
    }

    if status == "allotted" || allotment_date == Some(today) {
        out.push(IpoAlertKind::AllotmentAnnounced);
    }

    if let Some(ld) = listing_date {
        if ld == today {
            out.push(IpoAlertKind::ListingToday);
        } else if ld == today.succ_opt().unwrap_or(today) {
            out.push(IpoAlertKind::ListingTomorrow);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn opens_on_open_date_or_status() {
        let today = d(2026, 8, 6);
        let k = evaluate_ipo_alerts(today, "upcoming", Some(today), None, None, None);
        assert!(k.contains(&IpoAlertKind::Open));
        let k2 = evaluate_ipo_alerts(today, "open", Some(d(2026, 8, 1)), None, None, None);
        assert!(k2.contains(&IpoAlertKind::Open));
    }

    #[test]
    fn closes_today_only() {
        let today = d(2026, 8, 6);
        let k = evaluate_ipo_alerts(today, "open", None, Some(today), None, None);
        assert!(k.contains(&IpoAlertKind::ClosesToday));
        let k2 = evaluate_ipo_alerts(today, "open", None, Some(d(2026, 8, 7)), None, None);
        assert!(!k2.contains(&IpoAlertKind::ClosesToday));
    }

    #[test]
    fn allotment_and_listing() {
        let today = d(2026, 8, 6);
        let tomorrow = d(2026, 8, 7);
        let k = evaluate_ipo_alerts(
            today,
            "allotted",
            None,
            None,
            Some(today),
            Some(tomorrow),
        );
        assert!(k.contains(&IpoAlertKind::AllotmentAnnounced));
        assert!(k.contains(&IpoAlertKind::ListingTomorrow));
        assert!(!k.contains(&IpoAlertKind::ListingToday));

        let k2 = evaluate_ipo_alerts(today, "listed", None, None, None, Some(today));
        assert!(k2.contains(&IpoAlertKind::ListingToday));
    }

    #[test]
    fn merge_prefs_only_known_keys() {
        let m = merge_ipo_alert_prefs(&serde_json::json!({
            "ipo_open": false,
            "noise": true,
            "listing_day": false
        }));
        assert_eq!(m["ipo_open"], false);
        assert_eq!(m["ipo_close"], true);
        assert_eq!(m["listing_day"], false);
        assert!(m.get("noise").is_none());
    }
}
