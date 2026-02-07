import { invoke } from '@tauri-apps/api/core';
import type { Folder, CreateFolderRequest, UpdateFolderRequest, MoveFolderRequest, MoveNotesRequest } from '@/types/folder';

/**
 * 创建文件夹
 */
export async function createFolder(req: CreateFolderRequest): Promise<Folder> {
  return invoke('create_folder', { req });
}

/**
 * 获取单个文件夹
 */
export async function getFolder(id: string): Promise<Folder> {
  return invoke('get_folder', { id });
}

/**
 * 更新文件夹
 */
export async function updateFolder(req: UpdateFolderRequest): Promise<Folder> {
  return invoke('update_folder', { req });
}

/**
 * 删除文件夹（软删除）
 */
export async function deleteFolder(id: string): Promise<void> {
  return invoke('delete_folder', { id });
}

/**
 * 获取所有文件夹
 */
export async function listFolders(): Promise<Folder[]> {
  return invoke('list_folders');
}

/**
 * 移动文件夹
 */
export async function moveFolder(req: MoveFolderRequest): Promise<Folder> {
  return invoke('move_folder', { req });
}

/**
 * 获取文件夹路径
 */
export async function getFolderPath(id: string): Promise<Folder[]> {
  return invoke('get_folder_path', { id });
}

/**
 * 批量移动笔记到文件夹
 */
export async function moveNotesToFolder(req: MoveNotesRequest): Promise<void> {
  return invoke('move_notes_to_folder', { req });
}
