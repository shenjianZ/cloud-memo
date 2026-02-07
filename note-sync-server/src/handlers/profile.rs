use super::ErrorResponse;
use crate::middleware::logging::{log_info, RequestId};
use crate::services::profile_service::{
    CreateProfileRequest, ProfileService, UpdateProfileRequest, UserProfile,
};
use crate::AppState;
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProfileSyncRequest {
    pub user_id: String,
    pub username: Option<String>,
    pub phone: Option<String>,
    pub qq: Option<String>,
    pub wechat: Option<String>,
    pub avatar_data: Option<String>, // 头像图片数据（Base64 编码）
    pub avatar_mime_type: Option<String>, // 头像图片类型
    pub bio: Option<String>,
}

/// 图片验证结果
pub enum ValidationError {
    InvalidBase64,
    InvalidImageFormat,
    ImageTooLarge { size: usize, max_size: usize },
    UnsupportedMimeType(String),
}

impl ValidationError {
    pub fn message(&self) -> String {
        match self {
            ValidationError::InvalidBase64 => "无效的 Base64 编码".to_string(),
            ValidationError::InvalidImageFormat => "无效的图片格式".to_string(),
            ValidationError::ImageTooLarge { size, max_size } => {
                format!("图片过大: {} 字节（最大 {} 字节）", size, max_size)
            }
            ValidationError::UnsupportedMimeType(mime) => {
                format!("不支持的图片类型: {}", mime)
            }
        }
    }
}

/// 验证图片数据
pub fn validate_avatar_data(
    avatar_data: &Option<String>,
    avatar_mime_type: &Option<String>,
) -> Result<(), ValidationError> {
    // 如果没有上传头像，直接返回
    if avatar_data.is_none() && avatar_mime_type.is_none() {
        return Ok(());
    }

    // 确保两个字段同时存在
    let data = avatar_data.as_ref().ok_or(ValidationError::InvalidBase64)?;
    let mime_type = avatar_mime_type
        .as_ref()
        .ok_or(ValidationError::UnsupportedMimeType("未指定".to_string()))?;

    // 验证 MIME 类型
    match mime_type.as_str() {
        "image/jpeg" | "image/png" | "image/gif" | "image/webp" | "image/bmp" | "image/svg+xml" => {
        }
        _ => return Err(ValidationError::UnsupportedMimeType(mime_type.clone())),
    }

    // 验证 Base64 编码
    use base64::{engine::general_purpose, Engine as _};
    let decoded = general_purpose::STANDARD
        .decode(data)
        .map_err(|_| ValidationError::InvalidBase64)?;

    // 验证图片大小（最大 5MB）
    const MAX_AVATAR_SIZE: usize = 5 * 1024 * 1024; // 5MB
    if decoded.len() > MAX_AVATAR_SIZE {
        return Err(ValidationError::ImageTooLarge {
            size: decoded.len(),
            max_size: MAX_AVATAR_SIZE,
        });
    }

    // SVG 格式特殊处理（文本格式，不需要检查魔术字节）
    if mime_type != "image/svg+xml" {
        // 验证图片格式（通过检查魔术字节）
        validate_image_magic_bytes(&decoded, mime_type)?;
    }

    Ok(())
}

/// 验证图片魔术字节
fn validate_image_magic_bytes(data: &[u8], mime_type: &str) -> Result<(), ValidationError> {
    match mime_type {
        "image/jpeg" => {
            // JPEG: FF D8 FF
            if data.len() < 3 || data[0] != 0xFF || data[1] != 0xD8 || data[2] != 0xFF {
                return Err(ValidationError::InvalidImageFormat);
            }
        }
        "image/png" => {
            // PNG: 89 50 4E 47 0D 0A 1A 0A
            const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
            if data.len() < 8 || &data[0..8] != PNG_SIGNATURE {
                return Err(ValidationError::InvalidImageFormat);
            }
        }
        "image/gif" => {
            // GIF: 47 49 46 38 (GIF8)
            if data.len() < 4
                || data[0] != 0x47
                || data[1] != 0x49
                || data[2] != 0x46
                || data[3] != 0x38
            {
                return Err(ValidationError::InvalidImageFormat);
            }
        }
        "image/webp" => {
            // WebP: 52 49 46 46 ... 57 45 42 50 (RIFF...WEBP)
            if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
                return Err(ValidationError::InvalidImageFormat);
            }
        }
        "image/bmp" => {
            // BMP: 42 4D (BM)
            if data.len() < 2 || data[0] != 0x42 || data[1] != 0x4D {
                return Err(ValidationError::InvalidImageFormat);
            }
        }
        _ => {}
    }

    Ok(())
}

