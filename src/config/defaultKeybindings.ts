import type { KeybindingPreset, KeyCombination } from '@/types/keybinding';

/**
 * 默认快捷键配置
 * 根据 VSCode 风格设计
 */
export const DEFAULT_KEYBINDINGS: Record<string, KeyCombination> = {
  // 全局快捷键
  'global.newNote': { ctrl: true, alt: false, shift: false, meta: false, key: 'KeyN' },
  'global.openSearch': { ctrl: true, alt: false, shift: false, meta: false, key: 'KeyK' },
  'global.openSettings': { ctrl: true, alt: false, shift: false, meta: false, key: 'Comma' },
  'global.toggleSidebar': { ctrl: true, alt: false, shift: false, meta: false, key: 'KeyB' },

  // 笔记编辑器快捷键
  'note.save': { ctrl: true, alt: false, shift: false, meta: false, key: 'KeyS' },
  'note.find': { ctrl: true, alt: false, shift: false, meta: false, key: 'KeyF' },
  'note.closeTab': { ctrl: true, alt: false, shift: false, meta: false, key: 'KeyW' },
  'note.togglePreview': { ctrl: true, alt: false, shift: false, meta: false, key: 'KeyD' },
  'note.zoomIn': { ctrl: true, alt: false, shift: false, meta: false, key: 'Equal' },
  'note.zoomOut': { ctrl: true, alt: false, shift: false, meta: false, key: 'Minus' },
  'note.zoomReset': { ctrl: true, alt: false, shift: false, meta: false, key: 'Digit0' },
};

/**
 * VSCode 风格预设
 */
export const VSCODE_PRESET: KeybindingPreset = {
  id: 'vscode',
  name: 'VSCode 风格',
  description: '类似 VSCode 的快捷键布局',
  keybindings: DEFAULT_KEYBINDINGS,
}

/**
 * 所有可用的预设方案
 */
export const KEYBINDING_PRESETS: KeybindingPreset[] = [VSCODE_PRESET]

