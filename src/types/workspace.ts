/**
 * 工作空间
 */
export interface Workspace {
  id: string
  userId: string
  name: string
  description?: string
  icon?: string
  color?: string
  isDefault: boolean
  isCurrent: boolean
  sortOrder: number
  createdAt: number
  updatedAt: number
  isDeleted: boolean
  deletedAt?: number
  serverVer: number
  isDirty: boolean
  lastSyncedAt?: number
}

/**
 * 创建工作空间请求
 */
export interface CreateWorkspaceRequest {
  name: string
  description?: string
  icon?: string
  color?: string
}

/**
 * 更新工作空间请求
 */
export interface UpdateWorkspaceRequest {
  id: string
  name?: string
  description?: string
  icon?: string
  color?: string
}
