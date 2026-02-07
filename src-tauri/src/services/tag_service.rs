use crate::database::repositories::TagRepository;
use crate::models::{Tag, CreateTagRequest, UpdateTagRequest, NoteTagRequest, error::{Result, AppError}};

#[derive(Clone)]
pub struct TagService {
    repo: TagRepository,
}

impl TagService {
    pub fn new(repo: TagRepository) -> Self {
        Self { repo }
    }

    /// 获取所有标签
    pub fn get_all_tags(&self) -> Result<Vec<Tag>> {
        self.repo.find_all()
    }

    /// 根据 ID 获取标签
    pub fn get_tag(&self, id: &str) -> Result<Tag> {
        self.repo.find_by_id(id)?
            .ok_or(AppError::Internal(format!("Tag {} not found", id)))
    }

    /// 获取笔记的所有标签
    pub fn get_note_tags(&self, note_id: &str) -> Result<Vec<Tag>> {
        self.repo.find_by_note_id(note_id)
    }

    /// 创建标签
    pub fn create_tag(&self, req: CreateTagRequest) -> Result<Tag> {
        self.repo.create(&req)
    }

    /// 更新标签
    pub fn update_tag(&self, id: &str, req: UpdateTagRequest) -> Result<Tag> {
        self.repo.update(id, &req)
    }

    /// 删除标签
    pub fn delete_tag(&self, id: &str) -> Result<()> {
        self.repo.delete(id)
    }

    /// 为笔记添加标签
    pub fn add_tag_to_note(&self, req: NoteTagRequest) -> Result<()> {
        self.repo.add_tag_to_note(&req)
    }

    /// 从笔记移除标签
    pub fn remove_tag_from_note(&self, note_id: &str, tag_id: &str) -> Result<()> {
        self.repo.remove_tag_from_note(note_id, tag_id)
    }

    /// 设置笔记的标签（替换所有标签）
    pub fn set_note_tags(&self, note_id: &str, tag_ids: Vec<String>) -> Result<()> {
        self.repo.set_note_tags(note_id, &tag_ids)
    }

    /// 永久删除标签（硬删除）
    ///
    /// ## 删除行为
    ///
    /// - 从 `tags` 表中物理删除记录
    /// - 外键约束会自动删除 `note_tags` 中的关联记录
    pub fn permanently_delete_tag(&self, id: &str) -> Result<()> {
        self.repo.hard_delete(id)
    }

    /// 批量永久删除标签
    ///
    /// ## 返回
    ///
    /// 返回成功删除的标签数量
    pub fn permanently_delete_tags(&self, tag_ids: Vec<String>) -> Result<i64> {
        if tag_ids.is_empty() {
            return Ok(0);
        }
        self.repo.hard_delete_batch(&tag_ids)
    }

    /// 清理超过 30 天的软删除标签
    ///
    /// ## 返回
    ///
    /// 返回清理的标签数量
    pub fn purge_old_deleted_tags(&self) -> Result<i64> {
        const PURGE_AFTER_DAYS: i64 = 30;
        self.repo.purge_old_deleted_tags(PURGE_AFTER_DAYS)
    }
}
