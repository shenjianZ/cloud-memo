use crate::models::{Note, Folder, Tag, NoteSnapshot, NoteTagRelation, SyncRequest, SyncResponse, SyncReport, ConflictInfo, SyncStatus, ConflictStrategy, Workspace};
use crate::models::error::{Result, AppError};
use crate::services::auth_service::AuthService;
use crate::services::crypto::CryptoService;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2_sqlite::rusqlite::{self, params};
use chrono::Utc;
use reqwest::Client;
use std::time::Duration;

/// 同步会话状态
///
/// 记录同步开始时的用户和工作空间状态，用于防止同步过程中的状态变化
#[derive(Clone, Debug)]
pub struct SyncSession {
    pub session_id: String,           // 唯一会话 ID
    pub user_id: String,              // 同步开始时的用户 ID
    pub workspace_id: Option<String>, // 同步开始时的工作空间 ID
    pub started_at: i64,              // 开始时间戳
}

/// 同步服务
///
/// 处理与云服务器的双向同步
#[derive(Clone)]
pub struct SyncService {
    pool: Pool<SqliteConnectionManager>,
    client: Client,
}

impl SyncService {
    /// 创建新的 SyncService 实例
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { pool, client }
    }

    /// 获取数据库连接池（供其他服务使用）
    pub fn get_pool(&self) -> &Pool<SqliteConnectionManager> {
        &self.pool
    }

    /// 开始同步会话（记录当前状态）
    ///
    /// 在同步开始时调用，记录当前用户和工作空间的状态。
    /// 同步过程中可以通过 verify_sync_session 检查状态是否改变。
    fn begin_sync_session(&self) -> Result<SyncSession> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 获取当前用户 ID
        let user_id: String = conn.query_row(
            "SELECT user_id FROM user_auth WHERE is_current = 1 LIMIT 1",
            [],
            |row| row.get(0),
        ).map_err(|_| AppError::NotAuthenticated("User not logged in".to_string()))?;

        // 获取当前工作空间 ID
        let workspace_id: Option<String> = conn.query_row(
            "SELECT id FROM workspaces WHERE user_id = ? AND is_current = 1 AND is_deleted = 0 LIMIT 1",
            params![&user_id],
            |row| row.get(0),
        ).ok();

        let session = SyncSession {
            session_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            workspace_id,
            started_at: chrono::Utc::now().timestamp(),
        };

        log::info!("[SyncService] 开始同步会话: session_id={}, user_id={}, workspace_id={:?}",
            session.session_id, session.user_id, session.workspace_id);

        Ok(session)
    }

    /// 验证同步会话（检查状态是否改变）
    ///
    /// 在同步过程中的关键步骤调用此方法，检查用户或工作空间是否发生变化。
    /// 如果状态改变，应立即取消同步。
    fn verify_sync_session(&self, session: &SyncSession) -> Result<bool> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 获取当前用户 ID
        let current_user_id: Option<String> = conn.query_row(
            "SELECT user_id FROM user_auth WHERE is_current = 1 LIMIT 1",
            [],
            |row| row.get(0),
        ).ok();

        // 获取当前工作空间 ID
        let current_workspace_id: Option<String> = if let Some(ref uid) = current_user_id {
            conn.query_row(
                "SELECT id FROM workspaces WHERE user_id = ? AND is_current = 1 AND is_deleted = 0 LIMIT 1",
                params![uid],
                |row| row.get(0),
            ).ok()
        } else {
            None
        };

        let is_valid = current_user_id.as_ref() == Some(&session.user_id)
            && current_workspace_id == session.workspace_id;

        if !is_valid {
            log::warn!("[SyncService] 同步会话失效: session_id={}, expected user={}, workspace={:?}, current user={:?}, current workspace={:?}",
                session.session_id, session.user_id, session.workspace_id, current_user_id, current_workspace_id);
        }

        Ok(is_valid)
    }

    /// 获取服务器 URL、解密后的 token 和 device_id
    fn get_auth_info(&self) -> Result<(String, String, String)> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT server_url, access_token_encrypted, device_id
             FROM user_auth
             WHERE is_current = 1"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to query auth: {}", e)))?;

        let (server_url, encrypted_token, device_id) = stmt.query_row([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        }).map_err(|_| AppError::NotAuthenticated("User not logged in".to_string()))?;

        // 使用 device_id 解密 token
        let key = CryptoService::derive_key_from_device_id(&device_id);
        let token = CryptoService::decrypt_token(&encrypted_token, &key)?;

        Ok((server_url, token, device_id))
    }

    /// 完整同步（使用统一的 /sync 端点）
    pub async fn full_sync(&self) -> Result<SyncReport> {
        log::info!("Starting full sync");

        // 1. 开始同步会话（记录当前用户和工作空间状态）
        let session = self.begin_sync_session()?;

        // 2. 构建同步请求（包含所有数据）
        if !self.verify_sync_session(&session)? {
            return Err(AppError::SyncCancelled("用户或工作空间已切换".to_string()));
        }
        let request = self.build_sync_request()?;

        // 3. 发送同步请求（统一的 /sync 端点）
        if !self.verify_sync_session(&session)? {
            return Err(AppError::SyncCancelled("用户或工作空间已切换".to_string()));
        }
        let response = self.send_sync_request(&request).await?;

        // 4. 应用服务器响应，并获取修正后的统计（基于实际应用的数量）
        if !self.verify_sync_session(&session)? {
            return Err(AppError::SyncCancelled("用户或工作空间已切换，已取消同步".to_string()));
        }
        let corrected_response = self.apply_sync_response(&response)?;

        // 5. 清理脏标记
        if !self.verify_sync_session(&session)? {
            log::warn!("[SyncService] 应用完成但验证失败，跳过清理脏标记");
        } else {
            self.clear_dirty_markers(&request, response.last_sync_at)?;
        }

        // 6. 更新同步状态
        if !self.verify_sync_session(&session)? {
            log::warn!("[SyncService] 验证失败，跳过更新同步状态");
        } else {
            self.update_sync_state(response.last_sync_at, response.conflicts.len() as i32)?;
        }

        let report = SyncReport {
            success: response.status != "error",
            // ✅ 使用服务器确认的推送统计
            pushed_workspaces: response.pushed_workspaces,
            pushed_notes: response.pushed_notes,
            pushed_folders: response.pushed_folders,
            pushed_tags: response.pushed_tags,
            pushed_snapshots: response.pushed_snapshots,
            pushed_note_tags: response.pushed_note_tags,
            // ✅ 使用修正后的拉取统计（基于实际应用的数据数量）
            pulled_workspaces: corrected_response.pulled_workspaces,
            pulled_notes: corrected_response.pulled_notes,
            pulled_folders: corrected_response.pulled_folders,
            pulled_tags: corrected_response.pulled_tags,
            pulled_snapshots: corrected_response.pulled_snapshots,
            pulled_note_tags: corrected_response.pulled_note_tags,
            // 删除的数据统计
            deleted_workspaces: response.deleted_workspace_ids.len(),
            deleted_notes: response.deleted_note_ids.len(),
            deleted_folders: response.deleted_folder_ids.len(),
            deleted_tags: response.deleted_tag_ids.len(),
            // 冲突和错误
            conflict_count: response.conflicts.len(),
            error: if response.status == "error" {
                Some("Sync failed".to_string())
            } else {
                None
            },
            // 兼容旧版本
            pushed_count: None,
            pulled_count: None,
        };

        log::info!("Full sync completed: pushed_total={}, pulled_total={}, pushed_workspaces={}, pushed_notes={}, pushed_folders={}, pushed_tags={}, pushed_snapshots={}, pushed_note_tags={}, pulled_workspaces={}, pulled_notes={}, pulled_folders={}, pulled_tags={}, pulled_snapshots={}, pulled_note_tags={}, deleted_workspaces={}, deleted_notes={}, deleted_folders={}, deleted_tags={}, conflicts={}",
            response.pushed_total, corrected_response.pulled_total,
            report.pushed_workspaces, report.pushed_notes, report.pushed_folders, report.pushed_tags, report.pushed_snapshots, report.pushed_note_tags,
            report.pulled_workspaces, report.pulled_notes, report.pulled_folders, report.pulled_tags, report.pulled_snapshots, report.pulled_note_tags,
            report.deleted_workspaces, report.deleted_notes, report.deleted_folders, report.deleted_tags, report.conflict_count);

        Ok(report)
    }

    /// 推送到服务器（旧方法，保留以保持兼容性）
    #[deprecated(note = "使用 full_sync() 代替")]
    pub async fn push_to_server(&self) -> Result<SyncResponse> {
        // 使用新的统一同步方法
        let request = self.build_sync_request()?;

        // 发送同步请求
        let response = self.send_sync_request(&request).await?;

        // 应用响应并清理脏标记
        self.apply_sync_response(&response)?;
        self.clear_dirty_markers(&request, response.last_sync_at)?;

        Ok(response)
    }

    /// 从服务器拉取（旧方法，保留以保持兼容性）
    #[deprecated(note = "使用 full_sync() 代替")]
    pub async fn pull_from_server(&self) -> Result<SyncResponse> {
        // 使用新的统一同步方法
        let request = SyncRequest {
            workspaces: None,
            notes: None,
            folders: None,
            tags: None,
            snapshots: None,
            note_tags: None,
            last_sync_at: self.get_last_sync_at()?,
            conflict_resolution: ConflictStrategy::default(),
            device_id: None,
        };

        // 发送同步请求
        let response = self.send_sync_request(&request).await?;

        // 应用响应
        self.apply_sync_response(&response)?;

        Ok(response)
    }

    /// 获取同步状态
    pub fn get_sync_status(&self) -> Result<SyncStatus> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT last_sync_at, pending_count, conflict_count, last_error
             FROM sync_state
             WHERE id = 1"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get sync status: {}", e)))?;

        let status = stmt.query_row([], |row| {
            Ok(SyncStatus {
                last_sync_at: row.get(0)?,
                pending_count: row.get(1)?,
                conflict_count: row.get(2)?,
                last_error: row.get(3)?,
            })
        }).unwrap_or_else(|_| SyncStatus {
            last_sync_at: None,
            pending_count: 0,
            conflict_count: 0,
            last_error: None,
        });

        Ok(status)
    }

    // ===== 私有方法 =====

    /// 获取所有脏笔记
    fn get_dirty_notes(&self) -> Result<Vec<Note>> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, title, content, excerpt, markdown_cache, workspace_id, folder_id,
                    is_favorite, is_deleted, is_pinned, author,
                    created_at, updated_at, deleted_at, word_count, read_time_minutes,
                    server_ver, is_dirty, last_synced_at
             FROM notes
             WHERE is_dirty = 1 AND is_deleted = 0"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get dirty notes: {}", e)))?;

        let notes = stmt.query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                excerpt: row.get(3)?,
                markdown_cache: row.get(4)?,
                workspace_id: row.get(5)?,
                folder_id: row.get(6)?,
                is_favorite: row.get(7)?,
                is_deleted: row.get(8)?,
                is_pinned: row.get(9)?,
                author: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                deleted_at: row.get(13)?,
                word_count: row.get(14)?,
                read_time_minutes: row.get(15)?,
                server_ver: row.get(16)?,
                is_dirty: row.get(17)?,
                last_synced_at: row.get(18)?,
            })
        })
        .map_err(|e| AppError::DatabaseError(format!("Failed to parse notes: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect notes: {}", e)))?;

        log::info!("[SyncService] 获取到 {} 个脏笔记", notes.len());

        // 详细记录每个脏笔记的信息
        for note in &notes {
            log::info!("[SyncService] 脏笔记: id={}, title={}, server_ver={}, is_dirty={}",
                note.id, note.title, note.server_ver, note.is_dirty);
        }

        // 验证查询结果：立即查询数据库确认实际值
        if !notes.is_empty() {
            let first_note = &notes[0];
            let mut verify_stmt = conn.prepare(
                "SELECT server_ver, is_dirty FROM notes WHERE id = ?1"
            ).map_err(|e| AppError::DatabaseError(format!("Failed to prepare verify query: {}", e)))?;

            let (actual_ver, actual_dirty) = verify_stmt.query_row([&first_note.id], |row| {
                Ok((row.get::<_, i32>(0)?, row.get::<_, bool>(1)?))
            }).map_err(|e| AppError::DatabaseError(format!("Failed to verify note: {}", e)))?;

            log::info!("[SyncService] 验证笔记 {}: 查询结果 server_ver={}, is_dirty={}, 实际数据库 server_ver={}, is_dirty={}",
                first_note.id, first_note.server_ver, first_note.is_dirty, actual_ver, actual_dirty);
        }

        Ok(notes)
    }

    /// 获取所有脏工作空间
    fn get_dirty_workspaces(&self) -> Result<Vec<Workspace>> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, description, icon, color, is_default, is_current, sort_order,
                    created_at, updated_at, is_deleted, deleted_at, server_ver, is_dirty, last_synced_at
             FROM workspaces
             WHERE is_dirty = 1 AND is_deleted = 0"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get dirty workspaces: {}", e)))?;

        let workspaces = stmt.query_map([], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                color: row.get(5)?,
                is_default: row.get(6)?,
                is_current: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                is_deleted: row.get(11)?,
                deleted_at: row.get(12)?,
                server_ver: row.get(13)?,
                is_dirty: row.get(14)?,
                last_synced_at: row.get(15)?,
            })
        })
        .map_err(|e| AppError::DatabaseError(format!("Failed to parse workspaces: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect workspaces: {}", e)))?;

        Ok(workspaces)
    }

    /// 获取所有脏文件夹
    fn get_dirty_folders(&self) -> Result<Vec<Folder>> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, icon, color, sort_order,
                    is_deleted, created_at, updated_at, deleted_at,
                    server_ver, is_dirty, last_synced_at
             FROM folders
             WHERE is_dirty = 1 AND is_deleted = 0"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get dirty folders: {}", e)))?;

        let folders = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                workspace_id: row.get(6)?,
                is_deleted: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                deleted_at: row.get(10)?,
                server_ver: row.get(11)?,
                is_dirty: row.get(12)?,
                last_synced_at: row.get(13)?,
            })
        })
        .map_err(|e| AppError::DatabaseError(format!("Failed to parse folders: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect folders: {}", e)))?;

        Ok(folders)
    }

    /// 应用服务器笔记
    fn apply_server_note(&self, note: &Note) -> Result<()> {
        // 检查冲突
        let local_note = self.get_note_by_id(&note.id)?;
        if let Some(local) = local_note {
            if local.is_dirty && note.server_ver > local.server_ver {
                // 冲突：本地有修改，服务器也有新版本
                return Err(AppError::ConflictError(format!(
                    "Conflict in note {}: local version {}, server version {}",
                    note.id, local.server_ver, note.server_ver
                )));
            }
        }

        // 应用服务器更改
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO notes
             (id, title, content, excerpt, markdown_cache, folder_id,
              is_favorite, is_deleted, is_pinned, author,
              created_at, updated_at, deleted_at, word_count, read_time_minutes,
              server_ver, is_dirty, last_synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     ?11, ?12, ?13, ?14, ?15, ?16, 0, ?17)",
            [
                &note.id as &dyn rusqlite::ToSql, &note.title, &note.content, &note.excerpt,
                &note.markdown_cache, &note.folder_id, &note.is_favorite as &dyn rusqlite::ToSql,
                &note.is_deleted as &dyn rusqlite::ToSql, &note.is_pinned as &dyn rusqlite::ToSql,
                &note.author, &note.created_at as &dyn rusqlite::ToSql, &now as &dyn rusqlite::ToSql,
                &note.deleted_at as &dyn rusqlite::ToSql, &note.word_count as &dyn rusqlite::ToSql,
                &note.read_time_minutes as &dyn rusqlite::ToSql, &note.server_ver as &dyn rusqlite::ToSql,
                &now as &dyn rusqlite::ToSql,
            ],
        ).map_err(|e| AppError::DatabaseError(format!("Failed to apply server note: {}", e)))?;

        Ok(())
    }

    /// 应用服务器文件夹
    fn apply_server_folder(&self, folder: &Folder) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO folders
             (id, name, parent_id, icon, color, sort_order,
              is_deleted, created_at, updated_at, deleted_at,
              server_ver, is_dirty, last_synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, ?12)",
            (
                &folder.id, &folder.name, &folder.parent_id, &folder.icon,
                &folder.color, folder.sort_order, folder.is_deleted,
                folder.created_at, now, folder.deleted_at,
                folder.server_ver, now,
            ),
        ).map_err(|e| AppError::DatabaseError(format!("Failed to apply server folder: {}", e)))?;

        Ok(())
    }

    /// 应用服务器工作空间（v2，检查版本）
    fn apply_server_workspace_v2(&self, server_workspace: &crate::models::sync::ServerWorkspace, sync_time: i64) -> Result<bool> {
        let workspace: Workspace = server_workspace.clone().into();
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 检查本地工作空间的 server_ver
        let local_server_ver: Option<i32> = conn.query_row(
            "SELECT server_ver FROM workspaces WHERE id = ?",
            [&workspace.id],
            |row| row.get(0),
        ).ok();

        match local_server_ver {
            Some(local_ver) if local_ver >= server_workspace.server_ver => {
                log::info!("[SyncService] ⏭️ 跳过服务器工作空间（本地版本更新或相同）: id={}, local_ver={}, server_ver={}",
                    workspace.id, local_ver, server_workspace.server_ver);
                return Ok(false);
            },
            _ => {
                log::info!("[SyncService] ✅ 应用服务器工作空间: id={}, name={}, local_ver={:?}, server_ver={}",
                    workspace.id, workspace.name, local_server_ver, server_workspace.server_ver);
            }
        }

        // ⚠️ 注意：不更新 is_current 字段（保留本地设置）
        conn.execute(
            "INSERT INTO workspaces
             (id, user_id, name, description, icon, color, is_default, is_current, sort_order,
              is_deleted, created_at, updated_at, deleted_at, server_ver, is_dirty, last_synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 0, ?15)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                icon = COALESCE(excluded.icon, workspaces.icon),
                color = COALESCE(excluded.color, workspaces.color),
                is_default = excluded.is_default,
                -- is_current 不更新，保留本地设置
                sort_order = excluded.sort_order,
                is_deleted = excluded.is_deleted,
                updated_at = excluded.updated_at,
                deleted_at = excluded.deleted_at,
                server_ver = excluded.server_ver,
                is_dirty = 0,
                last_synced_at = excluded.last_synced_at",
            (
                &workspace.id, &workspace.user_id, &workspace.name, &workspace.description,
                &workspace.icon, &workspace.color, workspace.is_default, &workspace.is_current,
                workspace.sort_order, workspace.is_deleted, workspace.created_at, workspace.updated_at,
                workspace.deleted_at, server_workspace.server_ver, sync_time,
            ),
        ).map_err(|e| AppError::DatabaseError(format!("应用服务器工作空间失败: {}", e)))?;

        Ok(true)
    }

    /// 标记工作空间为已删除
    fn mark_workspace_deleted(&self, workspace_id: &str) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let now = Utc::now().timestamp();

        // 软删除工作空间（但保护默认工作空间）
        let is_default: bool = conn.query_row(
            "SELECT is_default FROM workspaces WHERE id = ?",
            [workspace_id],
            |row| row.get(0),
        ).unwrap_or(false);

        if is_default {
            log::warn!("拒绝删除默认工作空间: {}", workspace_id);
            return Ok(());  // 静默跳过
        }

        conn.execute(
            "UPDATE workspaces SET is_deleted = 1, deleted_at = ?, is_dirty = 0 WHERE id = ?",
            (now, workspace_id),
        ).map_err(|e| AppError::DatabaseError(format!("标记工作空间已删除失败: {}", e)))?;

        log::debug!("Workspace marked as deleted: {}", workspace_id);
        Ok(())
    }

    /// 解决冲突（保留服务器版本，创建本地副本）
    fn resolve_conflict(&self, conflict: &ConflictInfo) -> Result<()> {
        if conflict.entity_type == "note" {
            // 创建本地副本
            let original_note = self.get_note_by_id(&conflict.id)?
                .ok_or(AppError::NotFound(format!("Note {} not found", conflict.id)))?;

            // 使用 Note::conflict_copy() 方法创建冲突副本
            let conflict_note = original_note.conflict_copy("冲突副本 - 本地");

            let conn = self.pool.get()
                .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

            conn.execute(
                "INSERT INTO notes
                 (id, title, content, excerpt, markdown_cache, folder_id,
                  is_favorite, is_deleted, is_pinned, author,
                  created_at, updated_at, deleted_at, word_count, read_time_minutes,
                  server_ver, is_dirty, last_synced_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                         ?11, ?12, ?13, ?14, ?15, ?16, 1, ?17)",
                [
                    &conflict_note.id as &dyn rusqlite::ToSql, &conflict_note.title,
                    &conflict_note.content, &conflict_note.excerpt, &conflict_note.markdown_cache,
                    &conflict_note.folder_id, &conflict_note.is_favorite as &dyn rusqlite::ToSql,
                    &conflict_note.is_deleted as &dyn rusqlite::ToSql,
                    &conflict_note.is_pinned as &dyn rusqlite::ToSql,
                    &conflict_note.author, &conflict_note.created_at as &dyn rusqlite::ToSql,
                    &conflict_note.updated_at as &dyn rusqlite::ToSql,
                    &conflict_note.deleted_at as &dyn rusqlite::ToSql,
                    &conflict_note.word_count as &dyn rusqlite::ToSql,
                    &conflict_note.read_time_minutes as &dyn rusqlite::ToSql,
                    &conflict_note.server_ver as &dyn rusqlite::ToSql,
                    &conflict_note.last_synced_at as &dyn rusqlite::ToSql,
                ],
            ).map_err(|e| AppError::DatabaseError(format!("Failed to create conflict copy: {}", e)))?;

            log::warn!("Created conflict copy for note {} as {}", conflict.id, conflict_note.id);
        }

        Ok(())
    }

    /// 获取笔记（可能返回 None）
    fn get_note_by_id(&self, id: &str) -> Result<Option<Note>> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, title, content, excerpt, markdown_cache, workspace_id, folder_id,
                    is_favorite, is_deleted, is_pinned, author,
                    created_at, updated_at, deleted_at, word_count, read_time_minutes,
                    server_ver, is_dirty, last_synced_at
             FROM notes
             WHERE id = ?1"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get note: {}", e)))?;

        match stmt.query_row([id], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                excerpt: row.get(3)?,
                markdown_cache: row.get(4)?,
                workspace_id: row.get(5)?,
                folder_id: row.get(6)?,
                is_favorite: row.get(7)?,
                is_deleted: row.get(8)?,
                is_pinned: row.get(9)?,
                author: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                deleted_at: row.get(13)?,
                word_count: row.get(14)?,
                read_time_minutes: row.get(15)?,
                server_ver: row.get(16)?,
                is_dirty: row.get(17)?,
                last_synced_at: row.get(18)?,
            })
        }) {
            Ok(note) => Ok(Some(note)),
            Err(_) => Ok(None),
        }
    }

    /// 更新同步状态
    pub fn update_sync_state(&self, last_sync_at: i64, conflict_count: i32) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let pending_count = self.count_pending()?;

        conn.execute(
            "INSERT OR REPLACE INTO sync_state (id, last_sync_at, pending_count, conflict_count)
             VALUES (1, ?1, ?2, ?3)",
            (last_sync_at, pending_count, conflict_count),
        ).map_err(|e| AppError::DatabaseError(format!("Failed to update sync state: {}", e)))?;

        Ok(())
    }

    /// 统计待同步数量
    fn count_pending(&self) -> Result<i32> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let note_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE is_dirty = 1 AND is_deleted = 0",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let folder_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM folders WHERE is_dirty = 1 AND is_deleted = 0",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok(note_count + folder_count)
    }

    /// ===== 新增辅助方法 =====

    /// 获取上次同步时间（别名，供 SingleSyncService 使用）
    pub fn get_last_sync_time(&self) -> Result<Option<i64>> {
        self.get_last_sync_at()
    }

    /// 获取上次同步时间
    pub fn get_last_sync_at(&self) -> Result<Option<i64>> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare("SELECT last_sync_at FROM sync_state WHERE id = 1")?;

        match stmt.query_row([], |row| row.get(0)) {
            Ok(timestamp) => Ok(Some(timestamp)),
            Err(_) => Ok(None),
        }
    }

    /// 构建同步请求（收集所有脏数据）
    fn build_sync_request(&self) -> Result<SyncRequest> {
        use crate::models::ConflictStrategy;

        let dirty_workspaces = self.get_dirty_workspaces()?;
        let dirty_notes = self.get_dirty_notes()?;
        let dirty_folders = self.get_dirty_folders()?;

        log::info!("[SyncService] 构建同步请求: dirty_workspaces={}, dirty_notes={}, dirty_folders={}",
            dirty_workspaces.len(), dirty_notes.len(), dirty_folders.len());

        // 添加调试日志
        if !dirty_notes.is_empty() {
            for note in dirty_notes.iter().take(3) {
                log::info!("[SyncService] 脏笔记: id={}, title={}, server_ver={}, is_dirty={}",
                    note.id, note.title, note.server_ver, note.is_dirty);
            }
        }

        Ok(SyncRequest {
            workspaces: Some(dirty_workspaces.into_iter().map(|w| w.into()).collect()),
            notes: Some(dirty_notes.into_iter().map(|n| n.into()).collect()),
            folders: Some(dirty_folders.into_iter().map(|f| f.into()).collect()),
            tags: Some(self.get_dirty_tags()?.into_iter().map(|t| t.into()).collect()),
            snapshots: Some(self.get_dirty_snapshots()?.into_iter().map(|s| s.into()).collect()),
            note_tags: Some(self.get_note_tags_relations()?.into_iter().map(|nt| nt.into()).collect()),
            last_sync_at: self.get_last_sync_at()?,
            conflict_resolution: ConflictStrategy::default(),
            device_id: None, // 在 send_sync_request 中设置
        })
    }

    /// 获取所有脏标签
    fn get_dirty_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at, updated_at, deleted_at, server_ver, is_dirty, last_synced_at
             FROM tags
             WHERE is_dirty = 1 AND is_deleted = 0"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get dirty tags: {}", e)))?;

        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                workspace_id: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                deleted_at: row.get(6)?,
                server_ver: row.get(7)?,
                is_dirty: row.get(8)?,
                last_synced_at: row.get(9)?,
                is_deleted: false,
            })
        })
        .map_err(|e| AppError::DatabaseError(format!("Failed to parse tags: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect tags: {}", e)))?;

        Ok(tags)
    }

    /// 获取所有脏快照
    fn get_dirty_snapshots(&self) -> Result<Vec<NoteSnapshot>> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 获取当前 workspace_id（通过当前用户的 is_current 标记）
        let workspace_id: Option<String> = {
            // 获取当前用户 ID
            let user_id: Option<String> = conn
                .query_row(
                    "SELECT user_id FROM user_auth WHERE is_current = 1 LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .ok();

            match user_id {
                Some(uid) => {
                    // 查询该用户的当前工作空间（is_current = 1）
                    conn
                        .query_row(
                            "SELECT id FROM workspaces WHERE user_id = ? AND is_current = 1 AND is_deleted = 0 LIMIT 1",
                            params![&uid],
                            |row| row.get(0),
                        )
                        .ok()
                }
                None => None,  // 未登录
            }
        };

        let mut stmt = conn.prepare(
            "SELECT id, note_id, title, content, snapshot_name,
                    created_at, workspace_id, server_ver, is_dirty, last_synced_at
             FROM note_snapshots
             WHERE is_dirty = 1 AND (workspace_id = ? OR workspace_id IS NULL)"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get dirty snapshots: {}", e)))?;

        let snapshots = stmt.query_map(params![workspace_id], |row| {
            Ok(NoteSnapshot {
                id: row.get(0)?,
                note_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                snapshot_name: row.get(4)?,
                created_at: row.get(5)?,
                workspace_id: row.get(6)?,
                server_ver: row.get(7)?,
                is_dirty: row.get(8)?,
                last_synced_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::DatabaseError(format!("Failed to parse snapshots: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect snapshots: {}", e)))?;

        Ok(snapshots)
    }

    /// 获取所有笔记标签关联
    fn get_note_tags_relations(&self) -> Result<Vec<NoteTagRelation>> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 查询所有标签关联
        let mut stmt = conn.prepare(
            "SELECT nt.note_id, nt.tag_id, nt.created_at
             FROM note_tags nt"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get note tags: {}", e)))?;

        let note_tags = stmt.query_map([], |row| {
            Ok(NoteTagRelation {
                note_id: row.get(0)?,
                tag_id: row.get(1)?,
                user_id: String::new(), // 本地不存储 user_id
                created_at: row.get(2)?,
                is_deleted: false,
                deleted_at: None,
            })
        })
        .map_err(|e| AppError::DatabaseError(format!("Failed to parse note tags: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect note tags: {}", e)))?;

        Ok(note_tags)
    }

    /// 应用服务器响应（完整实现）
    /// 返回实际新拉取并应用的数据数量（修正服务器统计）
    pub fn apply_sync_response(&self, response: &SyncResponse) -> Result<SyncResponse> {
        let sync_time = response.last_sync_at;

        // 1. 应用 upserted 数据（新增或更新），统计实际应用的数量
        let mut actually_applied_workspaces = 0usize;
        let mut actually_applied_notes = 0usize;
        let mut actually_applied_folders = 0usize;
        let mut actually_applied_tags = 0usize;
        let mut actually_applied_snapshots = 0usize;
        let mut actually_applied_note_tags = 0usize;

        // ✅ 优先应用 workspaces（其他数据依赖 workspace_id）
        for workspace in &response.upserted_workspaces {
            if self.apply_server_workspace_v2(workspace, sync_time)? {
                actually_applied_workspaces += 1;
            }
        }

        for note in &response.upserted_notes {
            if self.apply_server_note_v2(note, sync_time)? {
                actually_applied_notes += 1;
            }
        }
        for folder in &response.upserted_folders {
            if self.apply_server_folder_v2(folder, sync_time)? {
                actually_applied_folders += 1;
            }
        }
        for tag in &response.upserted_tags {
            if self.apply_server_tag_v2(tag, sync_time)? {
                actually_applied_tags += 1;
            }
        }
        for snapshot in &response.upserted_snapshots {
            if self.apply_server_snapshot_v2(snapshot, sync_time)? {
                actually_applied_snapshots += 1;
            }
        }
        for relation in &response.upserted_note_tags {
            if self.apply_server_note_tag_v2(relation)? {
                actually_applied_note_tags += 1;
            }
        }

        // 2. 应用 deleted 数据（使用软删除）
        for workspace_id in &response.deleted_workspace_ids {
            self.mark_workspace_deleted(workspace_id)?;
        }
        for note_id in &response.deleted_note_ids {
            self.mark_note_deleted(note_id)?;
        }
        for folder_id in &response.deleted_folder_ids {
            self.mark_folder_deleted(folder_id)?;
        }
        for tag_id in &response.deleted_tag_ids {
            self.mark_tag_deleted(tag_id)?;
        }

        // 3. 处理冲突
        for conflict in &response.conflicts {
            self.resolve_conflict(conflict)?;
        }

        // 4. 返回修正后的统计（使用实际应用的数量）
        let mut corrected_response = response.clone();
        corrected_response.pulled_workspaces = actually_applied_workspaces;
        corrected_response.pulled_notes = actually_applied_notes;
        corrected_response.pulled_folders = actually_applied_folders;
        corrected_response.pulled_tags = actually_applied_tags;
        corrected_response.pulled_snapshots = actually_applied_snapshots;
        corrected_response.pulled_note_tags = actually_applied_note_tags;
        corrected_response.pulled_total = actually_applied_workspaces + actually_applied_notes + actually_applied_folders
            + actually_applied_tags + actually_applied_snapshots + actually_applied_note_tags;

        Ok(corrected_response)
    }

    /// 清理脏标记
    pub fn clear_dirty_markers(&self, request: &SyncRequest, sync_time: i64) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        log::info!("[SyncService] 开始清理脏标记: sync_time={}", sync_time);

        // ✅ 清理 workspaces
        if let Some(workspaces) = &request.workspaces {
            log::info!("[SyncService] 清理 {} 个工作空间的脏标记", workspaces.len());
            for workspace in workspaces {
                conn.execute(
                    "UPDATE workspaces SET is_dirty = 0, last_synced_at = ? WHERE id = ?",
                    (sync_time, &workspace.id),
                ).map_err(|e| AppError::DatabaseError(format!("清除工作空间脏标记失败: {}", e)))?;
            }
        }

        // 清理 notes
        if let Some(notes) = &request.notes {
            log::info!("[SyncService] 清理 {} 个笔记的脏标记", notes.len());
            for note in notes {
                let rows_affected = conn.execute(
                    "UPDATE notes SET is_dirty = 0, last_synced_at = ? WHERE id = ?",
                    (sync_time, &note.id),
                ).map_err(|e| AppError::DatabaseError(format!("清除笔记脏标记失败: {}", e)))?;

                // 验证清理结果
                if rows_affected > 0 {
                    log::info!("[SyncService] ✅ 清理笔记脏标记成功: id={}, rows_affected={}", note.id, rows_affected);
                } else {
                    log::warn!("[SyncService] ⚠️ 清理笔记脏标记失败: id={}, rows_affected=0（笔记不存在或已被清理）", note.id);
                }
            }
        }

        // 清理 folders
        if let Some(folders) = &request.folders {
            log::info!("[SyncService] 清理 {} 个文件夹的脏标记", folders.len());
            for folder in folders {
                let rows_affected = conn.execute(
                    "UPDATE folders SET is_dirty = 0, last_synced_at = ? WHERE id = ?",
                    (sync_time, &folder.id),
                ).map_err(|e| AppError::DatabaseError(format!("清除文件夹脏标记失败: {}", e)))?;

                // 验证清理结果
                if rows_affected > 0 {
                    log::info!("[SyncService] ✅ 清理文件夹脏标记成功: id={}, rows_affected={}", folder.id, rows_affected);
                } else {
                    log::warn!("[SyncService] ⚠️ 清理文件夹脏标记失败: id={}, rows_affected=0（文件夹不存在或已被清理）", folder.id);
                }
            }
        }

        // 清理 tags
        if let Some(tags) = &request.tags {
            log::info!("[SyncService] 清理 {} 个标签的脏标记", tags.len());
            for tag in tags {
                conn.execute(
                    "UPDATE tags SET is_dirty = 0, last_synced_at = ? WHERE id = ?",
                    (sync_time, &tag.id),
                ).map_err(|e| AppError::DatabaseError(format!("清除标签脏标记失败: {}", e)))?;
            }
        }

        // 清理 snapshots
        if let Some(snapshots) = &request.snapshots {
            log::info!("[SyncService] 清理 {} 个快照的脏标记", snapshots.len());
            for snapshot in snapshots {
                conn.execute(
                    "UPDATE note_snapshots SET is_dirty = 0, last_synced_at = ? WHERE id = ?",
                    (sync_time, &snapshot.id),
                ).map_err(|e| AppError::DatabaseError(format!("清除快照脏标记失败: {}", e)))?;
            }
        }

        log::info!("[SyncService] 清理脏标记完成");
        Ok(())
    }

    /// 应用服务器笔记（v2，接受 ServerNote）
    /// 返回是否真的应用了数据（true = 应用/更新，false = 跳过）
    fn apply_server_note_v2(&self, server_note: &crate::models::sync::ServerNote, sync_time: i64) -> Result<bool> {
        let note: Note = server_note.clone().into();
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 获取当前工作空间 ID（通过当前用户的 is_current 标记）
        let workspace_id: Option<String> = {
            // 获取当前用户 ID
            let user_id: Option<String> = conn
                .query_row(
                    "SELECT user_id FROM user_auth WHERE is_current = 1 LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .ok();

            match user_id {
                Some(uid) => {
                    // 查询该用户的当前工作空间（is_current = 1）
                    conn
                        .query_row(
                            "SELECT id FROM workspaces WHERE user_id = ? AND is_current = 1 AND is_deleted = 0 LIMIT 1",
                            params![&uid],
                            |row| row.get(0),
                        )
                        .ok()
                }
                None => None,  // 未登录
            }
        };

        // 检查本地笔记的 server_ver，只在服务器更新时才应用
        let local_server_ver: Option<i32> = conn.query_row(
            "SELECT server_ver FROM notes WHERE id = ?",
            [&note.id],
            |row| row.get(0)
        ).ok();

        match local_server_ver {
            Some(local_ver) if local_ver >= note.server_ver => {
                // 本地版本 >= 服务器版本，跳过更新
                log::info!("[SyncService] ⏭️ 跳过服务器笔记（本地版本更新或相同）: id={}, local_ver={}, server_ver={}",
                    note.id, local_ver, note.server_ver);
                return Ok(false);  // ❌ 没有应用数据
            },
            _ => {
                // 服务器版本更新，执行更新
                log::info!("[SyncService] ✅ 应用服务器笔记: id={}, title={}, local_ver={:?}, server_ver={}, sync_time={}",
                    note.id, note.title, local_server_ver, note.server_ver, sync_time);
            }
        }

        let rows_affected = conn.execute(
            "INSERT INTO notes
             (id, title, content, excerpt, markdown_cache, folder_id, workspace_id,
              is_favorite, is_deleted, is_pinned, author,
              created_at, updated_at, deleted_at, word_count, read_time_minutes,
              server_ver, is_dirty, last_synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     ?11, ?12, ?13, ?14, ?15, ?16, ?17, 0, ?18)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                excerpt = excluded.excerpt,
                markdown_cache = COALESCE(excluded.markdown_cache, notes.markdown_cache),
                folder_id = excluded.folder_id,
                workspace_id = excluded.workspace_id,
                is_favorite = excluded.is_favorite,
                is_deleted = excluded.is_deleted,
                is_pinned = excluded.is_pinned,
                author = COALESCE(excluded.author, notes.author),
                updated_at = excluded.updated_at,
                deleted_at = excluded.deleted_at,
                word_count = excluded.word_count,
                read_time_minutes = excluded.read_time_minutes,
                server_ver = excluded.server_ver,
                is_dirty = 0,
                last_synced_at = excluded.last_synced_at",
            [
                &note.id as &dyn rusqlite::ToSql, &note.title, &note.content, &note.excerpt,
                &note.markdown_cache, &note.folder_id, &workspace_id,
                &note.is_favorite as &dyn rusqlite::ToSql, &note.is_deleted as &dyn rusqlite::ToSql,
                &note.is_pinned as &dyn rusqlite::ToSql, &note.author,
                &note.created_at as &dyn rusqlite::ToSql, &note.updated_at as &dyn rusqlite::ToSql,
                &note.deleted_at as &dyn rusqlite::ToSql, &note.word_count as &dyn rusqlite::ToSql,
                &note.read_time_minutes as &dyn rusqlite::ToSql, &note.server_ver as &dyn rusqlite::ToSql,
                &sync_time as &dyn rusqlite::ToSql,
            ],
        ).map_err(|e| AppError::DatabaseError(format!("Failed to apply server note: {}", e)))?;

        log::info!("[SyncService] 笔记数据库更新完成: id={}, rows_affected={}, 新 server_ver={}, is_dirty=0",
            note.id, rows_affected, note.server_ver);

        Ok(true)  // ✅ 成功应用了数据
    }

    /// 应用服务器文件夹（v2，接受 ServerFolder）
    fn apply_server_folder_v2(&self, server_folder: &crate::models::sync::ServerFolder, sync_time: i64) -> Result<bool> {
        let folder: Folder = server_folder.clone().into();
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 获取当前工作空间 ID（通过当前用户的 is_current 标记）
        let workspace_id: Option<String> = {
            // 获取当前用户 ID
            let user_id: Option<String> = conn
                .query_row(
                    "SELECT user_id FROM user_auth WHERE is_current = 1 LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .ok();

            match user_id {
                Some(uid) => {
                    // 查询该用户的当前工作空间（is_current = 1）
                    conn
                        .query_row(
                            "SELECT id FROM workspaces WHERE user_id = ? AND is_current = 1 AND is_deleted = 0 LIMIT 1",
                            params![&uid],
                            |row| row.get(0),
                        )
                        .ok()
                }
                None => None,  // 未登录
            }
        };

        // 检查本地文件夹的 server_ver，只在服务器更新时才应用
        let local_server_ver: Option<i32> = conn.query_row(
            "SELECT server_ver FROM folders WHERE id = ?",
            [&folder.id],
            |row| row.get(0)
        ).ok();

        match local_server_ver {
            Some(local_ver) if local_ver >= server_folder.server_ver => {
                // 本地版本 >= 服务器版本，跳过更新
                log::info!("[SyncService] ⏭️ 跳过服务器文件夹（本地版本更新或相同）: id={}, local_ver={}, server_ver={}",
                    folder.id, local_ver, server_folder.server_ver);
                return Ok(false);  // ❌ 没有应用数据
            },
            _ => {
                // 服务器版本更新，执行更新
                log::info!("[SyncService] ✅ 应用服务器文件夹: id={}, name={}, local_ver={:?}, server_ver={}",
                    folder.id, folder.name, local_server_ver, server_folder.server_ver);
            }
        }

        conn.execute(
            "INSERT INTO folders
             (id, name, parent_id, icon, color, sort_order, workspace_id,
              is_deleted, created_at, updated_at, deleted_at,
              server_ver, is_dirty, last_synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0, ?13)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                parent_id = excluded.parent_id,
                icon = COALESCE(excluded.icon, folders.icon),
                color = COALESCE(excluded.color, folders.color),
                sort_order = excluded.sort_order,
                workspace_id = excluded.workspace_id,
                is_deleted = excluded.is_deleted,
                updated_at = excluded.updated_at,
                deleted_at = excluded.deleted_at,
                server_ver = excluded.server_ver,
                is_dirty = 0,
                last_synced_at = excluded.last_synced_at",
            (
                &folder.id, &folder.name, &folder.parent_id, &folder.icon,
                &folder.color, folder.sort_order, &workspace_id,
                folder.is_deleted, folder.created_at, sync_time, folder.deleted_at,
                server_folder.server_ver, sync_time,
            ),
        ).map_err(|e| AppError::DatabaseError(format!("Failed to apply server folder: {}", e)))?;

        Ok(true)  // ✅ 成功应用了数据
    }

    /// 应用服务器标签（v2，检查版本）
    fn apply_server_tag_v2(&self, server_tag: &crate::models::sync::ServerTag, sync_time: i64) -> Result<bool> {
        let tag: Tag = server_tag.clone().into();
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 获取当前工作空间 ID（通过当前用户的 is_current 标记）
        let workspace_id: Option<String> = {
            // 获取当前用户 ID
            let user_id: Option<String> = conn
                .query_row(
                    "SELECT user_id FROM user_auth WHERE is_current = 1 LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .ok();

            match user_id {
                Some(uid) => {
                    // 查询该用户的当前工作空间（is_current = 1）
                    conn
                        .query_row(
                            "SELECT id FROM workspaces WHERE user_id = ? AND is_current = 1 AND is_deleted = 0 LIMIT 1",
                            params![&uid],
                            |row| row.get(0),
                        )
                        .ok()
                }
                None => None,  // 未登录
            }
        };

        // 检查本地标签的 server_ver
        let local_server_ver: Option<i32> = conn.query_row(
            "SELECT server_ver FROM tags WHERE id = ?",
            [&tag.id],
            |row| row.get(0)
        ).ok();

        match local_server_ver {
            Some(local_ver) if local_ver >= server_tag.server_ver => {
                log::info!("[SyncService] ⏭️ 跳过服务器标签（本地版本更新或相同）: id={}, local_ver={}, server_ver={}",
                    tag.id, local_ver, server_tag.server_ver);
                return Ok(false);
            },
            _ => {
                log::info!("[SyncService] ✅ 应用服务器标签: id={}, name={}, local_ver={:?}, server_ver={}",
                    tag.id, tag.name, local_server_ver, server_tag.server_ver);
            }
        }

        conn.execute(
            "INSERT INTO tags
             (id, name, color, workspace_id, created_at, updated_at, deleted_at, server_ver, is_dirty, last_synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                color = COALESCE(excluded.color, tags.color),
                workspace_id = excluded.workspace_id,
                updated_at = excluded.updated_at,
                deleted_at = excluded.deleted_at,
                server_ver = excluded.server_ver,
                is_dirty = 0,
                last_synced_at = excluded.last_synced_at",
            (
                &tag.id, &tag.name, &tag.color, &workspace_id, tag.created_at, tag.updated_at,
                tag.deleted_at, server_tag.server_ver, sync_time,
            ),
        ).map_err(|e| AppError::DatabaseError(format!("应用服务器标签失败: {}", e)))?;

        Ok(true)
    }

    /// 应用服务器快照（v2，检查版本）
    fn apply_server_snapshot_v2(&self, server_snapshot: &crate::models::sync::ServerNoteSnapshot, sync_time: i64) -> Result<bool> {
        let snapshot: NoteSnapshot = server_snapshot.clone().into();
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 获取当前工作空间 ID（通过当前用户的 is_current 标记）
        let workspace_id: Option<String> = {
            // 获取当前用户 ID
            let user_id: Option<String> = conn
                .query_row(
                    "SELECT user_id FROM user_auth WHERE is_current = 1 LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .ok();

            match user_id {
                Some(uid) => {
                    // 查询该用户的当前工作空间（is_current = 1）
                    conn
                        .query_row(
                            "SELECT id FROM workspaces WHERE user_id = ? AND is_current = 1 AND is_deleted = 0 LIMIT 1",
                            params![&uid],
                            |row| row.get(0),
                        )
                        .ok()
                }
                None => None,  // 未登录
            }
        };

        // 检查本地快照的 server_ver
        let local_server_ver: Option<i32> = conn.query_row(
            "SELECT server_ver FROM note_snapshots WHERE id = ?",
            [&snapshot.id],
            |row| row.get(0)
        ).ok();

        match local_server_ver {
            Some(local_ver) if local_ver >= server_snapshot.server_ver => {
                log::info!("[SyncService] ⏭️ 跳过服务器快照（本地版本更新或相同）: id={}, local_ver={}, server_ver={}",
                    snapshot.id, local_ver, server_snapshot.server_ver);
                return Ok(false);
            },
            _ => {
                log::info!("[SyncService] ✅ 应用服务器快照: id={}, note_id={}, local_ver={:?}, server_ver={}",
                    snapshot.id, snapshot.note_id, local_server_ver, server_snapshot.server_ver);
            }
        }

        conn.execute(
            "INSERT INTO note_snapshots
             (id, note_id, title, content, snapshot_name, workspace_id,
              created_at, server_ver, is_dirty, last_synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9)
             ON CONFLICT(id) DO UPDATE SET
                note_id = excluded.note_id,
                title = excluded.title,
                content = excluded.content,
                snapshot_name = COALESCE(excluded.snapshot_name, note_snapshots.snapshot_name),
                workspace_id = excluded.workspace_id,
                created_at = excluded.created_at,
                server_ver = excluded.server_ver,
                is_dirty = 0,
                last_synced_at = excluded.last_synced_at",
            (
                &snapshot.id, &snapshot.note_id, &snapshot.title,
                &snapshot.content, &snapshot.snapshot_name, &workspace_id,
                snapshot.created_at,
                server_snapshot.server_ver,
                sync_time,
            ),
        ).map_err(|e| AppError::DatabaseError(format!("应用服务器快照失败: {}", e)))?;

        Ok(true)
    }

    /// 应用服务器笔记标签关联（v2，返回是否真的插入了）
    fn apply_server_note_tag_v2(&self, server_relation: &crate::models::sync::ServerNoteTagRelation) -> Result<bool> {
        let relation: NoteTagRelation = server_relation.clone().into();
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 获取当前工作空间 ID（通过当前用户的 is_current 标记）
        let workspace_id: Option<String> = {
            // 获取当前用户 ID
            let user_id: Option<String> = conn
                .query_row(
                    "SELECT user_id FROM user_auth WHERE is_current = 1 LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .ok();

            match user_id {
                Some(uid) => {
                    // 查询该用户的当前工作空间（is_current = 1）
                    conn
                        .query_row(
                            "SELECT id FROM workspaces WHERE user_id = ? AND is_current = 1 AND is_deleted = 0 LIMIT 1",
                            params![&uid],
                            |row| row.get(0),
                        )
                        .ok()
                }
                None => None,  // 未登录
            }
        };

        let rows_affected = conn.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id, workspace_id, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            (&relation.note_id, &relation.tag_id, &workspace_id, relation.created_at),
        ).map_err(|e| AppError::DatabaseError(format!("应用服务器笔记标签关联失败: {}", e)))?;

        // rows_affected > 0 表示真的插入了新数据，= 0 表示已存在（被 IGNORE）
        Ok(rows_affected > 0)
    }

    /// 标记笔记为已删除
    fn mark_note_deleted(&self, note_id: &str) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE notes SET is_deleted = 1, deleted_at = ?, is_dirty = 0 WHERE id = ?",
            (now, note_id),
        ).map_err(|e| AppError::DatabaseError(format!("标记笔记已删除失败: {}", e)))?;

        log::debug!("Note marked as deleted: {}", note_id);
        Ok(())
    }

    /// 标记文件夹为已删除（服务器删除）
    fn mark_folder_deleted(&self, folder_id: &str) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let now = Utc::now().timestamp();

        // 软删除文件夹及所有子文件夹
        conn.execute(
            "WITH RECURSIVE folder_tree AS (
                SELECT id FROM folders WHERE id = ?1
                UNION ALL
                SELECT f.id FROM folders f
                INNER JOIN folder_tree ft ON f.parent_id = ft.id
                WHERE f.is_deleted = 0
            )
            UPDATE folders SET is_deleted = 1, deleted_at = ?2, is_dirty = 0
            WHERE id IN folder_tree",
            (folder_id, now),
        ).map_err(|e| AppError::DatabaseError(format!("标记文件夹删除失败: {}", e)))?;

        log::debug!("Folder marked as deleted: {}", folder_id);
        Ok(())
    }

    /// 标记标签为已删除（服务器删除）
    fn mark_tag_deleted(&self, tag_id: &str) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let now = Utc::now().timestamp();

        // 软删除标签
        conn.execute(
            "UPDATE tags SET is_deleted = 1, deleted_at = ?, is_dirty = 0 WHERE id = ?",
            (now, tag_id),
        ).map_err(|e| AppError::DatabaseError(format!("标记标签删除失败: {}", e)))?;

        // 同时软删除所有关联的 note_tags
        conn.execute(
            "UPDATE note_tags SET is_deleted = 1, deleted_at = ? WHERE tag_id = ?",
            (now, tag_id),
        ).map_err(|e| AppError::DatabaseError(format!("标记标签的笔记关联删除失败: {}", e)))?;

        log::debug!("Tag marked as deleted: {}", tag_id);
        Ok(())
    }

    /// ===== 统一同步方法 =====

    /// 发送同步请求到服务器（统一的 /sync 端点）
    pub async fn send_sync_request(&self, request: &SyncRequest) -> Result<SyncResponse> {
        let (server_url, token, device_id) = self.get_auth_info()?;
        let url = format!("{}/sync", server_url.trim_end_matches('/'));

        log::info!("[SyncService] 发送同步请求到: {}, device_id={}", url, device_id);

        // 创建包含 device_id 的请求
        let mut request_with_device = request.clone();
        request_with_device.device_id = Some(device_id.clone());

        // 构建 User-Agent
        let user_agent = build_user_agent();

        // 发送请求
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("User-Agent", user_agent.clone())
            .json(&request_with_device)
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to send sync request: {}", e);
                return Err(AppError::NetworkError(format!("同步请求失败: {}", e)));
            }
        };

        let status = response.status();

        // 处理 401 未授权（尝试刷新 token）
        if status.as_u16() == 401 {
            log::info!("Access token expired, attempting to refresh");

            let auth_service = AuthService::new(self.pool.clone());
            let refresh_result = auth_service.refresh_access_token().await;

            match refresh_result {
                Ok(_) => {
                    log::info!("Token refreshed successfully, retrying sync request");

                    let (server_url, new_token, device_id) = self.get_auth_info()?;
                    let mut request_with_device = request.clone();
                    request_with_device.device_id = Some(device_id);

                    let response = self.client
                        .post(&format!("{}/sync", server_url.trim_end_matches('/')))
                        .header("Authorization", format!("Bearer {}", new_token))
                        .header("Content-Type", "application/json")
                        .header("User-Agent", user_agent)
                        .json(&request_with_device)
                        .send()
                        .await
                        .map_err(|e| {
                            log::error!("Failed to retry sync request: {}", e);
                            AppError::NetworkError(format!("重试同步请求失败: {}", e))
                        })?;

                    return self.parse_sync_response(response).await;
                }
                Err(e) => {
                    log::error!("Failed to refresh token: {}", e);
                    return Err(AppError::AuthenticationError(format!("Token 刷新失败: {}", e)));
                }
            }
        }

        self.parse_sync_response(response).await
    }

    /// 解析同步响应
    async fn parse_sync_response(&self, response: reqwest::Response) -> Result<SyncResponse> {
        let status = response.status();

        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            log::error!("Failed to parse response: {}", e);
            AppError::NetworkError(format!("响应无效: {}", e))
        })?;

        if !status.is_success() {
            let error_msg = response_json["error"]
                .as_str()
                .unwrap_or("Unknown error");
            log::error!("Server returned error {}: {}", status, error_msg);
            return Err(AppError::SyncError(error_msg.to_string()));
        }

        let sync_response: SyncResponse = serde_json::from_value(response_json).map_err(|e| {
            log::error!("Failed to parse sync response: {}", e);
            AppError::NetworkError(format!("同步响应无效: {}", e))
        })?;

        log::info!("[SyncService] 同步响应: status={}, upserted_notes={}, conflicts={}",
            sync_response.status,
            sync_response.upserted_notes.len(),
            sync_response.conflicts.len()
        );

        Ok(sync_response)
    }
}

/// 构建 User-Agent 头部
fn build_user_agent() -> String {
    format!(
        "MarkdownNotes/{} ({})",
        env!("CARGO_PKG_VERSION"),
        get_platform_info()
    )
}

/// 获取平台信息字符串
fn get_platform_info() -> String {
    #[cfg(target_os = "windows")]
    { "Windows NT 10.0; Win64; x64".to_string() }

    #[cfg(target_os = "macos")]
    { "Macintosh; Intel Mac OS X".to_string() }

    #[cfg(target_os = "linux")]
    { "X11; Linux x86_64".to_string() }

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    )))]
    { "Unknown".to_string() }
}
