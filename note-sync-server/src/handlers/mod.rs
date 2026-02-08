use axum::{http::StatusCode, Json, response::IntoResponse};
use serde::Serialize;

pub mod auth;
pub mod sync;
pub mod notes;
pub mod folders;
pub mod devices;
pub mod history;
pub mod profile;
pub mod workspaces;

/// 统一的错误响应结构
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,  // HTTP 状态码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,  // 应用错误代码
    pub error: String,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            status: None,
            error_code: None,
            error: message.into(),
        }
    }

    pub fn new_with_code(message: impl Into<String>, status: u16, error_code: impl Into<String>) -> Self {
        Self {
            status: Some(status),
            error_code: Some(error_code.into()),
            error: message.into(),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let status_code = self.status
            .and_then(|s| StatusCode::from_u16(s).ok())
            .unwrap_or(StatusCode::BAD_REQUEST);
        (status_code, Json(self)).into_response()
    }
}
