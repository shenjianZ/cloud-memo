import { invoke } from '@tauri-apps/api/core'

export interface AppSettings {
  id: number
  default_server_url: string
  auto_sync_enabled: boolean
  sync_interval_minutes: number
  theme: 'system' | 'light' | 'dark'
  language: string
  updated_at: number
}

export interface UpdateAppSettings {
  default_server_url?: string
  auto_sync_enabled?: boolean
  sync_interval_minutes?: number
  theme?: string
  language?: string
}

/**
 * 获取应用设置
 */
export async function getSettings(): Promise<AppSettings> {
  return await invoke('get_app_settings')
}

/**
 * 更新应用设置
 */
export async function updateSettings(
  updates: UpdateAppSettings
): Promise<AppSettings> {
  return await invoke('update_app_settings', { updates })
}

/**
 * 重置应用设置为默认值
 */
export async function resetSettings(): Promise<AppSettings> {
  return await invoke('reset_app_settings')
}

/**
 * 获取默认服务器 URL
 */
export async function getDefaultServerUrl(): Promise<string> {
  return await invoke('get_default_server_url')
}
