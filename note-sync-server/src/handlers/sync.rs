use axum::{extract::State, Extension, Json, http::HeaderMap};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::ErrorResponse;
use crate::middleware::logging::{log_info, RequestId};
use crate::models::{Folder, Note, Tag, NoteVersion, NoteTagRelation, Workspace, ConflictResolutionStrategy};
use crate::services::sync_history_service::SyncHistoryService;
use crate::services::sync_lock_service::SyncLockService;
use crate::AppState;

/// 统一同步请求
#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<Vec<Workspace>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<Note>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub folders: Option<Vec<Folder>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshots: Option<Vec<NoteVersion>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_tags: Option<Vec<NoteTagRelation>>,

    /// 冲突解决策略（默认：创建冲突副本）
    #[serde(default)]
    pub conflict_resolution: ConflictResolutionStrategy,

    /// 设备ID（用于操作锁和设备追踪）
    #[serde(default)]
    pub device_id: Option<String>,
}

/// 统一同步响应
#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub status: String,  // "success" | "partial_success" | "error"
    pub server_time: i64,
    pub last_sync_at: i64,

    // 云端新增/更新的数据
    pub upserted_workspaces: Vec<Workspace>,
    pub upserted_notes: Vec<Note>,
    pub upserted_folders: Vec<Folder>,
    pub upserted_tags: Vec<Tag>,
    pub upserted_snapshots: Vec<NoteVersion>,
    pub upserted_note_tags: Vec<NoteTagRelation>,

    // 云端删除的数据（只返回 ID）
    pub deleted_note_ids: Vec<String>,
    pub deleted_folder_ids: Vec<String>,
    pub deleted_tag_ids: Vec<String>,
    #[serde(default)]
    pub deleted_workspace_ids: Vec<String>,

    // 推送统计（服务器确认实际更新的数量）
    pub pushed_workspaces: usize,
    pub pushed_notes: usize,
    pub pushed_folders: usize,
    pub pushed_tags: usize,
    pub pushed_snapshots: usize,
    pub pushed_note_tags: usize,
    pub pushed_total: usize,  // 推送总数

    // 拉取统计（服务器端真正的新数据，不包括客户端刚推送的数据）
    pub pulled_workspaces: usize,
    pub pulled_notes: usize,
    pub pulled_folders: usize,
    pub pulled_tags: usize,
    pub pulled_snapshots: usize,
    pub pulled_note_tags: usize,
    pub pulled_total: usize,  // 拉取总数

    // 冲突列表
    #[serde(default)]
    pub conflicts: Vec<ConflictInfo>,
}

#[derive(Debug, Serialize)]
pub struct ConflictInfo {
    pub id: String,
    pub entity_type: String,
    pub local_version: i32,
    pub server_version: i32,
    pub title: String,
}

/// 验证工作空间是否属于当前用户
///
/// 在同步前验证，防止恶意客户端访问其他用户的工作空间
async fn verify_workspace_ownership(
    pool: &MySqlPool,
    user_id: &str,
    workspace_id: &str,
) -> Result<bool, String> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workspaces WHERE id = ? AND user_id = ? AND is_deleted = FALSE"
    )
    .bind(workspace_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("验证工作空间归属失败: {}", e))?;

    Ok(count > 0)
}

