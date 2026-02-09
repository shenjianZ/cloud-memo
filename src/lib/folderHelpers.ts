/**
 * 文件夹辅助函数
 */

/**
 * 获取已有文件夹的层级深度
 * @param folderId 文件夹 ID
 * @param folders 所有文件夹列表
 * @returns 层级深度（根文件夹为 0）
 */
export function getFolderDepth(
  folderId: string,
  folders: Array<{ id: string; parentId: string | null }>
): number {
  let depth = 0;
  let currentFolder = folders.find((f) => f.id === folderId);

  while (currentFolder?.parentId) {
    depth++;
    currentFolder = folders.find((f) => f.id === currentFolder!.parentId);
  }

  return depth;
}

/**
 * 计算新创建文件夹的层级深度
 * @param parentId 父文件夹 ID（null 表示根文件夹）
 * @param folders 所有文件夹列表
 * @returns 新文件夹的层级深度
 */
export function getNewFolderDepth(
  parentId: string | null,
  folders: Array<{ id: string; parentId: string | null }>
): number {
  // 根文件夹深度为 0
  if (!parentId) return 0;

  // 子文件夹深度 = 父文件夹深度 + 1
  const parentDepth = getFolderDepth(parentId, folders);
  return parentDepth + 1;
}

/**
 * 根据文件夹层级获取默认颜色
 * 使用精心挑选的颜色方案，确保各层级之间有明显的视觉区分
 * @param depth 文件夹层级深度
 * @returns 颜色值（十六进制）
 */
export function getDefaultFolderColor(depth: number): string {
  // 颜色方案：基于 Tailwind CSS 色板，选择饱和度适中、亮度适中的颜色
  // 这些颜色在深色和浅色背景下都有良好的可读性
  const colorScheme = [
    '#3b82f6', // 层级 0: blue-500 - 蓝色（专业、可信赖）
    '#8b5cf6', // 层级 1: violet-500 - 紫色（优雅、柔和）
    '#ec4899', // 层级 2: pink-500 - 粉色（温暖、友好）
    '#f97316', // 层级 3: orange-500 - 橙色（活力、醒目）
    '#22c55e', // 层级 4: green-500 - 绿色（清新、自然）
    '#14b8a6', // 层级 5: teal-500 - 青色（平静、稳重）
    '#64748b', // 层级 6+: slate-500 - 灰色（低调、中性）
  ];

  // 超过 6 层的文件夹都使用灰色
  return colorScheme[Math.min(depth, colorScheme.length - 1)];
}
