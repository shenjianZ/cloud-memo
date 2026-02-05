import { invoke } from '@tauri-apps/api/core'
import type { UserProfile, UpdateProfileRequest } from '@/types/auth'

/**
 * 获取当前用户的资料
 */
export async function getUserProfile(): Promise<UserProfile> {
  return await invoke<UserProfile>('get_user_profile')
}

/**
 * 更新当前用户的资料
 */
export async function updateUserProfile(
  req: UpdateProfileRequest
): Promise<UserProfile> {
  console.log('[profileApi] 准备更新用户资料:', {
    ...req,
    avatarData: req.avatarData ? `${req.avatarData.substring(0, 50)}... (${req.avatarData.length} chars)` : 'None',
    avatarMimeType: req.avatarMimeType || 'None'
  })
  const result = await invoke<UserProfile>('update_user_profile', { req })
  console.log('[profileApi] 更新成功，返回的头像:', {
    avatarData: result.avatarData ? `${result.avatarData.substring(0, 50)}... (${result.avatarData.length} chars)` : 'None',
    avatarMimeType: result.avatarMimeType || 'None'
  })
  return result
}

/**
 * 同步用户资料到云端
 */
export async function syncProfile(): Promise<UserProfile> {
  console.log('[profileApi] 准备同步用户资料到云端')
  const result = await invoke<UserProfile>('sync_profile')
  console.log('[profileApi] 用户资料同步成功:', result)
  return result
}
