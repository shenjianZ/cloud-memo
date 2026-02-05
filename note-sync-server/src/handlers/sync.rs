use axum::{Json, extract::State, Extension, response::IntoResponse};
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::models::{Note, Folder};
use crate::AppState;
use crate::services::sync_history_service::SyncHistoryService;
use crate::middleware::logging::{RequestId, log_info};
use super::ErrorResponse;

#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub notes: Vec<Note>,
    pub folders: Vec<Folder>,
}

#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub notes: Vec<Note>,
    pub folders: Vec<Folder>,
    pub conflicts: Vec<ConflictInfo>,
    pub server_time: i64,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub last_sync_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PullResponse {
    pub notes: Vec<Note>,
    pub folders: Vec<Folder>,
    pub server_time: i64,
}

#[derive(Debug, Serialize)]
pub struct ConflictInfo {
    pub id: String,
    pub entity_type: String,
    pub local_version: i32,
    pub server_version: i32,
    pub title: String,
}

pub async fn push(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResponse>, ErrorResponse> {
    let notes_count = req.notes.len();
    let folders_count = req.folders.len();

    log_info(&request_id, "推送同步请求",
        &format!("user_id={}, notes={}, folders={}", user_id, notes_count, folders_count));

    let start_time = Utc::now().timestamp_millis();
    let history_service = SyncHistoryService::new(state.pool.clone());

    // 开始事务
    let mut tx = state.pool.begin().await.map_err(|e| {
        log_info(&request_id, "开始事务失败", &e.to_string());
        ErrorResponse::new("开始事务失败")
    })?;

    let mut conflicts = Vec::new();

    // 更新 notes
    for note in req.notes {
        // 检查是否已存在
        let existing: Option<Note> = sqlx::query_as::<_, Note>(
            "SELECT * FROM notes WHERE id = ? AND user_id = ?"
        )
        .bind(&note.id)
        .bind(&user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "查询笔记失败", &e.to_string());
            ErrorResponse::new("查询笔记失败")
        })?;

        if let Some(existing_note) = existing {
            // 冲突检测：如果服务器版本比本地版本新，记录冲突
            if existing_note.server_ver > note.server_ver {
                conflicts.push(ConflictInfo {
                    id: note.id.clone(),
                    entity_type: "note".to_string(),
                    local_version: note.server_ver,
                    server_version: existing_note.server_ver,
                    title: note.title.clone(),
                });
                // 服务器版本优先，跳过更新
                continue;
            }
        }

        // 插入或更新笔记
        sqlx::query(
            "INSERT INTO notes (id, user_id, title, content, folder_id,
                              is_deleted, deleted_at, created_at, updated_at, server_ver,
                              is_dirty, last_synced_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
                title = VALUES(title),
                content = VALUES(content),
                folder_id = VALUES(folder_id),
                is_deleted = VALUES(is_deleted),
                deleted_at = VALUES(deleted_at),
                updated_at = VALUES(updated_at),
                server_ver = server_ver + 1,
                is_dirty = VALUES(is_dirty),
                last_synced_at = VALUES(last_synced_at)"
        )
        .bind(&note.id)
        .bind(&user_id)
        .bind(&note.title)
        .bind(&note.content)
        .bind(&note.folder_id)
        .bind(note.is_deleted)
        .bind(note.deleted_at)
        .bind(note.created_at)
        .bind(note.updated_at)
        .bind(note.server_ver)
        .bind(note.is_dirty)
        .bind(note.last_synced_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "更新笔记失败", &e.to_string());
            ErrorResponse::new("更新笔记失败")
        })?;
    }

    // 更新 folders（类似逻辑）
    for folder in req.folders {
        // 检查是否已存在
        let existing: Option<Folder> = sqlx::query_as::<_, Folder>(
            "SELECT * FROM folders WHERE id = ? AND user_id = ?"
        )
        .bind(&folder.id)
        .bind(&user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "查询文件夹失败", &e.to_string());
            ErrorResponse::new("查询文件夹失败")
        })?;

        if let Some(existing_folder) = existing {
            if existing_folder.server_ver > folder.server_ver {
                conflicts.push(ConflictInfo {
                    id: folder.id.clone(),
                    entity_type: "folder".to_string(),
                    local_version: folder.server_ver,
                    server_version: existing_folder.server_ver,
                    title: folder.name.clone(),
                });
                continue;
            }
        }

        sqlx::query(
            "INSERT INTO folders (id, user_id, name, parent_id,
                                is_deleted, deleted_at, created_at, updated_at, server_ver,
                                is_dirty, last_synced_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
                name = VALUES(name),
                parent_id = VALUES(parent_id),
                is_deleted = VALUES(is_deleted),
                deleted_at = VALUES(deleted_at),
                updated_at = VALUES(updated_at),
                server_ver = server_ver + 1,
                is_dirty = VALUES(is_dirty),
                last_synced_at = VALUES(last_synced_at)"
        )
        .bind(&folder.id)
        .bind(&user_id)
        .bind(&folder.name)
        .bind(&folder.parent_id)
        .bind(folder.is_deleted)
        .bind(folder.deleted_at)
        .bind(folder.created_at)
        .bind(folder.updated_at)
        .bind(folder.server_ver)
        .bind(folder.is_dirty)
        .bind(folder.last_synced_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "更新文件夹失败", &e.to_string());
            ErrorResponse::new("更新文件夹失败")
        })?;
    }

    tx.commit().await.map_err(|e| {
        log_info(&request_id, "提交事务失败", &e.to_string());
        ErrorResponse::new("提交事务失败")
    })?;

    // 记录同步历史
    let duration_ms = Utc::now().timestamp_millis() - start_time;
    let pushed_count = (notes_count + folders_count) as i32;
    let conflict_count = conflicts.len() as i32;

    if let Err(e) = history_service
        .create(
            &user_id,
            "push",
            pushed_count,
            0,
            conflict_count,
            None,
            duration_ms,
        )
        .await
    {
        log_info(&request_id, "记录同步历史失败", &e.to_string());
        // 历史记录失败不影响同步操作
    }

    let response = PushResponse {
        notes: vec![],
        folders: vec![],
        conflicts,
        server_time: Utc::now().timestamp(),
    };

    log_info(&request_id, "推送同步完成",
        &format!("pushed={}, conflicts={}, duration_ms={}", pushed_count, conflict_count, duration_ms));

    Ok(Json(response))
}

