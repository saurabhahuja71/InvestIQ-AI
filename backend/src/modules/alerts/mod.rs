//! IPO alert preference evaluation and HTTP surface (Milestone 3).
//!
//! - GET  /alerts
//! - PUT  /alerts/preferences

pub mod handlers;
pub mod logic;

pub use handlers::router;
