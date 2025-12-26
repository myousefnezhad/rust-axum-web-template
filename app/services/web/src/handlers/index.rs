use app_error::AppError;

pub async fn get_index() -> Result<String, AppError> {
    Ok("ok".to_owned())
}

pub async fn post_index() -> Result<String, AppError> {
    Err(AppError::new(
        "Test",
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        1,
    ))
}
