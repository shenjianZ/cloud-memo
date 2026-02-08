// 类型通过函数返回值间接使用，编译器有时检测不到
#[allow(unused_imports)]
use crate::models::{
    Note, Folder, Tag, NoteSnapshot, SyncRequest, SyncResponse, SyncReport,
    NoteTagRelation, ConflictStrategy
};
use crate::models::error::{Result, AppError};
use crate::services::SyncService;
use crate::database::repositories::tag_repository::TagRepository;
use crate::database::repositories::snapshot_repository::SnapshotRepository;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2_sqlite::rusqlite;

/// 单个数据同步服务
///
/// 用于同步单个笔记、标签或快照，并自动收集关联数据
#[derive(Clone)]
pub struct SingleSyncService {
    sync_service: SyncService,
    tag_repository: TagRepository,
    snapshot_repository: SnapshotRepository,
}

impl SingleSyncService {
    /// 创建新的 SingleSyncService 实例
    pub fn new(pool: Pool<SqliteConnectionManager>, sync_service: SyncService) -> Self {
        Self {
            sync_service,
            tag_repository: TagRepository::new(pool.clone()),
            snapshot_repository: SnapshotRepository::new(pool),
        }
    }

    /// 同步单个笔记及其关联数据（标签、快照）
    pub async fn sync_single_note(&self, note_id: &str) -> Result<SyncReport> {
        log::info!("[SingleSync] 同步单个笔记: {}", note_id);

        // 1. 尝试获取笔记数据（无论是否是脏数据）
        let note_opt = self.get_note_by_id(note_id)?;

        // 2. 获取该笔记的所有脏标签
        let tags = self.get_dirty_tags_by_note_id(note_id)?;

        // 3. 获取该笔记的所有脏快照
        let snapshots = self.get_dirty_snapshots_by_note_id(note_id)?;

        // 4. 获取笔记-标签关联（只获取脏关联）
        let note_tags = self.get_dirty_note_tag_relations(note_id)?;

        // 记录是否有数据要推送（在移动之前）
        let has_data_to_push = note_opt.is_some() || !tags.is_empty() || !snapshots.is_empty();
        let pushed_note_count = if note_opt.is_some() { 1 } else { 0 };

        // 5. 构建同步请求
        // 如果笔记存在且是脏数据，就推送；否则只拉取服务器更新
        let request = SyncRequest {
            workspaces: None, // 不同步工作空间
            notes: note_opt.map(|n| vec![n.into()]),
            folders: None, // 不同步文件夹
            tags: Some(tags.into_iter().map(|t| t.into()).collect()),
            snapshots: Some(snapshots.into_iter().map(|s| s.into()).collect()),
            note_tags: if note_tags.is_empty() { None } else { Some(note_tags.into_iter().map(|nt| nt.into()).collect()) },
            last_sync_at: self.sync_service.get_last_sync_time()?,
            conflict_resolution: ConflictStrategy::default(),
            device_id: None, // 在 send_sync_request 中设置
        };

        // 6. 发送同步请求
        let response = self.sync_service.send_sync_request(&request).await?;

        // 7. 应用服务器响应
        self.sync_service.apply_sync_response(&response)?;

        // 8. 清理脏标记（如果有推送数据）
        if has_data_to_push {
            self.sync_service.clear_dirty_markers(&request, response.last_sync_at)?;
        }

        // 9. 更新同步状态
        self.sync_service.update_sync_state(response.last_sync_at, response.conflicts.len() as i32)?;

        let report = SyncReport {
            success: response.status != "error",
            // ✅ 使用服务器确认的推送统计
            pushed_workspaces: response.pushed_workspaces,
            pushed_notes: response.pushed_notes,
            pushed_folders: response.pushed_folders,
            pushed_tags: response.pushed_tags,
            pushed_snapshots: response.pushed_snapshots,
            pushed_note_tags: response.pushed_note_tags,
            // ✅ 使用服务器计算的拉取统计
            pulled_workspaces: response.pulled_workspaces,
            pulled_notes: response.pulled_notes,
            pulled_folders: response.pulled_folders,
            pulled_tags: response.pulled_tags,
            pulled_snapshots: response.pulled_snapshots,
            pulled_note_tags: response.pulled_note_tags,
            // 删除的数据统计
            deleted_workspaces: response.deleted_workspace_ids.len(),
            deleted_notes: response.deleted_note_ids.len(),
            deleted_folders: response.deleted_folder_ids.len(),
            deleted_tags: response.deleted_tag_ids.len(),
            // 冲突和错误
            conflict_count: response.conflicts.len(),
            error: if response.status == "error" {
                Some("Single note sync failed".to_string())
            } else {
                None
            },
            // 兼容旧版本
            pushed_count: None,
            pulled_count: None,
        };

        log::info!("[SingleSync] 笔记同步完成: pushed_total={}, pulled_total={}, pushed_notes={}, pushed_folders={}, pushed_tags={}, pushed_snapshots={}, pushed_note_tags={}, pulled_notes={}, pulled_folders={}, pulled_tags={}, pulled_snapshots={}, pulled_note_tags={}, conflicts={}",
            response.pushed_total, response.pulled_total,
            report.pushed_notes, report.pushed_folders, report.pushed_tags, report.pushed_snapshots, report.pushed_note_tags,
            report.pulled_notes, report.pulled_folders, report.pulled_tags, report.pulled_snapshots, report.pulled_note_tags,
            report.conflict_count);

        Ok(report)
    }

