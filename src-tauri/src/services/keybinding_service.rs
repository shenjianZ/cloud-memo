use crate::database::repositories::KeybindingRepository;
use crate::models::{KeybindingsData, KeybindingPreset};
use crate::models::error::{Result, AppError};
use std::collections::HashMap;

/// 快捷键业务逻辑层
///
/// 处理快捷键配置相关的业务逻辑
pub struct KeybindingService {
    repo: KeybindingRepository,
}

impl KeybindingService {
    /// 创建新的 KeybindingService 实例
    pub fn new(repo: KeybindingRepository) -> Self {
        Self { repo }
    }

    /// 加载快捷键配置
    pub fn load_keybindings(&self) -> Result<KeybindingsData> {
        self.repo.load()
    }

    /// 保存快捷键配置
    pub fn save_keybindings(&self, keybindings: HashMap<String, crate::models::KeyCombination>, presets: Vec<KeybindingPreset>) -> Result<()> {
        let data = KeybindingsData {
            keybindings,
            presets,
        };
        self.repo.save(&data)
    }

    /// 导入快捷键配置
    pub fn import_keybindings(&self, json_string: &str) -> Result<()> {
        // 解析导入的 JSON
        let value: serde_json::Value = serde_json::from_str(json_string)
            .map_err(|e| AppError::InvalidInput(format!("解析 JSON 失败: {}", e)))?;

        // 验证版本
        let version = value.get("version")
            .and_then(|v| v.as_str())
            .ok_or(AppError::InvalidInput("缺少版本字段".to_string()))?;

        if version != "1.0" {
            return Err(AppError::InvalidInput(format!("不支持的版本: {}", version)));
        }

        // 解析 keybindings
        let keybindings: HashMap<String, crate::models::KeyCombination> =
            serde_json::from_value(
                value.get("keybindings")
                    .ok_or(AppError::InvalidInput("缺少快捷键字段".to_string()))?
                    .clone()
            ).map_err(|e| AppError::InvalidInput(format!("解析快捷键失败: {}", e)))?;

        // 解析 presets（可选）
        let presets = if let Some(presets_value) = value.get("presets") {
            serde_json::from_value(presets_value.clone())
                .map_err(|e| AppError::InvalidInput(format!("解析预设失败: {}", e)))?
        } else {
            vec![]
        };

        self.save_keybindings(keybindings, presets)?;
        log::info!("快捷键导入成功");
        Ok(())
    }

    /// 重置为默认配置
    pub fn reset_keybindings(&self) -> Result<()> {
        self.repo.reset()?;
        log::info!("Keybindings reset to default configuration");
        Ok(())
    }
}
