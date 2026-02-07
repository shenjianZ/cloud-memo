import { invoke } from '@tauri-apps/api/core'
import type { AppSettings, UpdateAppSettings } from '@/types/appSettings'

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
