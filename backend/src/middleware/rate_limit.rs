//! Simple Redis fixed-window rate limiter (per client IP).

use axum::extract::ConnectInfo;
use axum::extract::Request;
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use redis::AsyncCommands;
use std::net::SocketAddr;

use crate::error::AppError;
use crate::state::AppState;

pub async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    connect: Option<ConnectInfo<SocketAddr>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let limit = state.config().rate_limit_rps.max(1);
    let ip = client_ip(&headers, connect.as_ref());
    let key = format!("rl:ip:{ip}");

    let mut redis = state.redis.clone();
    match redis.incr::<_, _, u64>(&key, 1u64).await {
        Ok(count) => {
            if count == 1 {
                let _: Result<(), _> = redis.expire(&key, 1).await;
            }
            // `rate_limit_rps` = max requests per 1-second window
            if count > u64::from(limit) {
                return Err(AppError::RateLimited);
            }
        }
        Err(e) => {
            // Fail open so a Redis blip does not take down the API
            tracing::warn!(error = %e, "rate limit redis error; allowing request");
        }
    }

    Ok(next.run(request).await)
}

fn client_ip(headers: &HeaderMap, connect: Option<&ConnectInfo<SocketAddr>>) -> String {
    if let Some(fwd) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return fwd.to_string();
    }
    connect
        .map(|c| c.0.ip().to_string())
        .unwrap_or_else(|| "unknown".into())
}
