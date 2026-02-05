use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// 文件夹模型
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    // ===== 基础字段 =====
    pub id: String,  // 文件夹唯一标识（UUID）
    pub name: String,  // 文件夹名称

    // ===== 层级关系 =====
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,  // 父文件夹 ID（为空表示根文件夹）

    // ===== 显示属性 =====
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,  // 图标名称或路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,  // 颜色标识（十六进制或颜色名）
    #[serde(default)]
    pub sort_order: i32,  // 排序顺序（同级文件夹内的排序）

    // ===== 状态字段 =====
    #[serde(default)]
    pub is_deleted: bool,  // 是否已删除（软删除）

    // ===== 时间戳 =====
    pub created_at: i64,  // 创建时间（Unix 时间戳，秒）
    pub updated_at: i64,  // 更新时间（Unix 时间戳，秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,  // 删除时间（Unix 时间戳，秒）

    // ===== 云端同步字段 =====
    #[serde(default)]
    pub server_ver: i32,  // 服务器版本号（用于冲突检测）
    #[serde(default)]
    pub is_dirty: bool,  // 是否需要同步到服务器
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<i64>,  // 最后同步时间（Unix 时间戳，秒）
}

impl Folder {
    /// 创建新文件夹（构造函数）
    ///
    /// # 参数
    /// - `name`: 文件夹名称
    /// - `parent_id`: 父文件夹 ID（可选）
    /// - `color`: 颜色标识（可选）
    /// - `icon`: 图标（可选）
    pub fn new(
        name: String,
        parent_id: Option<String>,
        color: Option<String>,
        icon: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            parent_id,
            icon,
            color,
            sort_order: 0,
            is_deleted: false,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            server_ver: 0,
            is_dirty: true,
            last_synced_at: None,
        }
    }

    /// 创建根文件夹（用于默认文件夹）
    pub fn root(name: String) -> Self {
        Self::new(name, None, None, None)
    }
}

/// 创建文件夹请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderRequest {
    pub name: String,  // 文件夹名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,  // 父文件夹 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,  // 颜色标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,  // 图标
}

/// 更新文件夹请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFolderRequest {
    pub id: String,  // 文件夹 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,  // 新名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,  // 新父文件夹 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,  // 新颜色
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,  // 新图标
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,  // 新排序顺序
}

/// 移动文件夹请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveFolderRequest {
    pub id: String,  // 要移动的文件夹 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_parent_id: Option<String>,  // 新父文件夹 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_sort_order: Option<i32>,  // 新排序顺序
}

/// 批量移动笔记请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveNotesRequest {
    pub note_ids: Vec<String>,  // 要移动的笔记 ID 列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,  // 目标文件夹 ID（为空表示移出文件夹）
}
