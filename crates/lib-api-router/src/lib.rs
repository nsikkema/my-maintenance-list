mod health;

use axum::Router;
use axum::routing::get;

#[derive(Clone, Debug)]
struct AppState {}

pub fn api_router() -> Router {
    Router::new()
        .route("/health/live", get(health::live))
        .route("/health/ready", get(health::ready))
        .with_state(AppState {})
}
