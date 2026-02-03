# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 **Tauri 2** 的桌面笔记应用，支持 Markdown 编辑。前端使用 React + TypeScript + Vite，后端使用 Rust + SQLite。

## 常用命令

### 依赖安装
```bash
pnpm install
```

### 开发
```bash
pnpm tauri:dev    # 启动 Tauri 开发环境（前端端口固定为 1420）
pnpm dev          # 仅启动前端开发服务器
```

### 构建
```bash
pnpm build        # 构建前端 (tsc + vite build)
pnpm tauri:build  # 构建完整的桌面应用
```

## 架构概览

### 前后端通信
- 前端通过 `@tauri-apps/api` 的 `invoke()` 函数调用后端命令
- 后端命令定义在 `src-tauri/src/commands/` 目录
- 所有跨边界调用都是异步的

### 后端架构（Rust 三层架构）
```
commands/     # Tauri 命令层（API 入口）
  ↓
services/     # 业务逻辑层
  ↓
repositories/ # 数据访问层
  ↓
database/     # SQLite 数据库
```

### 前端架构（React 分层架构）
```
pages/        # 页面组件
  ↓
components/   # UI 组件
  ↓
store/        # Zustand 状态管理（持久化到 localStorage）
  ↓
services/     # API 调用封装（调用 Tauri commands）
```

## 核心技术特性

### 编辑器系统
- **主编辑器**: Tiptap (所见即所得)，内容存储为 JSON 对象
- **备用编辑器**: CodeMirror 6 (纯 Markdown 模式)
- 编辑器类型在 `editorSettingsStore` 中配置
- 自动保存：1 秒防抖（`services/autoSave.ts`）

### 数据库设计
- 使用 SQLite FTS5 全文搜索
- 软删除机制（`is_deleted` 字段）
- 触发器自动同步 FTS 索引

### 快捷键系统
- 三级优先级：组件级 > 页面级 > 全局
- 作用域：Global / Editor / Settings
- 用户可自定义，存储在 `keybindings.json`
- 全局处理器：`lib/globalKeyHandler.ts`（单例模式）

### 状态管理
- 使用 Zustand + persist 中间件
- Store 定义在 `src/store/` 目录
- 每个 store 独立管理一个领域的状态

## 路径别名
- `@/*` 映射到 `./src/*`（在 `tsconfig.json` 和 `vite.config.ts` 中配置）

## 重要配置说明

### Vite 开发服务器
- 端口固定为 **1420**（Tauri 要求）
- 不要修改端口，否则会导致 Tauri 连接失败

### Tauri 配置
- 应用窗口大小：1200x700（最小 800x600）
- 开发工具：已启用（`devtools: true`）
- 前端构建输出：`../dist`

## 命名规范
- **Rust**: snake_case（变量/函数），PascalCase（类型/结构体）
- **TypeScript**: camelCase（变量/属性），PascalCase（类型/组件）
- **React 组件**: PascalCase（如 `TiptapEditor`）
- **Hooks**: `use` 前缀（如 `useTiptapEditor`）

## 添加新功能时的注意事项

### 后端（Rust）
1. 在 `models/` 定义数据模型
2. 在 `database/repositories/` 创建 Repository（数据访问）
3. 在 `services/` 创建 Service（业务逻辑）
4. 在 `commands/` 创建 Command（暴露给前端）
5. 在 `commands/mod.rs` 中注册命令

### 前端（TypeScript）
1. 在 `types/` 定义 TypeScript 类型
2. 在 `services/` 创建 API 函数（调用 Tauri invoke）
3. 在 `store/` 创建 Zustand store（如需要状态管理）
4. 在 `components/` 或 `pages/` 创建 UI 组件

## 编辑器类型转换
- Tiptap JSON ↔ Markdown: 使用 `lib/tiptapMarkdown.ts`
- 向后兼容：旧笔记存储为 Markdown 字符串
