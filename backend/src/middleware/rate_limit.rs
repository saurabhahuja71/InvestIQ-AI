//! Simple Redis fixed-window rate limiter (per client IP).

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use redis::AsyncCommands;

use crate::error::AppError;
use crate::state::AppState;

pub async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let limit = state.config().rate_limit_rps.max(1);
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unknown".into());

    let key = format!("rl:ip:{ip}");
    let mut redis = state.redis.clone();

    match redis.incr::<_, _, u64>(&key, 1u64).await {
        Ok(count) => {
            if count == 1 {
                let _: Result<(), _> = redis.expire(&key, 1).await;
            }
            if count > u64::from(limit) {
                return Err(AppError::RateLimited);
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "rate limit redis error; allowing request");
        }
    }

    Ok(next.run(request).await)
}
