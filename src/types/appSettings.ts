/**
 * 应用设置类型定义
 */

/**
 * 应用设置
 */
export interface AppSettings {
  id: number;
  defaultServerUrl: string;
  autoSyncEnabled: boolean;
  syncIntervalMinutes: number;
  theme: 'system' | 'light' | 'dark';
  language: string;
  updatedAt: number;
}

/**
 * 更新应用设置
 */
export interface UpdateAppSettings {
  defaultServerUrl?: string;
  autoSyncEnabled?: boolean;
  syncIntervalMinutes?: number;
  theme?: string;
  language?: string;
}
