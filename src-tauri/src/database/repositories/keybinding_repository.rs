use crate::models::{KeybindingsData, get_default_keybindings};
use crate::models::error::{Result, AppError};
use std::fs;
use std::path::PathBuf;

/// 快捷键存储结构（内部格式）
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct KeybindingsStorage {
    version: String,
    keybindings: KeybindingsData,
}

/// 快捷键数据访问层
///
/// 负责快捷键配置的文件存储操作
pub struct KeybindingRepository {
    storage_path: PathBuf,
}

impl KeybindingRepository {
    /// 创建新的 KeybindingRepository 实例
    pub fn new(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    /// 加载快捷键配置
    pub fn load(&self) -> Result<KeybindingsData> {
        if !self.storage_path.exists() {
            // 如果文件不存在，返回默认配置
            log::info!("Keybindings file not found, using default configuration");
            return Ok(get_default_keybindings());
        }

        let content = fs::read_to_string(&self.storage_path)
            .map_err(|e| AppError::Internal(format!("Failed to read keybindings file: {}", e)))?;

        let storage: KeybindingsStorage = serde_json::from_str(&content)
            .map_err(|e| AppError::Internal(format!("Failed to parse keybindings file: {}", e)))?;

        log::debug!("Loaded {} keybindings from storage", storage.keybindings.keybindings.len());
        Ok(storage.keybindings)
    }

    /// 保存快捷键配置
    pub fn save(&self, data: &KeybindingsData) -> Result<()> {
        let storage = KeybindingsStorage {
            version: "1.0".to_string(),
            keybindings: data.clone(),
        };

        let content = serde_json::to_string_pretty(&storage)
            .map_err(|e| AppError::Internal(format!("Failed to serialize keybindings: {}", e)))?;

        fs::write(&self.storage_path, content)
            .map_err(|e| AppError::Internal(format!("Failed to write keybindings file: {}", e)))?;

        log::info!("Saved {} keybindings to storage", data.keybindings.len());
        Ok(())
    }

    /// 重置为默认配置
    pub fn reset(&self) -> Result<()> {
        let default_data = get_default_keybindings();
        self.save(&default_data)
    }
}
