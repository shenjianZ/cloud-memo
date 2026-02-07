use super::ErrorResponse;
use crate::middleware::logging::{log_info, RequestId};
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Debug, Deserialize)]
pub struct CreateSnapshotRequest {
    pub title: String,
    pub content: String,
    pub snapshot_name: Option<String>,
}

pub async fn create_snapshot(
    Extension(request_id): Extension<RequestId>,
    id: axum::extract::Path<String>,
    _req: Json<CreateSnapshotRequest>,
) -> Result<Json<()>, ErrorResponse> {
    log_info(&request_id, "创建快照请求", &format!("note_id={}", id.0));
    // TODO: 实现创建快照逻辑
    log_info(&request_id, "创建快照", "TODO: 未实现");
    Err(ErrorResponse::new("快照功能暂未实现"))
}

pub async fn list_snapshots(
    Extension(request_id): Extension<RequestId>,
    id: axum::extract::Path<String>,
    Extension(_pool): Extension<MySqlPool>,
) -> Result<Json<Vec<String>>, ErrorResponse> {
    log_info(&request_id, "列出快照请求", &format!("note_id={}", id.0));
    // TODO: 实现列出快照逻辑
    log_info(&request_id, "列出快照", "TODO: 未实现");
    Err(ErrorResponse::new("快照功能暂未实现"))
}
