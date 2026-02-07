/**
 * 文件夹类型定义
 */

/**
 * 文件夹模型
 */
export interface Folder {
  id: string;
  name: string;
  parentId?: string;
  icon?: string;
  color?: string;
  sortOrder: number;
  isDeleted: boolean;
  createdAt: number;
  updatedAt: number;
  deletedAt?: number;
}

/**
 * 创建文件夹请求
 */
export interface CreateFolderRequest {
  name: string;
  parentId?: string;
  color?: string;
  icon?: string;
}

/**
 * 更新文件夹请求
 */
export interface UpdateFolderRequest {
  id: string;
  name?: string;
  parentId?: string;
  color?: string;
  icon?: string;
  sortOrder?: number;
}

/**
 * 移动文件夹请求
 */
export interface MoveFolderRequest {
  id: string;
  newParentId?: string;
  newSortOrder?: number;
}

/**
 * 批量移动笔记请求
 */
export interface MoveNotesRequest {
  noteIds: string[];
  folderId?: string;
}
