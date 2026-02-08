use crate::database::repositories::NoteRepository;
use crate::database::repositories::FolderRepository;
use crate::models::{Note, Folder, CreateNoteRequest, UpdateNoteRequest, MoveNotesRequest};
use crate::models::error::{Result, AppError};

/// 笔记业务逻辑层
///
/// 处理笔记相关的业务逻辑，调用 Repository 进行数据操作
#[derive(Clone)]
pub struct NoteService {
    repo: NoteRepository,
    folder_repo: FolderRepository,  // 用于恢复笔记时创建/获取"已恢复笔记"文件夹
}

impl NoteService {
    /// 创建新的 NoteService 实例
    pub fn new(repo: NoteRepository, folder_repo: FolderRepository) -> Self {
        Self { repo, folder_repo }
    }

    /// 创建笔记
    pub fn create_note(&self, req: CreateNoteRequest) -> Result<Note> {
        let note = Note::new(req.title, req.content, req.folder_id);
        self.repo.create(&note)
    }

    /// 根据 ID 获取笔记
    pub fn get_note_by_id(&self, id: &str) -> Result<Note> {
        self.repo.find_by_id(id)?
            .ok_or(AppError::NoteNotFound(id.to_string()))
    }

    /// 更新笔记
    pub fn update_note(&self, req: UpdateNoteRequest) -> Result<Note> {
        let mut note = self.get_note_by_id(&req.id)?;

        if let Some(title) = req.title {
            note.title = title;
        }
        if let Some(content) = req.content {
            note.update_content(content);
        }
        if let Some(folder_id) = req.folder_id {
            note.folder_id = Some(folder_id);
        }
        if let Some(is_favorite) = req.is_favorite {
            note.is_favorite = is_favorite;
        }
        if let Some(is_pinned) = req.is_pinned {
            note.is_pinned = is_pinned;
        }
        if let Some(author) = req.author {
            note.author = Some(author);
        }

        note.updated_at = chrono::Utc::now().timestamp();
        // 云端同步：修改笔记时标记为需要同步
        note.is_dirty = true;

        self.repo.update(&note)
    }

    /// 删除笔记（软删除）
    pub fn delete_note(&self, id: &str) -> Result<()> {
        self.repo.soft_delete(id)
    }

    /// 恢复已删除的笔记到"已恢复笔记"文件夹
    ///
    /// ## 恢复行为
    ///
    /// - 自动获取或创建"已恢复笔记"系统文件夹
    /// - 将笔记从回收站恢复到该文件夹
    /// - 笔记状态：`is_deleted = false`
    /// - 文件夹位置：`folder_id = "已恢复笔记"文件夹 ID`
    ///
    /// ## 示例
    ///
    /// ```text
    /// 回收站：                    恢复后：
    /// 📄 项目笔记（已删除）      →  📁 已恢复笔记
    ///                              └─ 📄 项目笔记
    /// ```
    ///
    /// ## 注意事项
    ///
    /// - ✅ "已恢复笔记"文件夹会自动创建（如果不存在）
    /// - ✅ 用户可以手动整理恢复的笔记到其他文件夹
    /// - ⚠️ 笔记不会恢复到原始位置（使用方案 A 才能支持）
    pub fn restore_note(&self, id: &str) -> Result<Note> {
        // 获取或创建"已恢复笔记"文件夹
        let recovered_folder = self.get_or_create_recovered_folder()?;

        // 恢复笔记到该文件夹
        self.repo.restore(id, &recovered_folder.id)?;

        // 返回恢复后的笔记
        self.repo.find_by_id(id)?.ok_or(AppError::NotFound(format!("笔记 {} 恢复后未找到", id)))
    }

