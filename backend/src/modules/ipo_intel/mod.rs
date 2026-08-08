//! Milestone 4 — IPO Intelligence (revision: fundamentals-first).
//!
//! Subscription snapshots + history, financials with growth/valuation analysis,
//! and the transparent fundamentals-first InvestIQ IPO Score (see `logic.rs`
//! for the exact, deterministic methodology). No fabricated data: unavailable
//! fields are reported as "Not available". GMP is excluded from all production
//! responses in v1 but the schema/architecture remains extensible for a future
//! market-sentiment provider.

pub mod handlers;
pub mod logic;
pub mod models;
