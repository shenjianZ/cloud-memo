use crate::database::repositories::NoteRepository;
use crate::models::{Note, CreateNoteRequest, UpdateNoteRequest, MoveNotesRequest};
use crate::models::error::{Result, AppError};
use uuid::Uuid;

/// 笔记业务逻辑层
///
/// 处理笔记相关的业务逻辑，调用 Repository 进行数据操作
pub struct NoteService {
    repo: NoteRepository,
}

impl NoteService {
    /// 创建新的 NoteService 实例
    pub fn new(repo: NoteRepository) -> Self {
        Self { repo }
    }

    /// 创建笔记
    pub fn create_note(&self, req: CreateNoteRequest) -> Result<Note> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        let excerpt = self.generate_excerpt(&req.content);
        let word_count = self.count_words(&req.content);
        let read_time_minutes = self.calculate_read_time(word_count);

        let note = Note {
            id,
            title: req.title,
            content: req.content,
            excerpt,
            markdown_cache: None,
            folder_id: req.folder_id,
            is_favorite: false,
            is_deleted: false,
            is_pinned: false,
            author: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            word_count,
            read_time_minutes,
        };

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
            note.content = content;
            note.excerpt = self.generate_excerpt(&note.content);
            note.word_count = self.count_words(&note.content);
            note.read_time_minutes = self.calculate_read_time(note.word_count);
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

        self.repo.update(&note)
    }

    /// 删除笔记（软删除）
    pub fn delete_note(&self, id: &str) -> Result<()> {
        self.repo.soft_delete(id)
    }

    /// 获取所有笔记
    pub fn list_all_notes(&self) -> Result<Vec<Note>> {
        self.repo.find_all()
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

    // ===== 工具方法 =====

    /// 生成摘要
    fn generate_excerpt(&self, content: &str) -> Option<String> {
        let chars: Vec<char> = content.chars().collect();
        if chars.len() <= 200 {
            None
        } else {
            Some(chars[..200].iter().collect())
        }
    }

    /// 统计字数
    fn count_words(&self, content: &str) -> u32 {
        content.split_whitespace().count() as u32
    }

    /// 计算阅读时间（假设 200 字/分钟）
    fn calculate_read_time(&self, word_count: u32) -> u32 {
        (word_count / 200).max(1)
    }
}
