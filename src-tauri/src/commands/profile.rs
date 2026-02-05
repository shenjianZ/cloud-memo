use crate::models::{UserProfile, UpdateProfileRequest};
use crate::services::{UserProfileService, AuthService};
use tauri::State;

/// UserProfile Service 的 State 类型别名
pub type ProfileSvc<'a> = State<'a, UserProfileService>;

/// Auth Service 的 State 类型别名
pub type AuthSvc<'a> = State<'a, AuthService>;

/// 获取当前用户的资料
#[tauri::command]
pub async fn get_user_profile(
    profile_service: ProfileSvc<'_>,
    auth_service: AuthSvc<'_>,
) -> std::result::Result<UserProfile, String> {
    log::debug!("[commands/profile.rs::get_user_profile] 获取用户资料");

    // 从 AuthService 获取当前用户
    let user = auth_service.get_current_user()
        .map_err(|e| {
            log::error!("[commands/profile.rs::get_user_profile] 获取当前用户失败: {}", e);
            e.to_string()
        })?;

    log::debug!("[commands/profile.rs::get_user_profile] 当前用户: user_id={}", user.id);

    // 使用 user.id 获取用户资料
    profile_service
        .get_profile(&user.id)
        .map_err(|e| {
            log::error!("[commands/profile.rs::get_user_profile] 获取资料失败: user_id={}, error={}", user.id, e);
            e.to_string()
        })
        .map(|profile| {
            log::debug!("[commands/profile.rs::get_user_profile] 获取成功: user_id={}", user.id);
            profile
        })
}

/// 更新当前用户的资料
#[tauri::command]
pub async fn update_user_profile(
    req: UpdateProfileRequest,
    profile_service: ProfileSvc<'_>,
    auth_service: AuthSvc<'_>,
) -> std::result::Result<UserProfile, String> {
    log::info!("[commands/profile.rs::update_user_profile] 更新用户资料");

    // 从 AuthService 获取当前用户
    let user = auth_service.get_current_user()
        .map_err(|e| {
            log::error!("[commands/profile.rs::update_user_profile] 获取当前用户失败: {}", e);
            e.to_string()
        })?;

    log::info!("[commands/profile.rs::update_user_profile] 当前用户: user_id={}", user.id);

    // 使用 user.id 更新用户资料
    profile_service
        .update_profile(&user.id, req)
        .map_err(|e| {
            log::error!("[commands/profile.rs::update_user_profile] 更新失败: user_id={}, error={}", user.id, e);
            e.to_string()
        })
        .map(|profile| {
            log::info!("[commands/profile.rs::update_user_profile] 更新成功: user_id={}", user.id);
            profile
        })
}

/// 同步用户资料到云端
#[tauri::command]
pub async fn sync_profile(
    profile_service: ProfileSvc<'_>,
    auth_service: AuthSvc<'_>,
) -> std::result::Result<UserProfile, String> {
    log::info!("[commands/profile.rs::sync_profile] 同步用户资料到云端");

    // 从 AuthService 获取当前用户
    let user = auth_service.get_current_user()
        .map_err(|e| {
            log::error!("[commands/profile.rs::sync_profile] 获取当前用户失败: {}", e);
            e.to_string()
        })?;

    log::info!("[commands/profile.rs::sync_profile] 当前用户: user_id={}", user.id);

    // 同步用户资料
    profile_service
        .sync_profile(&user.id)
        .await
        .map_err(|e| {
            log::error!("[commands/profile.rs::sync_profile] 同步失败: user_id={}, error={}", user.id, e);
            e.to_string()
        })
        .map(|profile| {
            log::info!("[commands/profile.rs::sync_profile] 同步成功: user_id={}", user.id);
            profile
        })
}