pub async fn pull(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(req): Json<PullRequest>,
) -> Result<Json<PullResponse>, ErrorResponse> {
    log_info(&request_id, "拉取同步请求",
        &format!("user_id={}, last_sync_at={:?}", user_id, req.last_sync_at));

    let start_time = Utc::now().timestamp_millis();
    let history_service = SyncHistoryService::new(state.pool.clone());

    let notes = sqlx::query_as::<_, Note>(
        "SELECT * FROM notes
         WHERE user_id = ? AND updated_at > ?
         AND is_deleted = false"
    )
    .bind(&user_id)
    .bind(req.last_sync_at.unwrap_or(0))
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        log_info(&request_id, "拉取笔记失败", &e.to_string());
        ErrorResponse::new("拉取笔记失败")
    })?;

    let folders = sqlx::query_as::<_, Folder>(
        "SELECT * FROM folders
         WHERE user_id = ? AND updated_at > ?
         AND is_deleted = false"
    )
    .bind(&user_id)
    .bind(req.last_sync_at.unwrap_or(0))
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        log_info(&request_id, "拉取文件夹失败", &e.to_string());
        ErrorResponse::new("拉取文件夹失败")
    })?;

    // 记录同步历史
    let duration_ms = Utc::now().timestamp_millis() - start_time;
    let pulled_count = (notes.len() + folders.len()) as i32;

    if let Err(e) = history_service
        .create(
            &user_id,
            "pull",
            0,
            pulled_count,
            0,
            None,
            duration_ms,
        )
        .await
    {
        log_info(&request_id, "记录同步历史失败", &e.to_string());
    }

    log_info(&request_id, "拉取同步完成",
        &format!("notes={}, folders={}, duration_ms={}", notes.len(), folders.len(), duration_ms));

    Ok(Json(PullResponse {
        notes,
        folders,
        server_time: Utc::now().timestamp(),
    }))
}
