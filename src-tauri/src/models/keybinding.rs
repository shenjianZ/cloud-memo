use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 快捷键组合
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KeyCombination {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    pub key: String,
}

/// 快捷键预设
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KeybindingPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub keybindings: HashMap<String, KeyCombination>,
}

/// 快捷键数据
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KeybindingsData {
    pub keybindings: HashMap<String, KeyCombination>,
    pub presets: Vec<KeybindingPreset>,
}

/// 获取默认快捷键配置
pub fn get_default_keybindings() -> KeybindingsData {
    let mut keybindings = HashMap::new();

    // 全局快捷键
    keybindings.insert("global.newNote".to_string(), KeyCombination {
        ctrl: true, alt: false, shift: false, meta: false, key: "KeyN".to_string(),
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
