use axum::routing::get;
use axum::Router;

use crate::modules;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(modules::health::health))
        .route("/ready", get(modules::health::ready))
        .nest(
            "/api/v1",
            Router::new()
                .nest("/auth", modules::auth::router())
                .nest("/ipos", modules::ipo::router())
                .nest("/portfolios", modules::portfolio::router())
                .nest("/journal", modules::journal::router())
                .nest("/ai", modules::ai::router()),
        )
        .with_state(state)
}
