use serde::{Serialize, Deserialize};
use chrono::Utc;

/// 用户资料
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    #[serde(skip_deserializing)]  // 跳过服务器的 id（UUID），使用客户端自己的 id（i64）
    pub id: Option<i64>,  // 主键 ID（自增）
    pub user_id: String,  // 用户 ID（外键，关联 user_auth.user_id）
    pub username: Option<String>,  // 用户名
    pub phone: Option<String>,  // 手机号
    pub qq: Option<String>,  // QQ 号
    pub wechat: Option<String>,  // 微信号
    pub avatar_data: Option<String>,  // 头像图片数据（Base64 编码）
    pub avatar_mime_type: Option<String>,  // 头像图片类型（image/jpeg, image/png）
    pub bio: Option<String>,  // 个人简介
    pub created_at: i64,  // 创建时间（Unix 时间戳，秒）
    pub updated_at: i64,  // 更新时间（Unix 时间戳，秒）
}

/// 创建用户资料请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]  // 预留功能：将来用户注册时创建资料
pub struct CreateProfileRequest {
    pub user_id: String,  // 用户 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,  // 用户名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,  // 手机号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qq: Option<String>,  // QQ 号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wechat: Option<String>,  // 微信号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_data: Option<String>,  // 头像图片数据（Base64 编码）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_mime_type: Option<String>,  // 头像图片类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,  // 个人简介
}

/// 更新用户资料请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,  // 用户名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,  // 手机号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qq: Option<String>,  // QQ 号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wechat: Option<String>,  // 微信号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_data: Option<String>,  // 头像图片数据（Base64 编码）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_mime_type: Option<String>,  // 头像图片类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,  // 个人简介
}

impl UserProfile {
    /// 创建新的用户资料
    pub fn new(user_id: String) -> Self {
        let now = Utc::now().timestamp();
        UserProfile {
            id: None,
            user_id,
            username: None,
            phone: None,
            qq: None,
            wechat: None,
            avatar_data: None,
            avatar_mime_type: None,
            bio: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 从创建请求创建用户资料
    pub fn from_create_request(req: CreateProfileRequest) -> Self {
        let now = Utc::now().timestamp();
        UserProfile {
            id: None,
            user_id: req.user_id,
            username: req.username,
            phone: req.phone,
            qq: req.qq,
            wechat: req.wechat,
            avatar_data: req.avatar_data,
            avatar_mime_type: req.avatar_mime_type,
            bio: req.bio,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新用户资料
    pub fn update(&mut self, req: UpdateProfileRequest) {
        if req.username.is_some() {
            self.username = req.username;
        }
        if req.phone.is_some() {
            self.phone = req.phone;
        }
        if req.qq.is_some() {
            self.qq = req.qq;
        }
        if req.wechat.is_some() {
            self.wechat = req.wechat;
        }
        if req.avatar_data.is_some() {
            self.avatar_data = req.avatar_data;
        }
        if req.avatar_mime_type.is_some() {
            self.avatar_mime_type = req.avatar_mime_type;
        }
        if req.bio.is_some() {
            self.bio = req.bio;
        }
        self.updated_at = Utc::now().timestamp();
    }
}
