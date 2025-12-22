use app_cryptography::hash::hash;
use app_dto::auth::user::PostUserInput;
use app_error::AppError;
use app_schema::auth::users::User;
use app_state::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use std::sync::Arc;
use tracing::*;

pub async fn get_user(State(state): State<Arc<AppState>>) -> Result<Json<Vec<User>>, AppError> {
    let pg = state.pg.clone();
    let res = sqlx::query_as::<_, User>(&User::select_query())
        .fetch_all(&pg)
        .await?;
    debug!("{:?}", &res);
    Ok(Json(res))
}

pub async fn post_user(
    State(state): State<Arc<AppState>>,
    Json(args): Json<PostUserInput>,
) -> Result<StatusCode, AppError> {
    let pg = state.pg.clone();
    let hash_password = hash(&args.password)?.to_string();
    sqlx::query(&User::insert_query())
        .bind(&args.name)
        .bind(&args.email)
        .bind(&hash_password)
        .execute(&pg)
        .await?;
    debug!("{}", format!("INSERT user: {:#?}", &args));
    Ok(StatusCode::CREATED)
}
