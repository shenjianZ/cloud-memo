use crate::services::AuthService;
use crate::models::{LoginRequest, RegisterRequest, AuthResponse, User, AccountWithProfile};
use tauri::State;

/// Auth service 类型别名
type AuthSvc<'a> = State<'a, AuthService>;

/// 用户登录
#[tauri::command]
pub async fn login(
    req: LoginRequest,
    auth_service: AuthSvc<'_>,
) -> std::result::Result<AuthResponse, String> {
    log::info!("[commands/auth.rs::login] 收到登录请求: email={}, server_url={}", req.email, req.server_url);

    auth_service.login(req)
        .await
        .map_err(|e| {
            log::error!("[commands/auth.rs::login] 登录失败: {}", e);
            e.to_string()
        })
        .map(|result| {
            log::info!("[commands/auth.rs::login] 登录成功: user_id={}, email={}", result.user_id, result.email);
            result
        })
}

/// 用户注册
#[tauri::command]
pub async fn register(
    req: RegisterRequest,
    auth_service: AuthSvc<'_>,
) -> std::result::Result<AuthResponse, String> {
    log::info!("[commands/auth.rs] 收到注册请求: email={}, server_url={}", req.email, req.server_url);

    let result = auth_service.register(req)
        .await
        .map_err(|e| {
            log::error!("[commands/auth.rs] 注册失败: {}", e);
            e.to_string()
        })?;

    log::info!("[commands/auth.rs] 注册成功: user_id={}", result.user_id);
    Ok(result)
}

/// 用户登出
#[tauri::command]
pub async fn logout(
    service: AuthSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/auth.rs::logout] 收到登出请求");

    service.logout()
        .map_err(|e| {
            log::error!("[commands/auth.rs::logout] 登出失败: {}", e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/auth.rs::logout] 登出成功");
        })
}

/// 获取当前登录用户
#[tauri::command]
pub async fn get_current_user(
    service: AuthSvc<'_>,
) -> std::result::Result<User, String> {
    log::debug!("[commands/auth.rs::get_current_user] 获取当前用户");

    service.get_current_user()
        .map_err(|e| {
            log::error!("[commands/auth.rs::get_current_user] 获取失败: {}", e);
            e.to_string()
        })
        .map(|user| {
            log::info!("[commands/auth.rs::get_current_user] 获取成功: user_id={}, email={}", user.id, user.email);
            user
        })
}

/// 检查是否已登录
#[tauri::command]
pub async fn is_authenticated(
    service: AuthSvc<'_>,
) -> std::result::Result<bool, String> {
    log::debug!("[commands/auth.rs::is_authenticated] 检查认证状态");

    service.is_authenticated()
        .map_err(|e| {
            log::error!("[commands/auth.rs::is_authenticated] 检查失败: {}", e);
            e.to_string()
        })
        .map(|is_auth| {
            log::debug!("[commands/auth.rs::is_authenticated] 认证状态: {}", is_auth);
            is_auth
        })
}

/// 获取所有已登录的账号列表
#[tauri::command]
pub async fn list_accounts(
    service: AuthSvc<'_>,
) -> std::result::Result<Vec<AccountWithProfile>, String> {
    log::debug!("[commands/auth.rs::list_accounts] 获取账号列表");

    service.list_accounts()
        .map_err(|e| {
            log::error!("[commands/auth.rs::list_accounts] 获取失败: {}", e);
            e.to_string()
        })
        .map(|accounts| {
            log::debug!("[commands/auth.rs::list_accounts] 找到 {} 个账号", accounts.len());
            accounts
        })
}

/// 切换到指定账号
#[tauri::command]
pub async fn switch_account(
    user_id: String,
    service: AuthSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/auth.rs::switch_account] 切换账号: user_id={}", user_id);

    service.switch_account(&user_id)
        .map_err(|e| {
            log::error!("[commands/auth.rs::switch_account] 切换失败: {}", e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/auth.rs::switch_account] 切换成功: user_id={}", user_id);
        })
}

/// 删除指定账号
#[tauri::command]
pub async fn remove_account(
    user_id: String,
    service: AuthSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/auth.rs::remove_account] 删除账号: user_id={}", user_id);

    service.remove_account(&user_id)
        .map_err(|e| {
            log::error!("[commands/auth.rs::remove_account] 删除失败: {}", e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/auth.rs::remove_account] 删除成功: user_id={}", user_id);
        })
}

/// 刷新 access_token（使用 refresh_token）
#[tauri::command]
pub async fn refresh_access_token(
    service: AuthSvc<'_>,
) -> std::result::Result<AuthResponse, String> {
    log::info!("[commands/auth.rs::refresh_access_token] 刷新 access_token");

    service.refresh_access_token()
        .await
        .map_err(|e| {
            log::error!("[commands/auth.rs::refresh_access_token] 刷新失败: {}", e);
            e.to_string()
        })
        .map(|result| {
            log::info!("[commands/auth.rs::refresh_access_token] 刷新成功: user_id={}", result.user_id);
            result
        })
}
