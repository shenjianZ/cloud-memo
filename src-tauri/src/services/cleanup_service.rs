use crate::services::{NoteService, FolderService, TagService};
use crate::models::error::{Result, AppError};
use crate::database::DbPool;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 自动清理服务
///
/// 定期清理超过 30 天的软删除数据
///
/// ## 执行策略
///
/// - **应用启动时检查**：每次应用启动时检查是否需要清理
/// - **24小时执行一次**：距离上次清理超过 24 小时则执行
/// - **持久化记录**：将上次清理时间保存到数据库，避免重复执行
#[derive(Clone)]
pub struct CleanupService {
    note_service: NoteService,
    folder_service: FolderService,
    tag_service: TagService,
    pool: DbPool,
    is_running: Arc<Mutex<bool>>,
}

impl CleanupService {
    /// 创建新的 CleanupService 实例
    pub fn new(
        note_service: NoteService,
        folder_service: FolderService,
        tag_service: TagService,
        pool: DbPool,
    ) -> Self {
        Self {
            note_service,
            folder_service,
            tag_service,
            pool,
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// 应用启动时执行清理检查
    ///
    /// ## 检查逻辑
    ///
    /// 1. 从数据库读取上次检查时间
    /// 2. 如果距离上次检查超过 24 小时，执行清理
    /// 3. 无论是否实际删除数据，都更新数据库中的检查时间
    pub async fn startup_cleanup(&self) -> Result<CleanupStats> {
        log::info!("[CleanupService] 应用启动，检查是否需要执行清理任务");

        let mut is_running = self.is_running.lock().await;
        if *is_running {
            log::info!("[CleanupService] 清理任务正在运行，跳过");
            return Ok(CleanupStats { notes: 0, folders: 0, tags: 0 });
        }
        *is_running = true;
        drop(is_running);

        // 检查是否需要执行清理（距离上次检查超过 24 小时）
        let last_cleanup_time = self.get_last_cleanup_time()?;
        let now = chrono::Utc::now().timestamp();
        const CLEANUP_INTERVAL: i64 = 24 * 3600; // 24 小时

        if last_cleanup_time + CLEANUP_INTERVAL > now {
            let hours_since_cleanup = (now - last_cleanup_time) / 3600;
            log::info!(
                "[CleanupService] 距离上次检查仅 {} 小时，跳过（间隔：24小时）",
                hours_since_cleanup
            );
            *self.is_running.lock().await = false;
            return Ok(CleanupStats { notes: 0, folders: 0, tags: 0 });
        }

        // 执行清理（无论是否实际删除数据，都视为一次成功的检查）
        log::info!("[CleanupService] 开始执行清理任务");
        let stats = Self::do_cleanup(&self.note_service, &self.folder_service, &self.tag_service)?;

        // 更新检查时间（无论实际清理了多少数据）
        self.update_last_cleanup_time(now)?;

        log::info!(
            "[CleanupService] 清理检查完成: notes={}, folders={}, tags={}",
            stats.notes,
            stats.folders,
            stats.tags
        );

        *self.is_running.lock().await = false;
        Ok(stats)
    }

    /// 获取上次清理时间
    ///
    /// 从 `settings` 表中读取 `last_cleanup_time` 键的值
    /// 如果不存在，返回 0（表示从未清理过）
    fn get_last_cleanup_time(&self) -> Result<i64> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT value FROM settings WHERE key = 'last_cleanup_time'"
        ).map_err(|e| AppError::DatabaseError(format!("查询上次清理时间失败: {}", e)))?;

        match stmt.query_row([], |row| {
            let value: String = row.get(0)?;
            value.parse::<i64>().map_err(|e| {
                r2d2_sqlite::rusqlite::Error::ToSqlConversionFailure(Box::new(e))
            })
        }) {
            Ok(timestamp) => Ok(timestamp),
            Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => {
                log::debug!("[CleanupService] 未找到上次清理时间记录，返回 0");
                Ok(0)
            }
            Err(e) => Err(AppError::DatabaseError(format!("解析上次清理时间失败: {}", e)))
        }
    }

    /// 更新上次清理时间
    ///
    /// 将当前时间写入 `settings` 表的 `last_cleanup_time` 键
    fn update_last_cleanup_time(&self, timestamp: i64) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let now = chrono::Utc::now().timestamp().to_string();

        // 使用 INSERT OR REPLACE 来插入或更新记录
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, created_at, updated_at)
             VALUES ('last_cleanup_time', :value, :now, :now)",
            &[(":value", &timestamp.to_string()), (":now", &now)]
        ).map_err(|e| AppError::DatabaseError(format!("更新上次清理时间失败: {}", e)))?;

        log::debug!("[CleanupService] 更新清理时间: {}", timestamp);
        Ok(())
    }

    /// 执行清理的核心逻辑（私有方法）
    fn do_cleanup(
        note_service: &NoteService,
        folder_service: &FolderService,
        tag_service: &TagService,
    ) -> Result<CleanupStats> {
        let notes = note_service.purge_old_deleted_notes()?;
        let folders = folder_service.purge_old_deleted_folders()?;
        let tags = tag_service.purge_old_deleted_tags()?;

        Ok(CleanupStats { notes, folders, tags })
    }
}

/// 清理统计
#[derive(Debug, Clone)]
pub struct CleanupStats {
    pub notes: i64,
    pub folders: i64,
    pub tags: i64,
}
