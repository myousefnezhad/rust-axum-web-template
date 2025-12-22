use crate::handlers::{index::*, login::*};
use app_middleware::web_auth_middleware;
use app_state::AppState;
use axum::{
    Router, middleware,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub fn routers(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_index).post(post_index))
        .route("/login", post(post_login))
        .nest(
            "/auth",
            Router::new()
                .route("/user", get(get_index))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    web_auth_middleware,
                )),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
