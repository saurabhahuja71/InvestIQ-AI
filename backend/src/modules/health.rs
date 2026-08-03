use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use redis::AsyncCommands;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "investiq-api" }))
}

pub async fn ready(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(state.db())
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let mut redis = state.redis.clone();
    redis
        .set_ex::<_, _, ()>("healthcheck", "1", 5)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    Ok(Json(json!({ "status": "ready" })))
}
