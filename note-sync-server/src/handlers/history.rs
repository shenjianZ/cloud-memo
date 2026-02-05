use axum::{Json, extract::{Query, State}, Extension};
use serde::Deserialize;
use axum::http::StatusCode;
use crate::AppState;
use crate::services::sync_history_service::SyncHistoryService;
use crate::models::SyncHistoryEntry;
use crate::middleware::logging::{RequestId, log_info};
use super::ErrorResponse;

#[derive(Debug, Deserialize)]
pub struct HistoryQueryParams {
    limit: Option<usize>,
}

/// 获取同步历史记录
pub async fn get_history(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<Json<Vec<SyncHistoryEntry>>, ErrorResponse> {
    let limit = params.limit.unwrap_or(50);
    log_info(&request_id, "获取同步历史请求", &format!("user_id={}, limit={}", user_id, limit));

    let service = SyncHistoryService::new(state.pool);

    match service.list(&user_id, limit).await {
        Ok(history) => {
            log_info(&request_id, "获取成功", &format!("记录数量={}", history.len()));
            Ok(Json(history))
        }
        Err(e) => {
            log_info(&request_id, "获取失败", &e.to_string());
            Err(ErrorResponse::new("获取同步历史失败"))
        }
    }
}

/// 清空同步历史
pub async fn clear_history(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
) -> Result<StatusCode, ErrorResponse> {
    log_info(&request_id, "清空同步历史请求", &format!("user_id={}", user_id));

    let service = SyncHistoryService::new(state.pool);

    match service.clear(&user_id).await {
        Ok(_) => {
            log_info(&request_id, "清空成功", "");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            log_info(&request_id, "清空失败", &e.to_string());
            Err(ErrorResponse::new("清空同步历史失败"))
        }
    }
}
