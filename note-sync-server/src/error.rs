use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub struct AppError(pub anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("Application error: {:?}", self.0);

        let (status, message) = match self.0.downcast_ref::<&str>() {
            Some(&"not_found") => (StatusCode::NOT_FOUND, "Resource not found"),
            Some(&"unauthorized") => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = json!({
            "status": "error",
            "message": message
        });

        (status, Json(body)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
