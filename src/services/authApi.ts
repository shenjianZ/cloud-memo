import { invoke } from "@tauri-apps/api/core";
import type { User, AccountWithProfile, LoginRequest, RegisterRequest, AuthResponse } from "@/types/auth";

/**
 * 用户登录
 */
export async function login(req: LoginRequest): Promise<AuthResponse> {
    console.log("[authApi] 准备调用 Tauri invoke login:", req);
    const result = await invoke<AuthResponse>("login", { req });
    console.log("[authApi] Tauri invoke login 返回:", result);
    return result;
}

/**
 * 用户注册
 */
export async function register(req: RegisterRequest): Promise<AuthResponse> {
    console.log("[authApi] 准备调用 Tauri invoke register");
    console.log("[authApi] 完整的 req 对象:", JSON.stringify(req));

    try {
        const result = await invoke<AuthResponse>("register", { req });
        console.log("[authApi] Tauri invoke register 返回成功:", result);
        return result;
    } catch (error) {
        console.error("[authApi] invoke 失败，错误详情:", error);
        throw error;
    }
}

/**
 * 用户登出
 */
export async function logout(): Promise<void> {
    return await invoke("logout");
}

/**
 * 获取当前登录用户
 */
export async function getCurrentUser(): Promise<User> {
    return await invoke<User>("get_current_user");
}

/**
 * 检查是否已登录
 */
export async function isAuthenticated(): Promise<boolean> {
    return await invoke<boolean>("is_authenticated");
}

/**
 * 获取所有已登录的账号列表
 */
export async function listAccounts(): Promise<AccountWithProfile[]> {
    return await invoke<AccountWithProfile[]>("list_accounts");
}

/**
 * 切换到指定账号
 */
export async function switchAccount(userId: string): Promise<void> {
    return await invoke("switch_account", { userId });
}

/**
 * 删除指定账号
 */
export async function removeAccount(userId: string): Promise<void> {
    return await invoke("remove_account", { userId });
}

/**
 * 刷新 access_token（使用 refresh_token）
 */
export async function refreshAccessToken(): Promise<AuthResponse> {
    return await invoke("refresh_access_token");
}
