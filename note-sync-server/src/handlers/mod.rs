use axum::{http::StatusCode, Json, response::IntoResponse};
use serde::Serialize;

pub mod auth;
pub mod sync;
pub mod notes;
pub mod folders;
pub mod devices;
pub mod history;
pub mod profile;

/// 统一的错误响应结构
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}
