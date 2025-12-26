use crate::handlers::{index::*, login::*, ping::*, user::*};
use app_middleware::web_auth_middleware;
use app_state::AppState;
use axum::{
    Router, middleware,
    routing::{get, patch, post},
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_index).post(post_index))
        .route("/login", post(post_login))
        .route("/ping", get(ping).post(ping))
        .nest(
            "/auth",
            Router::new()
                .route("/ping", get(ping).post(ping))
                .route("/user", get(get_user).post(post_user))
                .route("/change_password", patch(patch_change_password))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    web_auth_middleware,
                )),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
