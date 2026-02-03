use crate::database::repositories::EditorSettingsRepository;
use crate::models::{EditorSettings, UpdateEditorSettingsRequest, error::{Result, AppError}};

pub struct EditorSettingsService {
    repo: EditorSettingsRepository,
}

impl EditorSettingsService {
    pub fn new(repo: EditorSettingsRepository) -> Self {
        Self { repo }
    }

    /// 获取编辑器设置
    pub fn get_settings(&self) -> Result<EditorSettings> {
        self.repo.find_by_id()?
            .ok_or(AppError::Internal("Editor settings not found".to_string()))
    }

    /// 更新编辑器设置
    pub fn update_settings(&self, req: UpdateEditorSettingsRequest) -> Result<EditorSettings> {
        self.repo.update(&req)
    }
}
