use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bcrypt::BcryptError;
use deadpool_redis::{CreatePoolError, redis::RedisError};
use jsonwebtoken::errors::Error as JwtError;
use log::warn;
use rmcp::{ErrorData, model::ErrorCode};
use rsa::Error as RsaError;
use serde::Serialize;
use serde_json::Error as JsonError;
use serde_json::json;
use sqlx::Error as SqlxError;
use std::{error::Error as StdError, fmt, io::Error as IoError};

pub static SYSTEM_ERROR_CODE: i64 = -1000;
pub static SYSTEM_ERROR_CODE_DB: i64 = -1001;
pub static SYSTEM_ERROR_CODE_IO: i64 = -1002;
pub static SYSTEM_ERROR_CODE_CRYPTO: i64 = -1003;
pub static SYSTEM_ERROR_CODE_JSON: i64 = -1004;

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub message: String,
    #[serde(serialize_with = "serialize_status")]
    pub status: StatusCode,
    pub code: i64,
}

fn serialize_status<S>(status: &StatusCode, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_u16(status.as_u16())
}

impl AppError {
    pub fn new(message: impl Into<String>, status: StatusCode, code: i64) -> Self {
        Self {
            message: message.into(),
            status,
            code,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            message,
            StatusCode::INTERNAL_SERVER_ERROR,
            SYSTEM_ERROR_CODE,
        )
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"{{"message":"{}","status":{},"code":{}}}"#,
            self.message,
            self.status.as_u16(),
            self.code
        )
    }
}

impl StdError for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        warn!(
            "AppError ({}): {} (HTTP {})",
            self.code,
            self.message,
            self.status.as_u16()
        );
        (self.status, Json(self)).into_response()
    }
}

// --------------------
// Error conversions
// --------------------

impl From<SqlxError> for AppError {
    fn from(value: SqlxError) -> Self {
        Self::new(
            format!("{value:?}"),
            StatusCode::BAD_REQUEST,
            SYSTEM_ERROR_CODE_DB,
        )
    }
}

impl From<RedisError> for AppError {
    fn from(value: RedisError) -> Self {
        Self::new(
            format!("{value:?}"),
            StatusCode::BAD_REQUEST,
            SYSTEM_ERROR_CODE_DB,
        )
    }
}

impl From<CreatePoolError> for AppError {
    fn from(value: CreatePoolError) -> Self {
        Self::new(
            format!("{value:?}"),
            StatusCode::BAD_REQUEST,
            SYSTEM_ERROR_CODE_DB,
        )
    }
}

impl From<IoError> for AppError {
    fn from(value: IoError) -> Self {
        Self::new(
            format!("{value:?}"),
            StatusCode::BAD_REQUEST,
            SYSTEM_ERROR_CODE_IO,
        )
    }
}

impl From<JsonError> for AppError {
    fn from(value: JsonError) -> Self {
        Self::new(
            format!("{value:?}"),
            StatusCode::INTERNAL_SERVER_ERROR,
            SYSTEM_ERROR_CODE_JSON,
        )
    }
}

impl From<JwtError> for AppError {
    fn from(value: JwtError) -> Self {
        Self::new(
            format!("{value:?}"),
            StatusCode::BAD_REQUEST,
            SYSTEM_ERROR_CODE_CRYPTO,
        )
    }
}

impl From<RsaError> for AppError {
    fn from(value: RsaError) -> Self {
        Self::new(
            format!("{value:?}"),
            StatusCode::BAD_REQUEST,
            SYSTEM_ERROR_CODE_CRYPTO,
        )
    }
}

impl From<BcryptError> for AppError {
    fn from(value: BcryptError) -> Self {
        Self::new(
            format!("{value:?}"),
            StatusCode::BAD_REQUEST,
            SYSTEM_ERROR_CODE_CRYPTO,
        )
    }
}

// Convert to MCP ErrorData
impl From<AppError> for ErrorData {
    fn from(e: AppError) -> Self {
        let code_i32 = i32::try_from(e.code).unwrap_or(i32::MAX);
        ErrorData {
            code: ErrorCode(code_i32),
            message: e.message.clone().into(),
            data: Some(json!({
                "http_status": e.status.as_u16(),
                "app_code": e.code,
                "message": e.message,
            })),
        }
    }
}