    /// 同步单个标签
    pub async fn sync_single_tag(&self, tag_id: &str) -> Result<SyncReport> {
        log::info!("[SingleSync] 同步单个标签: {}", tag_id);

        // 1. 获取标签数据
        let tag = self.tag_repository.find_by_id(tag_id)?
            .ok_or_else(|| AppError::Internal(format!("标签 {} 未找到", tag_id)))?;

        // 2. 构建同步请求
        let request = SyncRequest {
            workspaces: None,
            notes: None,
            folders: None,
            tags: Some(vec![tag.into()]),
            snapshots: None,
            note_tags: None,
            last_sync_at: self.sync_service.get_last_sync_time()?,
            conflict_resolution: ConflictStrategy::default(),
            device_id: None,
        };

        // 3. 发送同步请求
        let response = self.sync_service.send_sync_request(&request).await?;

        // 4. 应用服务器响应
        self.sync_service.apply_sync_response(&response)?;

        // 5. 清理脏标记
        self.sync_service.clear_dirty_markers(&request, response.last_sync_at)?;

        // 6. 更新同步状态
        self.sync_service.update_sync_state(response.last_sync_at, response.conflicts.len() as i32)?;

        Ok(SyncReport {
            success: response.status != "error",
            pushed_workspaces: response.pushed_workspaces,
            pushed_notes: response.pushed_notes,
            pushed_folders: response.pushed_folders,
            pushed_tags: response.pushed_tags,
            pushed_snapshots: response.pushed_snapshots,
            pushed_note_tags: response.pushed_note_tags,
            pulled_workspaces: response.pulled_workspaces,
            pulled_notes: response.pulled_notes,
            pulled_folders: response.pulled_folders,
            pulled_tags: response.pulled_tags,
            pulled_snapshots: response.pulled_snapshots,
            pulled_note_tags: response.pulled_note_tags,
            deleted_workspaces: response.deleted_workspace_ids.len(),
            deleted_notes: response.deleted_note_ids.len(),
            deleted_folders: response.deleted_folder_ids.len(),
            deleted_tags: response.deleted_tag_ids.len(),
            conflict_count: response.conflicts.len(),
            error: None,
            pushed_count: None,
            pulled_count: None,
        })
    }

