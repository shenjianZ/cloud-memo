import apiClient from './client'

export interface LoginRequest {
  email: string
  password: string
}

export interface RegisterRequest {
  email: string
  password: string
}

export interface AuthResponse {
  token: string
  user_id: string
  email: string
}

export interface User {
  id: string
  email: string
}

/**
 * 用户登录
 */
export async function login(req: LoginRequest): Promise<AuthResponse> {
  const response = await apiClient.post<AuthResponse>('/auth/login', req)
  // 存储 token
  localStorage.setItem('auth_token', response.data.token)
  return response.data
}

/**
 * 用户注册
 */
export async function register(req: RegisterRequest): Promise<AuthResponse> {
  const response = await apiClient.post<AuthResponse>('/auth/register', req)
  // 存储 token
  localStorage.setItem('auth_token', response.data.token)
  return response.data
}

/**
 * 用户登出
 */
export async function logout(): Promise<void> {
  try {
    await apiClient.post('/auth/logout')
  } finally {
    // 无论成功失败都清除本地 token
    localStorage.removeItem('auth_token')
  }
}

/**
 * 获取当前用户信息（从 JWT 解码）
 */
export async function getCurrentUser(): Promise<User> {
  const token = localStorage.getItem('auth_token')
  if (!token) {
    throw new Error('No token found')
  }

  try {
    // 简单的 JWT 解码（不验证签名）
    const payload = JSON.parse(atob(token.split('.')[1]))
    return {
      id: payload.sub,
      email: payload.email || '',
    }
  } catch (error) {
    throw new Error('Invalid token')
  }
}
