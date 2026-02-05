use crate::models::{Note, Folder, SyncRequest, SyncResponse, SyncReport, ConflictInfo, SyncStatus};
use crate::models::error::{Result, AppError};
use crate::services::auth_service::AuthService;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2_sqlite::rusqlite;
use chrono::Utc;
use reqwest::Client;
use std::time::Duration;

/// 同步服务
///
/// 处理与云服务器的双向同步
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

    /// 获取服务器 URL 和解密后的 token
    fn get_auth_info(&self) -> Result<(String, String)> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT server_url, access_token_encrypted
             FROM user_auth
             WHERE id = 1"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to query auth: {}", e)))?;

        let (server_url, encrypted_token) = stmt.query_row([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
            ))
        }).map_err(|_| AppError::NotAuthenticated("User not logged in".to_string()))?;

        // 使用 AuthService 的解密方法
        let auth_service = AuthService::new(self.pool.clone());
        let token = auth_service.decrypt_token(&encrypted_token)?;

        Ok((server_url, token))
    }

    /// 完整同步
    pub async fn full_sync(&self) -> Result<SyncReport> {
        log::info!("Starting full sync");

        let mut report = SyncReport {
            success: false,
            pushed_count: 0,
            pulled_count: 0,
            conflict_count: 0,
            error: None,
        };

        // 1. 推送本地更改到服务器
        match self.push_to_server().await {
            Ok(push_response) => {
                report.pushed_count = push_response.notes.len() + push_response.folders.len();
                report.conflict_count = push_response.conflicts.len();

                // 处理冲突
                for conflict in &push_response.conflicts {
                    self.resolve_conflict(conflict)?;
                }
            }
            Err(e) => {
                log::error!("Failed to push to server: {}", e);
                report.error = Some(format!("Push failed: {}", e));
                return Ok(report);
            }
        }

        // 2. 从服务器拉取更改
        match self.pull_from_server().await {
            Ok(pull_response) => {
                report.pulled_count = pull_response.notes.len() + pull_response.folders.len();

                // 应用服务器更改
                for note in &pull_response.notes {
                    self.apply_server_note(note)?;
                }
                for folder in &pull_response.folders {
                    self.apply_server_folder(folder)?;
                }

                // 更新同步状态
                self.update_sync_state(Utc::now().timestamp())?;
            }
            Err(e) => {
                log::error!("Failed to pull from server: {}", e);
                report.error = Some(format!("Pull failed: {}", e));
                return Ok(report);
            }
        }

        report.success = true;
        log::info!("Full sync completed: pushed={}, pulled={}, conflicts={}",
            report.pushed_count, report.pulled_count, report.conflict_count);

        Ok(report)
    }

    /// 推送到服务器
    pub async fn push_to_server(&self) -> Result<SyncResponse> {
        // 获取所有脏数据
        let dirty_notes = self.get_dirty_notes()?;
        let dirty_folders = self.get_dirty_folders()?;

        if dirty_notes.is_empty() && dirty_folders.is_empty() {
            log::info!("No dirty data to push");
            return Ok(SyncResponse {
                notes: vec![],
                folders: vec![],
                conflicts: vec![],
                server_time: Utc::now().timestamp(),
            });
        }

        let request = SyncRequest {
            notes: dirty_notes,
            folders: dirty_folders,
            last_sync_at: self.get_last_sync_at()?,
        };

        // 发送 HTTP POST 到服务器
        let (server_url, token) = self.get_auth_info()?;
        let url = format!("{}/sync/push", server_url.trim_end_matches('/'));

        log::info!("Pushing {} notes and {} folders to {}",
            request.notes.len(), request.folders.len(), url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to send push request: {}", e);
                AppError::NetworkError(format!("Push request failed: {}", e))
            })?;

        let status = response.status();

        // 先尝试解析响应为 JSON
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            log::error!("Failed to parse response: {}", e);
            AppError::NetworkError(format!("Invalid response: {}", e))
        })?;

        if !status.is_success() {
            // 解析错误消息
            let error_msg = response_json["error"]
                .as_str()
                .unwrap_or("Unknown error");
            log::error!("Server returned error {}: {}", status, error_msg);
            return Err(AppError::SyncError(error_msg.to_string()));
        }

        let sync_response: SyncResponse = serde_json::from_value(response_json).map_err(|e| {
            log::error!("Failed to parse sync response: {}", e);
            AppError::NetworkError(format!("Invalid sync response: {}", e))
        })?;

        log::info!("Push completed: {} conflicts", sync_response.conflicts.len());
        Ok(sync_response)
    }

    /// 从服务器拉取
    pub async fn pull_from_server(&self) -> Result<SyncResponse> {
        let last_sync_at = self.get_last_sync_at()?;

        // 发送 HTTP POST 到服务器
        let (server_url, token) = self.get_auth_info()?;
        let url = format!("{}/sync/pull", server_url.trim_end_matches('/'));

        log::info!("Pulling from server since {}", last_sync_at.unwrap_or(0));

        let request_body = serde_json::json!({
            "last_sync_at": last_sync_at.unwrap_or(0)
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to send pull request: {}", e);
                AppError::NetworkError(format!("Pull request failed: {}", e))
            })?;

        let status = response.status();

        // 先尝试解析响应为 JSON
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            log::error!("Failed to parse response: {}", e);
            AppError::NetworkError(format!("Invalid response: {}", e))
        })?;

        if !status.is_success() {
            // 解析错误消息
            let error_msg = response_json["error"]
                .as_str()
                .unwrap_or("Unknown error");
            log::error!("Server returned error {}: {}", status, error_msg);
            return Err(AppError::SyncError(error_msg.to_string()));
        }

        let sync_response: SyncResponse = serde_json::from_value(response_json).map_err(|e| {
            log::error!("Failed to parse sync response: {}", e);
            AppError::NetworkError(format!("Invalid sync response: {}", e))
        })?;

        log::info!("Pull completed: {} notes, {} folders",
            sync_response.notes.len(), sync_response.folders.len());
        Ok(sync_response)
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
            "SELECT id, title, content, excerpt, markdown_cache, folder_id,
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
                folder_id: row.get(5)?,
                is_favorite: row.get(6)?,
                is_deleted: row.get(7)?,
                is_pinned: row.get(8)?,
                author: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                deleted_at: row.get(12)?,
                word_count: row.get(13)?,
                read_time_minutes: row.get(14)?,
                server_ver: row.get(15)?,
                is_dirty: row.get(16)?,
                last_synced_at: row.get(17)?,
            })
        })
        .map_err(|e| AppError::DatabaseError(format!("Failed to parse notes: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect notes: {}", e)))?;

        Ok(notes)
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
                is_deleted: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                deleted_at: row.get(9)?,
                server_ver: row.get(10)?,
                is_dirty: row.get(11)?,
                last_synced_at: row.get(12)?,
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
            "SELECT id, title, content, excerpt, markdown_cache, folder_id,
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
                folder_id: row.get(5)?,
                is_favorite: row.get(6)?,
                is_deleted: row.get(7)?,
                is_pinned: row.get(8)?,
                author: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                deleted_at: row.get(12)?,
                word_count: row.get(13)?,
                read_time_minutes: row.get(14)?,
                server_ver: row.get(15)?,
                is_dirty: row.get(16)?,
                last_synced_at: row.get(17)?,
            })
        }) {
            Ok(note) => Ok(Some(note)),
            Err(_) => Ok(None),
        }
    }

    /// 获取上次同步时间
    fn get_last_sync_at(&self) -> Result<Option<i64>> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare("SELECT last_sync_at FROM sync_state WHERE id = 1")?;

        match stmt.query_row([], |row| row.get(0)) {
            Ok(timestamp) => Ok(Some(timestamp)),
            Err(_) => Ok(None),
        }
    }

    /// 更新同步状态
    fn update_sync_state(&self, last_sync_at: i64) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let pending_count = self.count_pending()?;
        let conflict_count = 0; // TODO: 实际追踪冲突数

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
}