/// 获取用户资料
pub async fn get_profile(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<UserProfile>, ErrorResponse> {
    log_info(&request_id, "获取用户资料", &format!("user_id={}", user_id));

    let service = ProfileService::new(state.pool);

    match service.get_profile(&user_id).await {
        Ok(Some(profile)) => {
            log_info(&request_id, "获取成功", &profile);
            Ok(Json(profile))
        }
        Ok(None) => {
            log_info(&request_id, "用户资料不存在", "NOT_FOUND");
            Err(ErrorResponse::new("用户资料不存在"))
        }
        Err(e) => {
            log_info(&request_id, "获取失败", &e.to_string());
            Err(ErrorResponse::new("获取用户资料失败"))
        }
    }
}

/// 创建用户资料
pub async fn create_profile(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Json(req): Json<CreateProfileRequest>,
) -> Result<Json<UserProfile>, ErrorResponse> {
    log_info(&request_id, "创建用户资料请求", &req);

    let service = ProfileService::new(state.pool);

    match service.create_profile(req).await {
        Ok(profile) => {
            log_info(&request_id, "创建成功", &profile);
            Ok(Json(profile))
        }
        Err(e) => {
            log_info(&request_id, "创建失败", &e.to_string());
            Err(ErrorResponse::new("创建用户资料失败"))
        }
    }
}

/// 更新用户资料
pub async fn update_profile(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, ErrorResponse> {
    log_info(
        &request_id,
        "更新用户资料请求",
        &format!("user_id={}, update={:?}", user_id, req),
    );

    let service = ProfileService::new(state.pool);

    match service.update_profile(&user_id, req).await {
        Ok(Some(profile)) => {
            log_info(&request_id, "更新成功", &profile);
            Ok(Json(profile))
        }
        Ok(None) => {
            log_info(&request_id, "用户资料不存在", "NOT_FOUND");
            Err(ErrorResponse::new("用户资料不存在"))
        }
        Err(e) => {
            log_info(&request_id, "更新失败", &e.to_string());
            Err(ErrorResponse::new("更新用户资料失败"))
        }
    }
}

/// 同步用户资料（upsert）
pub async fn sync_profile(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Json(req): Json<ProfileSyncRequest>,
) -> Result<Json<UserProfile>, ErrorResponse> {
    log_info(&request_id, "同步用户资料请求", &req);

    // 验证头像数据
    if let Err(e) = validate_avatar_data(&req.avatar_data, &req.avatar_mime_type) {
        log_info(&request_id, "头像验证失败", e.message());
        return Err(ErrorResponse::new(e.message()));
    }

    let service = ProfileService::new(state.pool);

    let create_req = CreateProfileRequest {
        user_id: req.user_id.clone(),
        username: req.username,
        phone: req.phone,
        qq: req.qq,
        wechat: req.wechat,
        avatar_data: req.avatar_data,
        avatar_mime_type: req.avatar_mime_type,
        bio: req.bio,
    };

    match service.upsert_profile(&req.user_id, create_req).await {
        Ok(profile) => {
            log_info(&request_id, "同步成功", &profile);
            Ok(Json(profile))
        }
        Err(e) => {
            log_info(&request_id, "同步失败", &e.to_string());
            Err(ErrorResponse::new("同步用户资料失败"))
        }
    }
}