    /// 同步单个快照
    pub async fn sync_single_snapshot(&self, snapshot_id: &str) -> Result<SyncReport> {
        log::info!("[SingleSync] 同步单个快照: {}", snapshot_id);

        // 1. 获取快照数据
        let snapshot = self.snapshot_repository.find_by_id(snapshot_id)?
            .ok_or_else(|| AppError::Internal(format!("快照 {} 未找到", snapshot_id)))?;

        // 2. 构建同步请求
        let request = SyncRequest {
            workspaces: None,
            notes: None,
            folders: None,
            tags: None,
            snapshots: Some(vec![snapshot.into()]),
            note_tags: None,
            last_sync_at: self.sync_service.get_last_sync_time()?,
            conflict_resolution: ConflictStrategy::default(),
            device_id: None,
        };

        // 3. 发送同步请求
        let response = self.sync_service.send_sync_request(&request).await?;

        // 4. 应用服务器响应
        self.sync_service.apply_sync_response(&response)?;

        // 5. 清理脏标记
        self.sync_service.clear_dirty_markers(&request, response.last_sync_at)?;

        // 6. 更新同步状态
        self.sync_service.update_sync_state(response.last_sync_at, response.conflicts.len() as i32)?;

        Ok(SyncReport {
            success: response.status != "error",
            pushed_workspaces: response.pushed_workspaces,
            pushed_notes: response.pushed_notes,
            pushed_folders: response.pushed_folders,
            pushed_tags: response.pushed_tags,
            pushed_snapshots: response.pushed_snapshots,
            pushed_note_tags: response.pushed_note_tags,
            pulled_workspaces: response.pulled_workspaces,
            pulled_notes: response.pulled_notes,
            pulled_folders: response.pulled_folders,
            pulled_tags: response.pulled_tags,
            pulled_snapshots: response.pulled_snapshots,
            pulled_note_tags: response.pulled_note_tags,
            deleted_workspaces: response.deleted_workspace_ids.len(),
            deleted_notes: response.deleted_note_ids.len(),
            deleted_folders: response.deleted_folder_ids.len(),
            deleted_tags: response.deleted_tag_ids.len(),
            conflict_count: response.conflicts.len(),
            error: None,
            pushed_count: None,
            pulled_count: None,
        })
    }

    /// 同步单个文件夹及其包含的所有笔记（含标签和快照）
    /// 递归同步所有子文件夹和它们的笔记
    pub async fn sync_single_folder(&self, folder_id: &str) -> Result<SyncReport> {
        log::info!("[SingleSync] 同步单个文件夹: {}", folder_id);

        // 1. 递归获取所有子文件夹
        let all_folder_ids = self.get_all_sub_folder_ids(folder_id)?;
        log::info!("[SingleSync] 文件夹及其子文件夹共 {} 个", all_folder_ids.len());

        // 2. 一次性获取所有文件夹数据（优化：避免重复查询）
        let all_folders = self.get_folders_by_ids(&all_folder_ids)?;

        // 3. 获取所有这些文件夹下的笔记
        let mut all_notes = Vec::new();
        for fid in &all_folder_ids {
            let notes = self.get_notes_by_folder(fid)?;
            all_notes.extend(notes);
        }

        log::info!("[SingleSync] 文件夹及其子文件夹共包含 {} 个笔记", all_notes.len());

        // 4. 收集这些笔记的所有标签和快照
        let mut all_tags = std::collections::HashMap::new();
        let mut all_snapshots = Vec::new();
        let mut all_note_tags = Vec::new();

        for note in &all_notes {
            // 获取笔记的标签
            let tags = self.tag_repository.find_by_note_id(&note.id)?;
            for tag in &tags {
                all_tags.insert(tag.id.clone(), tag.clone());
            }

            // 获取笔记-标签关联（使用真实的 created_at）
            let note_tag_relations = self.tag_repository.find_note_tag_relations(&note.id)?;
            all_note_tags.extend(note_tag_relations);

            // 获取笔记的快照
            let snapshots = self.snapshot_repository.find_by_note_id(&note.id)?;
            all_snapshots.extend(snapshots);
        }

        let tags_vec: Vec<Tag> = all_tags.into_values().collect();

        // 5. 构建同步请求
        let notes_count = all_notes.len();
        let folders_count = all_folders.len();
        let tags_count = tags_vec.len();
        let snapshots_count = all_snapshots.len();

        let request = SyncRequest {
            workspaces: None,
            conflict_resolution: ConflictStrategy::default(),
            device_id: None,
            notes: if all_notes.is_empty() { None } else { Some(all_notes.into_iter().map(|n| n.into()).collect()) },
            folders: if all_folders.is_empty() { None } else { Some(all_folders.into_iter().map(|f| f.into()).collect()) },
            tags: if tags_vec.is_empty() { None } else { Some(tags_vec.into_iter().map(|t| t.into()).collect()) },
            snapshots: if all_snapshots.is_empty() { None } else { Some(all_snapshots.into_iter().map(|s| s.into()).collect()) },
            note_tags: if all_note_tags.is_empty() { None } else { Some(all_note_tags.into_iter().map(|nt| nt.into()).collect()) },
            last_sync_at: self.sync_service.get_last_sync_time()?,
        };

        // 6. 发送同步请求
        let response = self.sync_service.send_sync_request(&request).await?;

        // 7. 应用服务器响应
        self.sync_service.apply_sync_response(&response)?;

        // 8. 清理脏标记
        self.sync_service.clear_dirty_markers(&request, response.last_sync_at)?;

        // 9. 更新同步状态
        self.sync_service.update_sync_state(response.last_sync_at, response.conflicts.len() as i32)?;

        let report = SyncReport {
            success: response.status != "error",
            // ✅ 使用服务器确认的推送统计
            pushed_workspaces: response.pushed_workspaces,
            pushed_notes: response.pushed_notes,
            pushed_folders: response.pushed_folders,
            pushed_tags: response.pushed_tags,
            pushed_snapshots: response.pushed_snapshots,
            pushed_note_tags: response.pushed_note_tags,
            // ✅ 使用服务器计算的拉取统计
            pulled_workspaces: response.pulled_workspaces,
            pulled_notes: response.pulled_notes,
            pulled_folders: response.pulled_folders,
            pulled_tags: response.pulled_tags,
            pulled_snapshots: response.pulled_snapshots,
            pulled_note_tags: response.pulled_note_tags,
            // 删除的数据统计
            deleted_workspaces: response.deleted_workspace_ids.len(),
            deleted_notes: response.deleted_note_ids.len(),
            deleted_folders: response.deleted_folder_ids.len(),
            deleted_tags: response.deleted_tag_ids.len(),
            // 冲突和错误
            conflict_count: response.conflicts.len(),
            error: if response.status == "error" {
                Some("Folder sync failed".to_string())
            } else {
                None
            },
            // 兼容旧版本
            pushed_count: None,
            pulled_count: None,
        };

        log::info!("[SingleSync] 文件夹同步完成: folder={}, pushed_total={}, pulled_total={}, pushed_notes={}, pushed_folders={}, pushed_tags={}, pushed_snapshots={}, pushed_note_tags={}, pulled_notes={}, pulled_folders={}, pulled_tags={}, pulled_snapshots={}, pulled_note_tags={}, conflicts={}",
            folder_id, response.pushed_total, response.pulled_total,
            report.pushed_notes, report.pushed_folders, report.pushed_tags, report.pushed_snapshots, report.pushed_note_tags,
            report.pulled_notes, report.pulled_folders, report.pulled_tags, report.pulled_snapshots, report.pulled_note_tags,
            report.conflict_count);

        Ok(report)
    }

