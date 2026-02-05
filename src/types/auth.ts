/**
 * 登录请求接口
 */
export interface LoginRequest {
  email: string
  password: string
  serverUrl: string
}

/**
 * 注册请求接口
 */
export interface RegisterRequest {
  email: string
  password: string
  serverUrl: string
}

/**
 * 认证响应接口
 */
export interface AuthResponse {
  token: string
  refreshToken?: string
  userId: string
  email: string
  expiresAt?: number
}

/**
 * 用户资料接口
 */
export interface UserProfile {
  id?: number
  userId: string  // camelCase
  username?: string
  phone?: string
  qq?: string
  wechat?: string
  avatarData?: string  // camelCase - Base64 编码的图片数据
  avatarMimeType?: string  // camelCase - 图片 MIME 类型
  bio?: string
  createdAt: number  // camelCase
  updatedAt: number  // camelCase
}

/**
 * 更新用户资料请求
 */
export interface UpdateProfileRequest {
  username?: string
  phone?: string
  qq?: string
  wechat?: string
  avatarData?: string  // Base64 编码的图片数据（前端使用 camelCase）
  avatarMimeType?: string  // 图片 MIME 类型（前端使用 camelCase）
  bio?: string
}

/**
 * 用户信息接口
 */
export interface User {
  id: string
  email: string
  serverUrl: string
  deviceId: string
  lastSyncAt?: number
}

/**
 * 账号信息接口（包含用户资料）
 */
export interface AccountWithProfile {
  id: string
  email: string
  serverUrl: string
  deviceId: string
  lastSyncAt?: number
  profile?: UserProfile  // 用户资料（可选）
}
