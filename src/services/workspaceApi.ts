import { invoke } from '@tauri-apps/api/core';
import type { Workspace, CreateWorkspaceRequest, UpdateWorkspaceRequest } from '@/types/workspace';

/**
 * 迁移孤立数据的返回结果
 */
export interface MigrateOrphanDataResult {
  notes: number;
  folders: number;
  tags: number;
  snapshots: number;
}

/**
 * 列出所有工作空间
 */
export async function listWorkspaces(): Promise<Workspace[]> {
  return invoke('list_workspaces');
}

/**
 * 创建工作空间
 */
export async function createWorkspace(req: CreateWorkspaceRequest): Promise<Workspace> {
  return invoke('create_workspace', { req });
}

/**
 * 更新工作空间
 */
export async function updateWorkspace(req: UpdateWorkspaceRequest): Promise<Workspace> {
  return invoke('update_workspace', { req });
}

/**
 * 删除工作空间（软删除）
 */
export async function deleteWorkspace(id: string): Promise<void> {
  return invoke('delete_workspace', { id });
}

/**
 * 设置默认工作空间
 */
export async function setDefaultWorkspace(id: string): Promise<void> {
  return invoke('set_default_workspace', { id });
}

/**
 * 获取当前工作空间
 */
export async function getCurrentWorkspace(): Promise<Workspace> {
  return invoke('get_current_workspace');
}

/**
 * 切换工作空间
 */
export async function switchWorkspace(id: string): Promise<void> {
  return invoke('switch_workspace', { id });
}

/**
 * 迁移孤立数据到当前工作空间
 *
 * 将所有 workspace_id = NULL 的数据（未登录时创建的数据）分配到指定的工作空间
 */
export async function migrateOrphanDataToWorkspace(workspaceId: string): Promise<MigrateOrphanDataResult> {
  return invoke('migrate_orphan_data_to_workspace', { workspaceId });
}