    /// 获取或创建"已恢复笔记"系统文件夹
    ///
    /// ## 文件夹属性
    ///
    /// - **名称**：`已恢复笔记`
    /// - **父级**：根目录（`parent_id = NULL`）
    /// - **图标**：📋 或 ♻️（前端可配置）
    /// - **颜色**：绿色（表示恢复）
    /// - **排序**：`sort_order = 9999`（永远在根目录最下边）
    ///
    /// ## 行为
    ///
    /// - 如果文件夹已存在且未删除，直接返回
    /// - 如果文件夹已存在但已删除，自动恢复后返回
    /// - 如果不存在，自动创建（sort_order = 9999）
    fn get_or_create_recovered_folder(&self) -> Result<Folder> {
        const RECOVERED_FOLDER_NAME: &str = "已恢复笔记";
        const RECOVERED_FOLDER_SORT_ORDER: i32 = 9999;  // 永远在最下边

        // 尝试查找已存在的"已恢复笔记"文件夹（包括已删除的）
        if let Some(existing) = self.folder_repo.find_by_name_include_deleted(RECOVERED_FOLDER_NAME)? {
            // 文件夹已存在，如果已删除则恢复
            if existing.is_deleted {
                log::info!("恢复已删除的文件夹 '{}'", RECOVERED_FOLDER_NAME);
                return self.folder_repo.restore(&existing.id);
            }
            return Ok(existing);
        }

        // 不存在则创建
        let mut folder = Folder::new(
            RECOVERED_FOLDER_NAME.to_string(),
            None,  // 根目录
            Some("#4CAF50".to_string()),  // 绿色
            Some("recycle".to_string()),  // 图标
            None,  // 工作空间 ID
        );
        folder.sort_order = RECOVERED_FOLDER_SORT_ORDER;  // 设置为最下边
        self.folder_repo.create(&folder)?;

        Ok(folder)
    }

    /// 批量恢复笔记到"已恢复笔记"文件夹
    ///
    /// ## 参数
    ///
    /// - `note_ids`: 要恢复的笔记 ID 列表
    ///
    /// ## 返回
    ///
    /// 返回成功恢复的笔记列表
    pub fn restore_notes(&self, note_ids: Vec<String>) -> Result<Vec<Note>> {
        let mut restored_notes = Vec::new();

        for note_id in note_ids {
            match self.restore_note(&note_id) {
                Ok(note) => restored_notes.push(note),
                Err(e) => {
                    log::warn!("Failed to restore note {}: {}", note_id, e);
                    // 继续恢复其他笔记，不中断整个操作
                }
            }
        }

        Ok(restored_notes)
    }

    /// 获取所有笔记
    pub fn list_all_notes(&self) -> Result<Vec<Note>> {
        self.repo.find_all()
    }

    /// 获取所有已删除的笔记（回收站）
    ///
    /// ## 返回
    ///
    /// 返回所有已删除的笔记列表，按删除时间倒序排列
    pub fn list_deleted_notes(&self) -> Result<Vec<Note>> {
        self.repo.find_deleted()
    }

    /// 搜索笔记
    pub fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        self.repo.search(query)
    }

    /// 批量移动笔记到文件夹
    pub fn move_notes_to_folder(&self, req: MoveNotesRequest) -> Result<Vec<Note>> {
        let mut moved_notes = Vec::new();

        for note_id in req.note_ids {
            let update_req = UpdateNoteRequest {
                id: note_id.clone(),
                title: None,
                content: None,
                folder_id: req.folder_id.clone(),
                is_favorite: None,
                is_pinned: None,
                author: None,
            };

            let note = self.update_note(update_req)?;
            moved_notes.push(note);
        }

        Ok(moved_notes)
    }

    /// 获取笔记数量（不包括软删除的笔记）
    ///
    /// ## 返回
    ///
    /// 返回 `is_deleted = 0` 的笔记总数
    pub fn count_notes(&self) -> Result<i64> {
        self.repo.count()
    }

    /// 永久删除笔记（硬删除）
    ///
    /// ## 行为
    ///
    /// - 物理删除笔记记录
    /// - FTS 索引自动同步删除
    /// - 笔记标签关联自动级联删除
    /// - **不会触发云端同步**（硬删除的数据不再同步）
    pub fn permanently_delete_note(&self, id: &str) -> Result<()> {
        self.repo.hard_delete(id)
    }

    /// 批量永久删除笔记
    ///
    /// ## 返回
    ///
    /// 返回成功删除的笔记数量
    pub fn permanently_delete_notes(&self, note_ids: Vec<String>) -> Result<i64> {
        if note_ids.is_empty() {
            return Ok(0);
        }
        self.repo.hard_delete_batch(&note_ids)
    }

    /// 清理超过 30 天的软删除笔记
    ///
    /// ## 返回
    ///
    /// 返回清理的笔记数量
    pub fn purge_old_deleted_notes(&self) -> Result<i64> {
        const PURGE_AFTER_DAYS: i64 = 30;
        self.repo.purge_old_deleted_notes(PURGE_AFTER_DAYS)
    }
}