/// 统一同步接口：合并 push 和 pull
pub async fn sync(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    headers: HeaderMap,
    Json(req): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, ErrorResponse> {
    // 从请求头中获取 User-Agent
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 获取 workspace_id，如果未指定则使用用户的默认空间
    let workspace_id = if let Some(ref ws_id) = req.workspace_id {
        // 验证工作空间是否属于当前用户（安全检查）
        match verify_workspace_ownership(&state.pool, &user_id, ws_id).await {
            Ok(true) => {
                log_info(&request_id, "工作空间验证成功", &format!("workspace_id={}", ws_id));
                Some(ws_id.clone())
            },
            Ok(false) => {
                log_info(&request_id, "工作空间验证失败", &format!("workspace_id={} 不属于用户 user_id={}", ws_id, user_id));
                return Err(ErrorResponse::new_with_code(
                    format!("工作空间 {} 不属于当前用户", ws_id),
                    403,
                    "WORKSPACE_NOT_OWNED",
                ));
            },
            Err(e) => {
                log_info(&request_id, "工作空间验证错误", &e);
                return Err(ErrorResponse::new_with_code(
                    "验证工作空间归属失败".to_string(),
                    500,
                    "WORKSPACE_VERIFICATION_ERROR",
                ));
            }
        }
    } else {
        // 查询用户的默认空间
        let default_ws_id: Option<String> = sqlx::query_scalar(
            "SELECT id FROM workspaces WHERE user_id = ? AND is_default = TRUE AND is_deleted = FALSE LIMIT 1"
        )
        .bind(&user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            log_info(&request_id, "查询默认工作空间失败", &e.to_string());
            ErrorResponse::new("查询默认工作空间失败")
        })?;
        default_ws_id
    };

    log_info(&request_id, "工作空间ID", &format!("workspace_id={:?}", workspace_id));

    // 处理可选字段，None 转为空数组
    let workspaces = req.workspaces.unwrap_or_default();
    let notes = req.notes.unwrap_or_default();
    let folders = req.folders.unwrap_or_default();
    let tags = req.tags.unwrap_or_default();
    let snapshots = req.snapshots.unwrap_or_default();
    let note_tags = req.note_tags.unwrap_or_default();

    let workspaces_count = workspaces.len();
    let notes_count = notes.len();
    let folders_count = folders.len();
    let tags_count = tags.len();
    let snapshots_count = snapshots.len();
    let note_tags_count = note_tags.len();

    // 收集客户端推送的 workspace ID（用于后续计算真实的 pulled 统计）
    let pushed_workspace_ids: std::collections::HashSet<String> = workspaces.iter().map(|w| w.id.clone()).collect();

    // 推送统计计数器（服务器确认实际更新的数量）
    let mut pushed_workspaces = 0usize;
    let mut pushed_notes = 0usize;
    let mut pushed_folders = 0usize;
    let mut pushed_tags = 0usize;
    let mut pushed_snapshots = 0usize;
    let mut pushed_note_tags = 0usize;

    // 打印同步请求参数
    log_info(
        &request_id,
        "同步请求参数",
        &format!(
            "user_id={}, device_id={:?}, conflict_resolution={:?}, last_sync_at={:?}, workspaces={}, notes={}, folders={}, tags={}, snapshots={}, note_tags={}",
            user_id,
            req.device_id,
            req.conflict_resolution,
            req.last_sync_at,
            workspaces_count,
            notes_count,
            folders_count,
            tags_count,
            snapshots_count,
            note_tags_count
        ),
    );

    // 打印工作空间详情（前 3 个）
    if !workspaces.is_empty() {
        let preview_workspaces: Vec<_> = workspaces.iter()
            .take(3)
            .map(|w| format!("id={}, name={}, ver={}", w.id, w.name, w.server_ver))
            .collect();
        log_info(
            &request_id,
            "同步工作空间预览",
            &format!("total={}, preview=[{}]", workspaces_count, preview_workspaces.join(", ")),
        );
    }

    // 打印笔记详情（前 3 个）
    if !notes.is_empty() {
        let preview_notes: Vec<_> = notes.iter()
            .take(3)
            .map(|n| format!("id={}, title={}, ver={}", n.id, n.title, n.server_ver))
            .collect();
        log_info(
            &request_id,
            "同步笔记预览",
            &format!("total={}, preview=[{}]", notes_count, preview_notes.join(", ")),
        );
    }

    // 打印文件夹详情（前 3 个）
    if !folders.is_empty() {
        let preview_folders: Vec<_> = folders.iter()
            .take(3)
            .map(|f| format!("id={}, name={}, ver={}", f.id, f.name, f.server_ver))
            .collect();
        log_info(
            &request_id,
            "同步文件夹预览",
            &format!("total={}, preview=[{}]", folders_count, preview_folders.join(", ")),
        );
    }

    // 打印标签详情（前 3 个）
    if !tags.is_empty() {
        let preview_tags: Vec<_> = tags.iter()
            .take(3)
            .map(|t| format!("id={}, name={}, ver={}", t.id, t.name, t.server_ver))
            .collect();
        log_info(
            &request_id,
            "同步标签预览",
            &format!("total={}, preview=[{}]", tags_count, preview_tags.join(", ")),
        );
    }

    // 打印快照详情（前 3 个）
    if !snapshots.is_empty() {
        let preview_snapshots: Vec<_> = snapshots.iter()
            .take(3)
            .map(|s| format!("id={}, note_id={}, ver={}", s.id, s.note_id, s.server_ver))
            .collect();
        log_info(
            &request_id,
            "同步快照预览",
            &format!("total={}, preview=[{}]", snapshots_count, preview_snapshots.join(", ")),
        );
    }

    // 打印笔记标签关联详情（前 3 个）
    if !note_tags.is_empty() {
        let preview_note_tags: Vec<_> = note_tags.iter()
            .take(3)
            .map(|nt| format!("note_id={}, tag_id={}", nt.note_id, nt.tag_id))
            .collect();
        log_info(
            &request_id,
            "同步笔记标签关联预览",
            &format!("total={}, preview=[{}]", note_tags_count, preview_note_tags.join(", ")),
        );
    }

    let start_time = Utc::now().timestamp();
    let history_service = SyncHistoryService::new(state.pool.clone());
    let lock_service = SyncLockService::new(state.pool.clone());

    // 获取操作锁（使用 RAII Guard 自动释放，包含工作空间）
    // 锁持续时间：30秒（作为兜底，实际会在 sync 完成后立即释放）
    let device_id = req.device_id.as_deref().unwrap_or("unknown");
    let _lock_guard = lock_service.acquire_guard(
        &user_id,
        device_id,
        workspace_id.as_deref(),  // 传入 workspace_id
        30,
    ).await
        .map_err(|e| {
            log_info(&request_id, "获取同步锁失败", &e.to_string());
            ErrorResponse::new_with_code(
                format!("该用户的其他工作空间正在同步，请稍后重试"),
                409,  // Conflict
                "SYNC_IN_PROGRESS",
            )
        })?;

    log_info(&request_id, "获取同步锁成功", &format!("device_id={}, workspace_id={:?}", device_id, workspace_id));

    // 开始事务
    let mut tx = state.pool.begin().await.map_err(|e| {
        log_info(&request_id, "开始事务失败", &e.to_string());
        ErrorResponse::new("开始事务失败")
    })?;

    let mut conflicts = Vec::new();

    // ===== 1. 保存客户端更改（带版本冲突检测） =====

    // 优先处理 workspaces（其他数据依赖 workspace_id）
    log_info(&request_id, "开始处理工作空间同步", &format!("workspaces_count={}", workspaces_count));

    for workspace in workspaces {
        // 使用 FOR UPDATE 锁定行，防止并发修改
        log_info(&request_id, "查询工作空间", &format!("id={}, local_ver={}", workspace.id, workspace.server_ver));
        let existing: Option<Workspace> =
            sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = ? AND user_id = ? FOR UPDATE")
                .bind(&workspace.id)
                .bind(&user_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| {
                    log_info(&request_id, "查询工作空间失败", &e.to_string());
                    ErrorResponse::new("查询工作空间失败")
                })?;

        if let Some(existing_ws) = existing {
            log_info(&request_id, "工作空间已存在", &format!("id={}, server_ver={}", workspace.id, existing_ws.server_ver));
            // 冲突检测：如果服务器版本比本地版本新，记录冲突并跳过
            if existing_ws.server_ver > workspace.server_ver {
                log_info(&request_id, "检测到冲突", &format!("id={}, local_ver={}, server_ver={}", workspace.id, workspace.server_ver, existing_ws.server_ver));
                conflicts.push(ConflictInfo {
                    id: workspace.id.clone(),
                    entity_type: "workspace".to_string(),
                    local_version: workspace.server_ver,
                    server_version: existing_ws.server_ver,
                    title: workspace.name.clone(),
                });
                continue;
            } else {
                log_info(&request_id, "无冲突，正常更新", &format!("id={}, server_ver={} -> {}", workspace.id, existing_ws.server_ver, existing_ws.server_ver + 1));
            }
        } else {
            log_info(&request_id, "工作空间不存在，新建", &format!("id={}, name={}", workspace.id, workspace.name));
        }

        // 插入或更新工作空间
        let new_server_ver = workspace.server_ver + 1;

        // 构建设备描述
        let updated_by_device = format!(
            "{} ({})",
            req.device_id.as_deref().unwrap_or("unknown"),
            user_agent.as_deref().unwrap_or("Unknown Device")
        );

        sqlx::query(
            "INSERT INTO workspaces
             (id, user_id, name, description, icon, color, is_default, sort_order,
              is_deleted, deleted_at, created_at, updated_at, server_ver, device_id, updated_by_device)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
                name = VALUES(name),
                description = VALUES(description),
                icon = VALUES(icon),
                color = VALUES(color),
                is_default = VALUES(is_default),
                sort_order = VALUES(sort_order),
                is_deleted = VALUES(is_deleted),
                deleted_at = VALUES(deleted_at),
                updated_at = UNIX_TIMESTAMP(),
                server_ver = server_ver + 1,
                device_id = VALUES(device_id),
                updated_by_device = VALUES(updated_by_device)"
        )
        .bind(&workspace.id)
        .bind(&user_id)
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(&workspace.icon)
        .bind(&workspace.color)
        .bind(workspace.is_default)
        .bind(workspace.sort_order)
        .bind(workspace.is_deleted)
        .bind(workspace.deleted_at)
        .bind(workspace.created_at)
        .bind(workspace.updated_at)
        .bind(new_server_ver)
        .bind(&req.device_id)
        .bind(&updated_by_device)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "更新工作空间失败", &e.to_string());
            ErrorResponse::new("更新工作空间失败")
        })?;

        // 推送成功，递增计数器
        pushed_workspaces += 1;

        // 验证：只查询 server_ver 字段
        let verify_server_ver: Option<i32> = sqlx::query_scalar(
            "SELECT server_ver FROM workspaces WHERE id = ? AND user_id = ?"
        )
        .bind(&workspace.id)
        .bind(&user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "验证工作空间失败", &e.to_string());
            ErrorResponse::new("验证工作空间失败")
        })?;

        if let Some(ver) = verify_server_ver {
            log_info(&request_id, "验证工作空间更新", &format!("id={}, 数据库中 server_ver={}", workspace.id, ver));
        }
    }

    // 更新 notes
    log_info(&request_id, "开始处理笔记同步", &format!("notes_count={}", notes_count));

    // 提前收集客户端推送的数据 ID（用于后续计算真实的 pulled 统计）
    let pushed_note_ids: std::collections::HashSet<String> = notes.iter().map(|n| n.id.clone()).collect();

    for note in notes {
        // 使用 FOR UPDATE 锁定行，防止并发修改
        log_info(&request_id, "查询笔记", &format!("id={}, local_ver={}", note.id, note.server_ver));
        let existing: Option<Note> =
            sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ? AND user_id = ? AND (workspace_id = ? OR workspace_id IS NULL) FOR UPDATE")
                .bind(&note.id)
                .bind(&user_id)
                .bind(&workspace_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| {
                    log_info(&request_id, "查询笔记失败", &e.to_string());
                    ErrorResponse::new("查询笔记失败")
                })?;

        if let Some(existing_note) = existing {
            log_info(&request_id, "笔记已存在", &format!("id={}, server_ver={}", note.id, existing_note.server_ver));
            // 冲突检测：如果服务器版本比本地版本新，根据策略处理
            if existing_note.server_ver > note.server_ver {
                log_info(&request_id, "检测到冲突", &format!("id={}, local_ver={}, server_ver={}", note.id, note.server_ver, existing_note.server_ver));
                match req.conflict_resolution {
                    ConflictResolutionStrategy::KeepServer => {
                        // 服务器版本优先，跳过更新
                        log_info(&request_id, "冲突解决：保留服务器版本", &format!("id={}", note.id));
                        conflicts.push(ConflictInfo {
                            id: note.id.clone(),
                            entity_type: "note".to_string(),
                            local_version: note.server_ver,
                            server_version: existing_note.server_ver,
                            title: note.title.clone(),
                        });
                        continue;
                    }
                    ConflictResolutionStrategy::KeepLocal => {
                        // 本地版本优先，强制更新
                        log_info(&request_id, "冲突解决：本地优先", &format!("note_id={}, local_ver={}, server_ver={}",
                            note.id, note.server_ver, existing_note.server_ver));
                    }
                    ConflictResolutionStrategy::KeepBoth => {
                        // 创建冲突副本
                        let conflict_copy_id = uuid::Uuid::new_v4().to_string();

                        // 构建设备描述
                        let updated_by_device = format!(
                            "{} ({})",
                            req.device_id.as_deref().unwrap_or("unknown"),
                            user_agent.as_deref().unwrap_or("Unknown Device")
                        );

                        sqlx::query(
                            "INSERT INTO notes (id, user_id, workspace_id, title, content, folder_id,
                              is_deleted, deleted_at, created_at, updated_at, server_ver,
                              excerpt, markdown_cache, is_favorite, is_pinned, author,
                              word_count, read_time_minutes,
                              device_id, updated_by_device)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                        )
                        .bind(&conflict_copy_id)
                        .bind(&user_id)
                        .bind(&workspace_id)
                        .bind(&format!("{} (冲突副本-本地)", note.title))
                        .bind(&note.content)
                        .bind(&note.folder_id)
                        .bind(note.is_deleted)
                        .bind(note.deleted_at)
                        .bind(note.created_at)
                        .bind(note.updated_at)
                        .bind(existing_note.server_ver)
                        .bind(&note.excerpt)
                        .bind(&note.markdown_cache)
                        .bind(note.is_favorite)
                        .bind(note.is_pinned)
                        .bind(&note.author)
                        .bind(note.word_count)
                        .bind(note.read_time_minutes)
                        .bind(&req.device_id)
                        .bind(&updated_by_device)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| {
                            log_info(&request_id, "创建冲突副本失败", &e.to_string());
                            ErrorResponse::new("创建冲突副本失败")
                        })?;

                        log_info(&request_id, "创建冲突副本", &format!("original_id={}, copy_id={}", note.id, conflict_copy_id));
                        conflicts.push(ConflictInfo {
                            id: note.id.clone(),
                            entity_type: "note".to_string(),
                            local_version: note.server_ver,
                            server_version: existing_note.server_ver,
                            title: note.title.clone(),
                        });
                        continue;
                    }
                    ConflictResolutionStrategy::ManualMerge => {
                        // 等待手动合并，记录冲突
                        conflicts.push(ConflictInfo {
                            id: note.id.clone(),
                            entity_type: "note".to_string(),
                            local_version: note.server_ver,
                            server_version: existing_note.server_ver,
                            title: note.title.clone(),
                        });
                        continue;
                    }
                }
            } else {
                log_info(&request_id, "无冲突，正常更新", &format!("id={}, server_ver={} -> {}", note.id, existing_note.server_ver, existing_note.server_ver + 1));
            }
        } else {
            log_info(&request_id, "笔记不存在，新建", &format!("id={}, title={}", note.id, note.title));
        }

        // 插入或更新笔记
        // 注意：VALUES(server_ver) + 1 确保第一次插入时 server_ver = 1（客户端发送 0），更新时 server_ver = server_ver + 1
        let new_server_ver = note.server_ver + 1;

        // 构建设备描述
        let updated_by_device = format!(
            "{} ({})",
            req.device_id.as_deref().unwrap_or("unknown"),
            user_agent.as_deref().unwrap_or("Unknown Device")
        );

        sqlx::query(
            "INSERT INTO notes (id, user_id, workspace_id, title, content, folder_id,
                              is_deleted, deleted_at, created_at, updated_at, server_ver,
                              excerpt, markdown_cache, is_favorite, is_pinned, author,
                              word_count, read_time_minutes,
                              device_id, updated_by_device)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
                title = VALUES(title),
                content = VALUES(content),
                folder_id = VALUES(folder_id),
                is_deleted = VALUES(is_deleted),
                deleted_at = VALUES(deleted_at),
                updated_at = UNIX_TIMESTAMP(),
                server_ver = server_ver + 1,
                excerpt = VALUES(excerpt),
                markdown_cache = VALUES(markdown_cache),
                is_favorite = VALUES(is_favorite),
                is_pinned = VALUES(is_pinned),
                author = VALUES(author),
                word_count = VALUES(word_count),
                read_time_minutes = VALUES(read_time_minutes),
                device_id = VALUES(device_id),
                updated_by_device = VALUES(updated_by_device)",
        )
        .bind(&note.id)
        .bind(&user_id)
        .bind(&workspace_id)
        .bind(&note.title)
        .bind(&note.content)
        .bind(&note.folder_id)
        .bind(note.is_deleted)
        .bind(note.deleted_at)
        .bind(note.created_at)
        .bind(note.updated_at)
        .bind(new_server_ver)  // ✅ 使用 new_server_ver（客户端 server_ver + 1）
        .bind(&note.excerpt)
        .bind(&note.markdown_cache)
        .bind(note.is_favorite)
        .bind(note.is_pinned)
        .bind(&note.author)
        .bind(note.word_count)
        .bind(note.read_time_minutes)
        .bind(&req.device_id)
        .bind(&updated_by_device)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "更新笔记失败", &e.to_string());
            ErrorResponse::new("更新笔记失败")
        })?;

        // ✅ 推送成功，递增计数器
        pushed_notes += 1;

        // 验证：只查询 server_ver 字段
        let verify_server_ver: Option<i32> = sqlx::query_scalar(
            "SELECT server_ver FROM notes WHERE id = ? AND user_id = ?"
        )
        .bind(&note.id)
        .bind(&user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "验证笔记失败", &e.to_string());
            ErrorResponse::new("验证笔记失败")
        })?;

        if let Some(ver) = verify_server_ver {
            log_info(&request_id, "验证笔记更新", &format!("id={}, 数据库中 server_ver={}", note.id, ver));
        }
    }

    // 更新 folders（按依赖关系排序：父文件夹优先）
    // 使用迭代方法确保父文件夹先于子文件夹插入（支持多层嵌套）
    let mut remaining_ids: std::collections::HashSet<String> = folders.iter().map(|f| f.id.clone()).collect();
    let mut inserted_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut iteration_count = 0;
    let max_iterations = folders.len() + 1; // 防止无限循环

    while !remaining_ids.is_empty() && iteration_count < max_iterations {
        iteration_count += 1;
        let mut inserted_in_this_iteration = Vec::new();

        for folder_id in &remaining_ids {
            let folder = folders.iter().find(|f| f.id == *folder_id).unwrap();

            // 检查是否可以插入：
            // 1. 是根文件夹（parent_id 为空）
            // 2. 或者父文件夹已经插入
            // 3. 或者父文件夹已经存在于数据库中
            let can_insert = if folder.parent_id.is_none() || folder.parent_id.as_ref().map(|p| p.is_empty()).unwrap_or(false) {
                true // 根文件夹
            } else if let Some(ref parent_id) = folder.parent_id {
                // 检查父文件夹是否已经插入本次同步中
                if inserted_ids.contains(parent_id) {
                    true
                } else {
                    // 检查父文件夹是否已存在于数据库中
                    let parent_exists: bool = sqlx::query_scalar(
                        "SELECT COUNT(*) > 0 FROM folders WHERE id = ? AND user_id = ? AND (workspace_id = ? OR workspace_id IS NULL)"
                    )
                    .bind(parent_id)
                    .bind(&user_id)
                    .bind(&workspace_id)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| {
                        log_info(&request_id, "检查父文件夹失败", &e.to_string());
                        ErrorResponse::new("检查父文件夹失败")
                    })?;
                    parent_exists
                }
            } else {
                true
            };

            if can_insert {
                // 插入文件夹（复用现有逻辑）
                let existing: Option<Folder> =
                    sqlx::query_as::<_, Folder>("SELECT * FROM folders WHERE id = ? AND user_id = ? AND (workspace_id = ? OR workspace_id IS NULL) FOR UPDATE")
                        .bind(&folder.id)
                        .bind(&user_id)
                        .bind(&workspace_id)
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
                        inserted_in_this_iteration.push(folder.id.clone());
                        continue;
                    }
                } else {
                    log_info(&request_id, "文件夹不存在，新建", &format!("id={}, name={}", folder.id, folder.name));
                }

                // 插入或更新文件夹
                let new_server_ver = folder.server_ver + 1;

                // 构建设备描述
                let updated_by_device = format!(
                    "{} ({})",
                    req.device_id.as_deref().unwrap_or("unknown"),
                    user_agent.as_deref().unwrap_or("Unknown Device")
                );

                sqlx::query(
                    "INSERT INTO folders (id, user_id, workspace_id, name, parent_id,
                                        created_at, updated_at, server_ver,
                                        device_id, updated_by_device)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                     ON DUPLICATE KEY UPDATE
                        name = VALUES(name),
                        parent_id = VALUES(parent_id),
                        updated_at = UNIX_TIMESTAMP(),
                        server_ver = server_ver + 1,
                        device_id = VALUES(device_id),
                        updated_by_device = VALUES(updated_by_device)",
                )
                .bind(&folder.id)
                .bind(&user_id)
                .bind(&workspace_id)
                .bind(&folder.name)
                .bind(&folder.parent_id)
                .bind(folder.created_at)
                .bind(folder.updated_at)
                .bind(new_server_ver)  // ✅ 使用 new_server_ver（客户端 server_ver + 1）
                .bind(&req.device_id)
                .bind(&updated_by_device)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    log_info(&request_id, "更新文件夹失败", &e.to_string());
                    ErrorResponse::new("更新文件夹失败")
                })?;

                // ✅ 推送成功，递增计数器
                pushed_folders += 1;

                // 验证：只查询 server_ver 字段
                let verify_server_ver: Option<i32> = sqlx::query_scalar(
                    "SELECT server_ver FROM folders WHERE id = ? AND user_id = ?"
                )
                .bind(&folder.id)
                .bind(&user_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| {
                    log_info(&request_id, "验证文件夹失败", &e.to_string());
                    ErrorResponse::new("验证文件夹失败")
                })?;

                if let Some(ver) = verify_server_ver {
                    log_info(&request_id, "验证文件夹更新", &format!("id={}, 数据库中 server_ver={}", folder.id, ver));
                }

                inserted_ids.insert(folder.id.clone());
                inserted_in_this_iteration.push(folder.id.clone());
            }
        }

        // 从 remaining_ids 中移除已插入的文件夹
        for id in &inserted_in_this_iteration {
            remaining_ids.remove(id);
        }

        if inserted_in_this_iteration.is_empty() && !remaining_ids.is_empty() {
            // 无法插入更多文件夹（可能是循环引用或父 ID 不存在）
            log_info(&request_id, "无法插入部分文件夹", &format!("remaining={:?}", remaining_ids));
            break;
        }
    }

    // 更新 tags
    log_info(&request_id, "开始处理标签同步", &format!("tags_count={}", tags_count));

    // 提前收集 tags 的 ID（用于后续计算 pulled 统计）
    let pushed_tag_ids: std::collections::HashSet<String> = tags.iter().map(|t| t.id.clone()).collect();

    for tag in tags {
        let existing: Option<Tag> =
            sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = ? AND user_id = ? AND (workspace_id = ? OR workspace_id IS NULL) FOR UPDATE")
                .bind(&tag.id)
                .bind(&user_id)
                .bind(&workspace_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| {
                    log_info(&request_id, "查询标签失败", &e.to_string());
                    ErrorResponse::new("查询标签失败")
                })?;

        if let Some(existing_tag) = existing {
            if existing_tag.server_ver > tag.server_ver {
                conflicts.push(ConflictInfo {
                    id: tag.id.clone(),
                    entity_type: "tag".to_string(),
                    local_version: tag.server_ver,
                    server_version: existing_tag.server_ver,
                    title: tag.name.clone(),
                });
                continue;
            }
        }

        // 插入或更新标签
        let new_server_ver = tag.server_ver + 1;

        // 构建设备描述
        let updated_by_device = format!(
            "{} ({})",
            req.device_id.as_deref().unwrap_or("unknown"),
            user_agent.as_deref().unwrap_or("Unknown Device")
        );

        sqlx::query(
            "INSERT INTO tags (id, user_id, workspace_id, name, color,
                              created_at, updated_at, server_ver, device_id, updated_by_device)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
                name = VALUES(name),
                color = VALUES(color),
                updated_at = UNIX_TIMESTAMP(),
                server_ver = server_ver + 1,
                device_id = VALUES(device_id),
                updated_by_device = VALUES(updated_by_device)",
        )
        .bind(&tag.id)
        .bind(&user_id)
        .bind(&workspace_id)
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(tag.created_at)
        .bind(tag.updated_at)
        .bind(new_server_ver)  // ✅ 使用 new_server_ver（客户端 server_ver + 1）
        .bind(&req.device_id)
        .bind(&updated_by_device)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "更新标签失败", &e.to_string());
            ErrorResponse::new("更新标签失败")
        })?;

        // ✅ 推送成功，递增计数器
        pushed_tags += 1;
    }

    // 更新 snapshots（限制每个笔记最多 20 个）
    log_info(&request_id, "开始处理快照同步", &format!("snapshots_count={}", snapshots_count));

    // 提前收集 snapshots 的 ID（用于后续计算 pulled 统计）
    let pushed_snapshot_ids: std::collections::HashSet<String> = snapshots.iter().map(|s| s.id.clone()).collect();

    // 先按 note_id 分组统计本次同步中每个笔记的快照数量
    use std::collections::HashMap;
    let mut snapshots_by_note: HashMap<String, Vec<&NoteVersion>> = HashMap::new();
    for snapshot in &snapshots {
        snapshots_by_note
            .entry(snapshot.note_id.clone())
            .or_insert_with(Vec::new)
            .push(snapshot);
    }

    // 对每个笔记处理快照
    for (note_id, note_snapshots) in &snapshots_by_note {
        // 1. 先删除重复的快照（基于 ID）
        for snapshot in note_snapshots {
            sqlx::query(
                "DELETE FROM note_versions
                 WHERE note_id = ? AND user_id = ? AND (workspace_id = ? OR workspace_id IS NULL)
                 AND id = ?"
            )
            .bind(note_id)
            .bind(&user_id)
            .bind(&workspace_id)
            .bind(&snapshot.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                log_info(&request_id, "删除重复快照失败", &e.to_string());
                ErrorResponse::new("删除重复快照失败")
            })?;
        }

        // 2. 查询当前快照数量
        let current_snapshot_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM note_versions WHERE note_id = ? AND user_id = ? AND (workspace_id = ? OR workspace_id IS NULL)"
        )
        .bind(note_id)
        .bind(&user_id)
        .bind(&workspace_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "查询快照数量失败", &e.to_string());
            ErrorResponse::new("查询快照数量失败")
        })?;

        let new_snapshot_count = note_snapshots.len() as i64;

        // 计算同步后总数量
        let total_after_sync = current_snapshot_count + new_snapshot_count;

        // 3. 如果超过20，需要删除最久远的快照
        if total_after_sync > 20 {
            let to_delete = total_after_sync - 20;
            log_info(&request_id, "快照数量超限，删除最久远的快照",
                     &format!("note_id={}, current={}, new={}, total={}, to_delete={}",
                              note_id, current_snapshot_count, new_snapshot_count, total_after_sync, to_delete));

            // 删除创建时间最久的 to_delete 个快照
            sqlx::query(
                "DELETE FROM note_versions
                 WHERE note_id = ? AND user_id = ? AND (workspace_id = ? OR workspace_id IS NULL)
                 ORDER BY created_at ASC
                 LIMIT ?"
            )
            .bind(note_id)
            .bind(&user_id)
            .bind(&workspace_id)
            .bind(to_delete)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                log_info(&request_id, "删除旧快照失败", &e.to_string());
                ErrorResponse::new("删除旧快照失败")
            })?;

            log_info(&request_id, "已删除最旧的快照",
                     &format!("note_id={}, count={}", note_id, to_delete));
        }
    }

    // 处理所有快照的插入/更新
    for snapshot in snapshots {
        let existing: Option<NoteVersion> =
            sqlx::query_as::<_, NoteVersion>(
                "SELECT * FROM note_versions WHERE id = ? AND user_id = ? AND (workspace_id = ? OR workspace_id IS NULL) FOR UPDATE"
            )
            .bind(&snapshot.id)
            .bind(&user_id)
            .bind(&workspace_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                log_info(&request_id, "查询快照失败", &e.to_string());
                ErrorResponse::new("查询快照失败")
            })?;

        if let Some(existing_snapshot) = existing {
            if existing_snapshot.server_ver > snapshot.server_ver {
                conflicts.push(ConflictInfo {
                    id: snapshot.id.clone(),
                    entity_type: "snapshot".to_string(),
                    local_version: snapshot.server_ver,
                    server_version: existing_snapshot.server_ver,
                    title: snapshot.snapshot_name.clone()
                        .unwrap_or_else(|| snapshot.title.clone()),
                });
                continue;
            }
        }

        // 插入或更新快照
        let new_server_ver = snapshot.server_ver + 1;

        sqlx::query(
            "INSERT INTO note_versions
             (id, note_id, user_id, workspace_id, title, content, snapshot_name, created_at,
              device_id, server_ver)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
                title = VALUES(title),
                content = VALUES(content),
                snapshot_name = VALUES(snapshot_name),
                device_id = VALUES(device_id),
                server_ver = server_ver + 1",
        )
        .bind(&snapshot.id)
        .bind(&snapshot.note_id)
        .bind(&user_id)
        .bind(&workspace_id)
        .bind(&snapshot.title)
        .bind(&snapshot.content)
        .bind(&snapshot.snapshot_name)
        .bind(snapshot.created_at)
        .bind(&req.device_id)
        .bind(new_server_ver)  // ✅ 使用 new_server_ver（客户端 server_ver + 1）
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "更新快照失败", &e.to_string());
            ErrorResponse::new("更新快照失败")
        })?;

        // ✅ 推送成功，递增计数器
        pushed_snapshots += 1;
    }

    // 更新 note_tags 关联
    // 收集客户端推送的 note_tag 复合键（用于后续计算真实的 pulled 统计）
    let pushed_note_tag_keys: std::collections::HashSet<(String, String)> = note_tags
        .iter()
        .map(|nt| (nt.note_id.clone(), nt.tag_id.clone()))
        .collect();

    for relation in note_tags {
        sqlx::query(
            "INSERT IGNORE INTO note_tags (note_id, tag_id, user_id, workspace_id, created_at)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&relation.note_id)
        .bind(&relation.tag_id)
        .bind(&user_id)
        .bind(&workspace_id)
        .bind(relation.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "更新笔记标签关联失败", &e.to_string());
            ErrorResponse::new("更新笔记标签关联失败")
        })?;

        // ✅ 推送成功，递增计数器
        pushed_note_tags += 1;
    }

    // ===== 2. 查询云端更新（包括软删除） =====
    let last_sync = req.last_sync_at.unwrap_or(0);
    log_info(&request_id, "开始查询云端更新", &format!("last_sync_at={}", last_sync));

    // 查询工作空间（不要加 is_deleted = false!）
    let all_workspaces: Vec<Workspace> = sqlx::query_as::<_, Workspace>(
        "SELECT * FROM workspaces
         WHERE user_id = ? AND updated_at > ?"
    )
    .bind(&user_id)
    .bind(last_sync)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        log_info(&request_id, "查询工作空间失败", &e.to_string());
        ErrorResponse::new("查询工作空间失败")
    })?;
    log_info(&request_id, "查询云端工作空间", &format!("found={}", all_workspaces.len()));

    // 查询笔记（不要加 is_deleted = false!）
    let all_notes: Vec<Note> = sqlx::query_as::<_, Note>(
        "SELECT * FROM notes
         WHERE user_id = ? AND (workspace_id = ? OR workspace_id IS NULL) AND updated_at > ?"
    )
    .bind(&user_id)
    .bind(&workspace_id)
    .bind(last_sync)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        log_info(&request_id, "查询笔记失败", &e.to_string());
        ErrorResponse::new("查询笔记失败")
    })?;
    log_info(&request_id, "查询云端笔记", &format!("found={}", all_notes.len()));

    // 查询文件夹
    let all_folders: Vec<Folder> = sqlx::query_as::<_, Folder>(
        "SELECT * FROM folders
         WHERE user_id = ? AND (workspace_id = ? OR workspace_id IS NULL) AND updated_at > ?"
    )
    .bind(&user_id)
    .bind(&workspace_id)
    .bind(last_sync)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        log_info(&request_id, "查询文件夹失败", &e.to_string());
        ErrorResponse::new("查询文件夹失败")
    })?;
    log_info(&request_id, "查询云端文件夹", &format!("found={}", all_folders.len()));

    // 查询标签
    let all_tags: Vec<Tag> = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags
         WHERE user_id = ? AND (workspace_id = ? OR workspace_id IS NULL) AND updated_at > ?"
    )
    .bind(&user_id)
    .bind(&workspace_id)
    .bind(last_sync)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        log_info(&request_id, "查询标签失败", &e.to_string());
        ErrorResponse::new("查询标签失败")
    })?;
    log_info(&request_id, "查询云端标签", &format!("found={}", all_tags.len()));

    // 查询快照（使用 created_at，因为快照创建后不会修改）
    let all_snapshots: Vec<NoteVersion> = sqlx::query_as::<_, NoteVersion>(
        "SELECT * FROM note_versions
         WHERE user_id = ? AND (workspace_id = ? OR workspace_id IS NULL) AND created_at > ?"
    )
    .bind(&user_id)
    .bind(&workspace_id)
    .bind(last_sync)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        log_info(&request_id, "查询快照失败", &e.to_string());
        ErrorResponse::new("查询快照失败")
    })?;
    log_info(&request_id, "查询云端快照", &format!("found={}", all_snapshots.len()));

    // 查询笔记标签关联
    let all_note_tags: Vec<NoteTagRelation> = sqlx::query_as::<_, NoteTagRelation>(
        "SELECT nt.* FROM note_tags nt
         INNER JOIN tags t ON nt.tag_id = t.id
         WHERE t.user_id = ? AND (t.workspace_id = ? OR t.workspace_id IS NULL) AND t.updated_at > ?"
    )
    .bind(&user_id)
    .bind(&workspace_id)
    .bind(last_sync)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        log_info(&request_id, "查询笔记标签关联失败", &e.to_string());
        ErrorResponse::new("查询笔记标签关联失败")
    })?;
    log_info(&request_id, "查询云端笔记标签关联", &format!("found={}", all_note_tags.len()));

    // ===== 3. 分类数据（upserted vs deleted） =====
    // 工作空间：支持软删除，分类 upserted 和 deleted
    let mut upserted_workspaces = Vec::new();
    let mut deleted_workspace_ids = Vec::new();
    for workspace in all_workspaces {
        if workspace.is_deleted {
            deleted_workspace_ids.push(workspace.id);
        } else {
            upserted_workspaces.push(workspace);
        }
    }
    log_info(&request_id, "分类云端工作空间", &format!("upserted={}, deleted={}", upserted_workspaces.len(), deleted_workspace_ids.len()));

    let mut upserted_notes = Vec::new();
    let mut deleted_note_ids = Vec::new();
    for note in all_notes {
        if note.is_deleted {
            deleted_note_ids.push(note.id);
        } else {
            // 详细日志：记录返回给客户端的笔记版本号
            log_info(&request_id, "返回笔记给客户端", &format!("id={}, title={}, server_ver={}", note.id, note.title, note.server_ver));
            upserted_notes.push(note);
        }
    }
    log_info(&request_id, "分类云端笔记", &format!("upserted={}, deleted={}", upserted_notes.len(), deleted_note_ids.len()));

    // 文件夹：支持软删除，分类 upserted 和 deleted
    let mut upserted_folders = Vec::new();
    let mut deleted_folder_ids = Vec::new();
    for folder in all_folders {
        if folder.is_deleted {
            deleted_folder_ids.push(folder.id);
        } else {
            upserted_folders.push(folder);
        }
    }
    log_info(&request_id, "分类云端文件夹", &format!("upserted={}, deleted={}", upserted_folders.len(), deleted_folder_ids.len()));

    // 标签：支持软删除，分类 upserted 和 deleted
    let mut upserted_tags = Vec::new();
    let mut deleted_tag_ids = Vec::new();
    for tag in all_tags {
        if tag.is_deleted {
            deleted_tag_ids.push(tag.id);
        } else {
            upserted_tags.push(tag);
        }
    }
    log_info(&request_id, "分类云端标签", &format!("upserted={}, deleted={}", upserted_tags.len(), deleted_tag_ids.len()));

    // 快照不需要 deleted_ids，因为快照是不可变的，不会被软删除
    let upserted_snapshots = all_snapshots;

    // note_tags 也支持软删除
    let upserted_note_tags: Vec<NoteTagRelation> = all_note_tags.into_iter()
        .filter(|nt| !nt.is_deleted)
        .collect();

    // 提交事务
    tx.commit().await.map_err(|e| {
        log_info(&request_id, "提交事务失败", &e.to_string());
        ErrorResponse::new("提交事务失败")
    })?;
    log_info(&request_id, "事务提交成功", "");

    // ===== 4. 记录同步历史 =====
    let duration_s = Utc::now().timestamp() - start_time;
    let pushed_count = (notes_count + folders_count) as i32;
    let pulled_count = (upserted_notes.len() + upserted_folders.len()
        + deleted_note_ids.len() + deleted_folder_ids.len()) as i32;
    let deleted_count = (deleted_note_ids.len() + deleted_folder_ids.len()) as i32;
    let conflict_count = conflicts.len() as i32;

    if let Err(e) = history_service
        .create(
            &user_id,
            "sync",
            pushed_count,
            pulled_count,
            conflict_count,
            None,
            duration_s * 1000,  // 转换为毫秒
        )
        .await
    {
        log_info(&request_id, "记录同步历史失败", &e.to_string());
    }

    log_info(
        &request_id,
        "同步完成",
        &format!(
            "pushed={}, pulled(upserted)={}, deleted={}, conflicts={}, duration_s={}",
            pushed_count,
            pulled_count,
            deleted_count,
            conflict_count,
            duration_s
        ),
    );

    // ===== 5. 计算真实的 pulled 统计（排除客户端刚推送的数据） =====
    // pushed_workspace_ids, pushed_note_ids, pushed_tag_ids, pushed_snapshot_ids 已在前面收集
    let pushed_folder_ids: std::collections::HashSet<String> = folders.iter().map(|f| f.id.clone()).collect();

    // 计算真实的 pulled 数量（排除客户端刚推送的数据）
    let pulled_workspaces = upserted_workspaces.iter()
        .filter(|w| !pushed_workspace_ids.contains(&w.id))
        .count();
    let pulled_notes = upserted_notes.iter().filter(|n| !pushed_note_ids.contains(&n.id)).count();
    let pulled_folders = upserted_folders.iter().filter(|f| !pushed_folder_ids.contains(&f.id)).count();
    let pulled_tags = upserted_tags.iter().filter(|t| !pushed_tag_ids.contains(&t.id)).count();
    let pulled_snapshots = upserted_snapshots.iter().filter(|s| !pushed_snapshot_ids.contains(&s.id)).count();
    let pulled_note_tags = upserted_note_tags.iter().filter(|nt| !pushed_note_tag_keys.contains(&(nt.note_id.clone(), nt.tag_id.clone()))).count();

    let pushed_total = pushed_workspaces + pushed_notes + pushed_folders + pushed_tags + pushed_snapshots + pushed_note_tags;
    let pulled_total = pulled_workspaces + pulled_notes + pulled_folders + pulled_tags + pulled_snapshots + pulled_note_tags;

    // ===== 6. 返回响应 =====
    Ok(Json(SyncResponse {
        status: if conflicts.is_empty() {
            "success".to_string()
        } else {
            "partial_success".to_string()
        },
        server_time: Utc::now().timestamp(),
        last_sync_at: Utc::now().timestamp(),
        upserted_workspaces,
        upserted_notes,
        upserted_folders,
        upserted_tags,
        upserted_snapshots,
        upserted_note_tags,
        deleted_note_ids,
        deleted_folder_ids,
        deleted_tag_ids,
        deleted_workspace_ids,
        // 推送统计（服务器确认实际更新的数量）
        pushed_workspaces,
        pushed_notes,
        pushed_folders,
        pushed_tags,
        pushed_snapshots,
        pushed_note_tags,
        pushed_total,
        // 拉取统计（服务器端真正的新数据）
        pulled_workspaces,
        pulled_notes,
        pulled_folders,
        pulled_tags,
        pulled_snapshots,
        pulled_note_tags,
        pulled_total,
        conflicts,
    }))
}
