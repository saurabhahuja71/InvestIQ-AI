use axum::middleware as axum_mw;
use axum::routing::get;
use axum::Router;

use crate::middleware::rate_limit_middleware;
use crate::modules;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .nest("/auth", modules::auth::router())
        .nest("/ipos", modules::ipo::router())
        .nest("/portfolios", modules::portfolio::router())
        .nest("/journal", modules::journal::router())
        .nest("/ai", modules::ai::router())
        .nest("/notifications", modules::notifications::router())
        .layer(axum_mw::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ));

    Router::new()
        .route("/health", get(modules::health::health))
        .route("/ready", get(modules::health::ready))
        .nest("/api/v1", api)
        .with_state(state)
}