    // ===== 私有辅助方法 =====

    /// 递归获取所有子文件夹ID（包括自己）
    fn get_all_sub_folder_ids(&self, folder_id: &str) -> Result<Vec<String>> {
        self.get_all_sub_folder_ids_with_visited(folder_id, &mut std::collections::HashSet::new())
    }

    /// 递归获取所有子文件夹ID（带循环引用检测）
    fn get_all_sub_folder_ids_with_visited(
        &self,
        folder_id: &str,
        visited: &mut std::collections::HashSet<String>
    ) -> Result<Vec<String>> {
        use r2d2_sqlite::rusqlite::params;

        // 检测循环引用
        if visited.contains(folder_id) {
            log::warn!("[SingleSync] 检测到循环引用: {}", folder_id);
            return Ok(Vec::new());
        }

        visited.insert(folder_id.to_string());

        let pool = self.sync_service.get_pool();
        let conn = pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 获取直接子文件夹
        let mut stmt = conn.prepare(
            "SELECT id FROM folders WHERE parent_id = ?1"
        ).map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let child_ids = stmt.query_map(params![folder_id], |row| {
            Ok(row.get::<_, String>(0)?)
        }).map_err(|e| AppError::DatabaseError(format!("查询文件夹失败: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("收集文件夹 ID 失败: {}", e)))?;

        // 递归获取所有子文件夹的子文件夹
        let mut all_ids = Vec::new();
        all_ids.push(folder_id.to_string());
        for child_id in child_ids {
            let sub_ids = self.get_all_sub_folder_ids_with_visited(&child_id, visited)?;
            all_ids.extend(sub_ids);
        }

        // 允许其他路径再次访问（如果有必要）
        visited.remove(folder_id);

        Ok(all_ids)
    }

    /// 根据 ID 获取文件夹
    fn get_folder_by_id(&self, folder_id: &str) -> Result<Folder> {
        use r2d2_sqlite::rusqlite::params;

        let pool = self.sync_service.get_pool();
        let conn = pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, icon, color, sort_order,
                    created_at, updated_at,
                    server_ver, is_dirty, last_synced_at
             FROM folders
             WHERE id = ?1"
        ).map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let folder = stmt.query_row(params![folder_id], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                workspace_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                server_ver: row.get(9)?,
                is_dirty: row.get(10)?,
                last_synced_at: row.get(11)?,
                deleted_at: None,
                is_deleted: false,
            })
        }).map_err(|e| AppError::DatabaseError(format!("文件夹 {} 未找到: {}", folder_id, e)))?;

