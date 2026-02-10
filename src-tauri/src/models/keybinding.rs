use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 快捷键组合
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KeyCombination {
    pub ctrl: bool,  // 是否按下 Ctrl 键
    pub alt: bool,  // 是否按下 Alt 键
    pub shift: bool,  // 是否按下 Shift 键
    pub meta: bool,  // 是否按下 Meta 键（Windows 键或 Command 键）
    pub key: String,  // 按键代码（如 "KeyA", "Enter", "Space"）
}

/// 快捷键预设
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KeybindingPreset {
    pub id: String,  // 预设 ID
    pub name: String,  // 预设名称
    pub description: String,  // 预设描述
    pub keybindings: HashMap<String, KeyCombination>,  // 快捷键映射（命令ID -> 快捷键组合）
}

/// 快捷键数据
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KeybindingsData {
    pub keybindings: HashMap<String, KeyCombination>,  // 当前快捷键映射
    pub presets: Vec<KeybindingPreset>,  // 可用的预设列表
}

/// 获取默认快捷键配置
pub fn get_default_keybindings() -> KeybindingsData {
    let mut keybindings = HashMap::new();

    // 全局快捷键
    keybindings.insert("global.newNote".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "KeyN".to_string(),
    });
    keybindings.insert("global.openSearch".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "KeyK".to_string(),
    });
    keybindings.insert("global.openSettings".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "Comma".to_string(),
    });
    keybindings.insert("global.toggleSidebar".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "KeyB".to_string(),
    });

    // 笔记编辑器快捷键
    keybindings.insert("note.save".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "KeyS".to_string(),
    });
    keybindings.insert("note.find".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "KeyF".to_string(),
    });
    keybindings.insert("note.closeTab".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "KeyW".to_string(),
    });
    keybindings.insert("note.togglePreview".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "KeyD".to_string(),
    });
    keybindings.insert("note.zoomIn".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "Equal".to_string(),
    });
    keybindings.insert("note.zoomOut".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "Minus".to_string(),
    });
    keybindings.insert("note.zoomReset".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "Digit0".to_string(),
    });

    let presets = vec![
        KeybindingPreset {
            id: "vscode".to_string(),
            name: "VSCode 风格".to_string(),
            description: "类似 VSCode 的快捷键布局".to_string(),
            keybindings: keybindings.clone(),
        },
    ];

    KeybindingsData {
        keybindings,
        presets,
    }
}
