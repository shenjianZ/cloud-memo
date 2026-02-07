/**
 * 同步相关类型定义
 */

/**
 * 冲突解决策略
 */
export type ConflictResolutionStrategy =
  | 'server_wins'      // 服务器优先
  | 'client_wins'      // 本地优先
  | 'create_conflict_copy'  // 创建冲突副本（默认）
  | 'manual_merge';    // 手动合并

/**
 * 同步选项
 */
export interface SyncOptions extends Record<string, unknown> {
  /** 冲突解决策略（默认：创建冲突副本） */
  conflictResolution?: ConflictResolutionStrategy;
  /** 设备ID（用于操作锁） */
  deviceId?: string;
}

/**
 * 同步报告
 */
export interface SyncReport {
  success: boolean;

  // 推送到服务器的详细统计
  pushedNotes: number;
  pushedFolders: number;
  pushedTags: number;
  pushedSnapshots: number;
  pushedNoteTags: number;

  // 从服务器拉取的详细统计
  pulledNotes: number;
  pulledFolders: number;
  pulledTags: number;
  pulledSnapshots: number;
  pulledNoteTags: number;

  // 删除的数据统计
  deletedNotes: number;
  deletedFolders: number;
  deletedTags: number;

  conflictCount: number;
  error?: string;

  // 兼容旧版本的汇总字段
  pushedCount?: number;
  pulledCount?: number;
}

/**
 * 同步状态
 */
export interface SyncStatus {
  lastSyncAt: number | null;
  pendingCount: number;
  conflictCount: number;
  lastError: string | null;
}