        Ok(folder)
    }

    /// 根据ID列表批量获取脏文件夹（is_dirty = 1）（优化：避免重复查询）
    fn get_folders_by_ids(&self, folder_ids: &[String]) -> Result<Vec<Folder>> {
        if folder_ids.is_empty() {
            return Ok(Vec::new());
        }

        let pool = self.sync_service.get_pool();
        let conn = pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 使用 IN 查询一次性获取所有文件夹（只返回脏数据）
        let placeholders = (0..folder_ids.len())
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            "SELECT id, name, parent_id, icon, color, sort_order,
                    created_at, updated_at,
                    server_ver, is_dirty, last_synced_at
             FROM folders
             WHERE id IN ({}) AND is_dirty = 1",  // ✅ 只返回脏数据
            placeholders
        );

        let mut stmt = conn.prepare(&query)
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        // 构建参数
        let params: Vec<&dyn rusqlite::ToSql> = folder_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        let folders = stmt.query_map(params.as_slice(), |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                workspace_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                server_ver: row.get(9)?,
                is_dirty: row.get(10)?,
                last_synced_at: row.get(11)?,
                deleted_at: None,
                is_deleted: false,
            })
        }).map_err(|e| AppError::DatabaseError(format!("查询文件夹失败: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("收集文件夹数据失败: {}", e)))?;

        Ok(folders)
    }

    /// 获取文件夹下的所有脏笔记（is_dirty = 1）
    fn get_notes_by_folder(&self, folder_id: &str) -> Result<Vec<Note>> {
        use r2d2_sqlite::rusqlite::params;

        let pool = self.sync_service.get_pool();
        let conn = pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, title, content, excerpt, markdown_cache, workspace_id, folder_id,
                    is_favorite, is_deleted, is_pinned, author,
                    created_at, updated_at, deleted_at, word_count, read_time_minutes,
                    server_ver, is_dirty, last_synced_at
             FROM notes
             WHERE folder_id = ?1 AND is_deleted = 0 AND is_dirty = 1"  // ✅ 只返回脏数据
        ).map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let notes = stmt.query_map(params![folder_id], |row| {
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
        }).map_err(|e| AppError::DatabaseError(format!("解析笔记失败: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("收集笔记失败: {}", e)))?;

        Ok(notes)
    }

    // ===== 私有辅助方法 =====

    /// 根据 ID 获取笔记（无论是否是脏数据）
    /// 只返回脏笔记：如果笔记是脏数据，返回 Some(note)；否则返回 None
    fn get_note_by_id(&self, note_id: &str) -> Result<Option<Note>> {
        use r2d2_sqlite::rusqlite::params;

        let pool = self.sync_service.get_pool();
        let conn = pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, title, content, excerpt, markdown_cache, workspace_id, folder_id,
                    is_favorite, is_deleted, is_pinned, author,
                    created_at, updated_at, deleted_at, word_count, read_time_minutes,
                    server_ver, is_dirty, last_synced_at
             FROM notes
             WHERE id = ?1 AND is_dirty = 1"  // ✅ 只返回脏数据
        ).map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        match stmt.query_row(params![note_id], |row| {
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
            Ok(note) => {
                log::info!("[SingleSync] 获取脏笔记: id={}, title={}, server_ver={}, is_dirty=1",
                    note.id, note.title, note.server_ver);
                Ok(Some(note))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                log::info!("[SingleSync] 笔记不存在或不是脏数据: id={}, is_dirty=0", note_id);
                Ok(None)
            }
            Err(e) => Err(AppError::DatabaseError(format!("查询笔记失败: {}", e)))
        }
    }

    /// 根据 ID 获取脏笔记（is_dirty = 1）
    /// 如果笔记不存在或不是脏数据，返回错误
    fn get_dirty_note_by_id(&self, note_id: &str) -> Result<Note> {
        use r2d2_sqlite::rusqlite::params;

        let pool = self.sync_service.get_pool();
        let conn = pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, title, content, excerpt, markdown_cache, workspace_id, folder_id,
                    is_favorite, is_deleted, is_pinned, author,
                    created_at, updated_at, deleted_at, word_count, read_time_minutes,
                    server_ver, is_dirty, last_synced_at
             FROM notes
             WHERE id = ?1 AND is_dirty = 1"  // ✅ 只返回脏数据
        ).map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let note = stmt.query_row(params![note_id], |row| {
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
        }).map_err(|e| AppError::DatabaseError(format!("笔记 {} 未找到或不是脏数据: {}", note_id, e)))?;

        log::info!("[SingleSync] 获取脏笔记: id={}, title={}, server_ver={}, is_dirty=1",
            note.id, note.title, note.server_ver);

        Ok(note)
    }

    /// 获取笔记的所有脏标签（is_dirty = 1）
    fn get_dirty_tags_by_note_id(&self, note_id: &str) -> Result<Vec<Tag>> {
        use r2d2_sqlite::rusqlite::params;

        let pool = self.sync_service.get_pool();
        let conn = pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at, t.updated_at,
                    t.deleted_at, t.is_deleted, t.server_ver, t.is_dirty, t.last_synced_at
             FROM tags t
             INNER JOIN note_tags nt ON t.id = nt.tag_id
             WHERE nt.note_id = ?1 AND t.is_dirty = 1 AND nt.is_deleted = 0"  // ✅ 只返回脏标签
        ).map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let tags = stmt.query_map(params![note_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                workspace_id: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                deleted_at: row.get(6)?,
                is_deleted: row.get(7)?,  // ✅ 添加 is_deleted 字段
                server_ver: row.get(8)?,
                is_dirty: row.get(9)?,
                last_synced_at: row.get(10)?,
            })
        }).map_err(|e| AppError::DatabaseError(format!("查询标签失败: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("收集标签失败: {}", e)))?;

        log::info!("[SingleSync] 获取笔记 {} 的脏标签: {} 个", note_id, tags.len());

        Ok(tags)
    }

    /// 获取笔记的所有脏快照（is_dirty = 1）
    fn get_dirty_snapshots_by_note_id(&self, note_id: &str) -> Result<Vec<NoteSnapshot>> {
        use r2d2_sqlite::rusqlite::params;

        let pool = self.sync_service.get_pool();
        let conn = pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, note_id, title, content, snapshot_name,
                    created_at, workspace_id, server_ver, is_dirty, last_synced_at
             FROM note_snapshots
             WHERE note_id = ?1 AND is_dirty = 1"  // ✅ 只返回脏快照
        ).map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let snapshots = stmt.query_map(params![note_id], |row| {
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
        }).map_err(|e| AppError::DatabaseError(format!("查询快照失败: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("收集快照失败: {}", e)))?;

        log::info!("[SingleSync] 获取笔记 {} 的脏快照: {} 个", note_id, snapshots.len());

        Ok(snapshots)
    }

    /// 获取笔记的所有脏笔记-标签关联（is_deleted = 0）
    /// 注意：客户端 note_tags 表没有 user_id 字段，使用空字符串代替
    fn get_dirty_note_tag_relations(&self, note_id: &str) -> Result<Vec<NoteTagRelation>> {
        use r2d2_sqlite::rusqlite::params;

        let pool = self.sync_service.get_pool();
        let conn = pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 客户端 note_tags 表没有 user_id 字段，服务器有
        let mut stmt = conn.prepare(
            "SELECT note_id, tag_id, created_at, is_deleted, deleted_at
             FROM note_tags
             WHERE note_id = ?1 AND is_deleted = 0"  // ✅ 只返回未删除的关联
        ).map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let relations = stmt.query_map(params![note_id], |row| {
            Ok(NoteTagRelation {
                note_id: row.get(0)?,
                tag_id: row.get(1)?,
                user_id: String::new(),  // ✅ 客户端不需要 user_id，使用空字符串
                created_at: row.get(2)?,
                is_deleted: row.get(3)?,
                deleted_at: row.get(4)?,
            })
        }).map_err(|e| AppError::DatabaseError(format!("查询笔记-标签关联失败: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("收集笔记-标签关联失败: {}", e)))?;

        log::info!("[SingleSync] 获取笔记 {} 的笔记-标签关联: {} 个", note_id, relations.len());

        Ok(relations)
    }
}
