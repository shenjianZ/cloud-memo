use super::ErrorResponse;
use crate::middleware::logging::{log_info, RequestId};
use crate::models::Note;
use crate::AppState;
use axum::{Extension, Json};
use axum::extract::State;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateSnapshotRequest {
    pub title: String,
    pub content: String,
    pub snapshot_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListNotesRequest {
    pub workspace_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// 列出笔记
pub async fn list_notes(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(req): Json<ListNotesRequest>,
) -> Result<Json<Vec<Note>>, ErrorResponse> {
    log_info(&request_id, "列出笔记请求", &format!("user_id={}, workspace_id={:?}", user_id, req.workspace_id));

    let limit = req.limit.unwrap_or(100).min(1000) as i64;
    let offset = req.offset.unwrap_or(0) as i64;

    let notes = if let Some(workspace_id) = req.workspace_id {
        // 指定工作空间
        sqlx::query_as::<_, Note>(
            "SELECT * FROM notes
             WHERE user_id = ? AND workspace_id = ? AND is_deleted = FALSE
             ORDER BY updated_at DESC
             LIMIT ? OFFSET ?"
        )
        .bind(&user_id)
        .bind(&workspace_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    } else {
        // 查询默认空间
        sqlx::query_as::<_, Note>(
            "SELECT n.* FROM notes n
             INNER JOIN workspaces w ON n.workspace_id = w.id
             WHERE n.user_id = ? AND w.user_id = ? AND w.is_default = TRUE
               AND n.is_deleted = FALSE AND w.is_deleted = FALSE
             ORDER BY n.updated_at DESC
             LIMIT ? OFFSET ?"
        )
        .bind(&user_id)
        .bind(&user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    };

    let notes = notes.map_err(|e| {
        log_info(&request_id, "查询笔记失败", &e.to_string());
        ErrorResponse::new("查询笔记失败")
    })?;

    log_info(&request_id, "查询笔记成功", &format!("count={}", notes.len()));
    Ok(Json(notes))
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
) -> Result<Json<Vec<String>>, ErrorResponse> {
    log_info(&request_id, "列出快照请求", &format!("note_id={}", id.0));
    // TODO: 实现列出快照逻辑
    log_info(&request_id, "列出快照", "TODO: 未实现");
    Err(ErrorResponse::new("快照功能暂未实现"))
}
