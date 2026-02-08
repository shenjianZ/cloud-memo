use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// 工作空间模型
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    // ===== 基础字段 =====
    pub id: String,              // 工作空间唯一标识（UUID）
    pub user_id: String,         // 用户 ID
    pub name: String,            // 工作空间名称

    // ===== 显示属性 =====
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,  // 工作空间描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,        // 图标
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,       // 颜色标识

    // ===== 排序和状态 =====
    #[serde(default)]
    pub is_default: bool,       // 是否为默认工作空间
    #[serde(default)]
    pub is_current: bool,       // 是否为当前激活的工作空间
    #[serde(default)]
    pub sort_order: i32,        // 排序顺序

    // ===== 时间戳 =====
    pub created_at: i64,        // 创建时间（Unix 时间戳，秒）
    pub updated_at: i64,        // 更新时间（Unix 时间戳，秒）

    // ===== 软删除字段 =====
    #[serde(default)]
    pub is_deleted: bool,       // 是否已删除（软删除）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,    // 删除时间（Unix 时间戳，秒）

    // ===== 云端同步字段 =====
    #[serde(default)]
    pub server_ver: i32,        // 服务器版本号（用于冲突检测）
    #[serde(default)]
    pub is_dirty: bool,         // 是否需要同步到服务器
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<i64>,  // 最后同步时间（Unix 时间戳，秒）
}

impl Workspace {
    /// 创建新工作空间（构造函数）
    ///
    /// # 参数
    /// - `user_id`: 用户 ID
    /// - `name`: 工作空间名称
    /// - `description`: 工作空间描述（可选）
    /// - `icon`: 图标（可选）
    /// - `color`: 颜色（可选）
    pub fn new(
        user_id: String,
        name: String,
        description: Option<String>,
        icon: Option<String>,
        color: Option<String>,
    ) -> Self {
        Self::new_with_default(user_id, name, description, icon, color, false)
    }

    /// 创建新的工作空间（可指定是否为默认空间）
    pub fn new_with_default(
        user_id: String,
        name: String,
        description: Option<String>,
        icon: Option<String>,
        color: Option<String>,
        is_default: bool,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();

        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            name,
            description,
            icon,
            color,
            is_default,
            is_current: false,
            sort_order: 0,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            deleted_at: None,
            server_ver: 0,
            is_dirty: true,
            last_synced_at: None,
        }
    }
}

/// 创建工作空间请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRequest {
    pub name: String,              // 工作空间名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,  // 工作空间描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,         // 图标
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,        // 颜色标识
}

/// 更新工作空间请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceRequest {
    pub id: String,  // 工作空间 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,         // 新名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,  // 新描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,         // 新图标
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,        // 新颜色
}
