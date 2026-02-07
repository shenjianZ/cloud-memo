use crate::models::{EditorSettings, UpdateEditorSettingsRequest, error::{Result, AppError}};
use crate::database::DbPool;
use r2d2_sqlite::rusqlite::{self as rusqlite, Row, params};

pub struct EditorSettingsRepository {
    pool: DbPool,
}

impl EditorSettingsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 获取编辑器设置（单例模式，id = 1）
    pub fn find_by_id(&self) -> Result<Option<EditorSettings>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, content_font_family, content_font_size, content_font_weight,
                    content_line_height, heading_font_family, heading_font_weight,
                    code_font_family, code_font_size, updated_at
             FROM editor_settings WHERE id = 1"
        )?;

        let result = stmt.query_row([], |row: &Row| {
            Ok(EditorSettings {
                id: row.get(0)?,
                content_font_family: row.get(1)?,
                content_font_size: row.get(2)?,
                content_font_weight: row.get(3)?,
                content_line_height: row.get(4)?,
                heading_font_family: row.get(5)?,
                heading_font_weight: row.get(6)?,
                code_font_family: row.get(7)?,
                code_font_size: row.get(8)?,
                updated_at: row.get(9)?,
            })
        });

        match result {
            Ok(settings) => Ok(Some(settings)),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // 如果不存在，创建默认设置
                let default_settings = EditorSettings::default();
                self.create(&default_settings)?;
                Ok(Some(default_settings))
            }
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 创建编辑器设置
    fn create(&self, settings: &EditorSettings) -> Result<EditorSettings> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR REPLACE INTO editor_settings (
                id, content_font_family, content_font_size, content_font_weight,
                content_line_height, heading_font_family, heading_font_weight,
                code_font_family, code_font_size, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                settings.id,
                &settings.content_font_family,
                settings.content_font_size,
                settings.content_font_weight,
                settings.content_line_height,
                &settings.heading_font_family,
                settings.heading_font_weight,
                &settings.code_font_family,
                settings.code_font_size,
                settings.updated_at,
            ],
        )?;

        Ok(settings.clone())
    }

    /// 更新编辑器设置
    pub fn update(&self, req: &UpdateEditorSettingsRequest) -> Result<EditorSettings> {
        // 获取当前设置
        let current = self.find_by_id()?.ok_or(AppError::Internal("Editor settings not found".to_string()))?;

        // 构建更新后的设置
        let updated = EditorSettings {
            id: 1,
            content_font_family: req.content_font_family.clone().unwrap_or(current.content_font_family),
            content_font_size: req.content_font_size.unwrap_or(current.content_font_size),
            content_font_weight: req.content_font_weight.unwrap_or(current.content_font_weight),
            content_line_height: req.content_line_height.unwrap_or(current.content_line_height),
            heading_font_family: req.heading_font_family.clone().unwrap_or(current.heading_font_family),
            heading_font_weight: req.heading_font_weight.unwrap_or(current.heading_font_weight),
            code_font_family: req.code_font_family.clone().unwrap_or(current.code_font_family),
            code_font_size: req.code_font_size.unwrap_or(current.code_font_size),
            updated_at: chrono::Utc::now().timestamp(),
        };

        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE editor_settings SET
                content_font_family = ?1,
                content_font_size = ?2,
                content_font_weight = ?3,
                content_line_height = ?4,
                heading_font_family = ?5,
                heading_font_weight = ?6,
                code_font_family = ?7,
                code_font_size = ?8,
                updated_at = ?9
            WHERE id = 1",
            params![
                &updated.content_font_family,
                updated.content_font_size,
                updated.content_font_weight,
                updated.content_line_height,
                &updated.heading_font_family,
                updated.heading_font_weight,
                &updated.code_font_family,
                updated.code_font_size,
                updated.updated_at,
            ],
        )?;

        Ok(updated)
    }
}
