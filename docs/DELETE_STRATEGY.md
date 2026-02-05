# 删除策略设计文档

## 📋 概述

本文档定义了笔记应用中 **Folder（文件夹）** 和 **Note（笔记）** 的删除策略，确保数据一致性、用户体验和云同步友好性。

---

## 🎯 核心原则

| 对象 | 删除方式 | 回收站支持 | 理由 |
|------|---------|-----------|------|
| **Folder** | 物理删除（DELETE） | ❌ 不支持 | 树结构必须完整，外键自动维护 |
| **Note** | 软删除（is_deleted） | ✅ 支持 | 用户内容保护，可恢复 |

### 设计理念

```
┌─────────────────────────────────────────────────────────┐
│  Folder = 组织结构（动态、可调整、不可恢复）              │
│  Note    = 用户内容（永久、可恢复、受保护）                │
└─────────────────────────────────────────────────────────┘
```

---

## 📁 Folder 删除策略

### 删除方式：物理删除（Hard Delete）

```sql
DELETE FROM folders WHERE id = ?;
```

### 数据库外键行为

#### 1️⃣ folders 表自引用外键

```sql
FOREIGN KEY (parent_id) REFERENCES folders(id)
ON DELETE CASCADE
```

**结果**：
- ✅ 删除目标文件夹
- ✅ **级联删除所有子文件夹**（自动递归）
- ✅ 子文件夹的子文件夹……整棵子树全部删除

#### 2️⃣ notes 表外键

```sql
FOREIGN KEY (folder_id) REFERENCES folders(id)
ON DELETE SET NULL
```

**结果**：
- ✅ **笔记不会被删除**（用户内容保护）
- ✅ 所有笔记的 `folder_id` 设为 `NULL`
- ✅ 笔记变成"未分类"状态

---

## 📖 删除场景示例

### 场景 1：删除包含笔记的文件夹

**删除前**：
```
📁 工作文件夹
  ├─ 📄 项目A笔记
  ├─ 📄 项目B笔记
  └─ 📁 2024
      └─ 📄 年度计划
```

**删除后**：
```
📄 项目A笔记（未分类）
📄 项目B笔记（未分类）
📄 年度计划（未分类）

❌ 工作文件夹（已删除）
❌ 2024（级联删除）
```

**数据库状态**：
```sql
-- folders 表
id: "work-id"        → 已删除（DELETE）
id: "2024-id"        → 已删除（CASCADE）

-- notes 表
id: "note-a-id"      → folder_id = NULL（SET NULL）
id: "note-b-id"      → folder_id = NULL（SET NULL）
id: "plan-id"        → folder_id = NULL（SET NULL）
```

---

### 场景 2：删除空文件夹

**删除前**：
```
📁 空文件夹（无子文件夹，无笔记）
```

**删除后**：
```
❌ 空文件夹（已删除）
```

**结果**：
- 文件夹直接删除
- 无级联影响
- 无笔记受影响

---

### 场景 3：删除多层嵌套文件夹

**删除前**：
```
📁 根文件夹
  ├─ 📄 笔记1
  ├─ 📁 子文件夹A
  │   ├─ 📄 笔记2
  │   └─ 📁 孙文件夹
  │       └─ 📄 笔记3
  └─ 📄 笔记4
```

**删除"根文件夹"后**：
```
📄 笔记1（未分类）
📄 笔记2（未分类）
📄 笔记3（未分类）
📄 笔记4（未分类）

❌ 根文件夹（已删除）
❌ 子文件夹A（级联删除）
❌ 孙文件夹（级联删除）
```

---

## 📝 Note 删除策略

### 删除方式：软删除（Soft Delete）

```sql
UPDATE notes
SET is_deleted = 1,
    deleted_at = ?
WHERE id = ?;
```

### 特点

- ✅ **可恢复**：通过回收站恢复
- ✅ **延迟清理**：支持"30天后自动清空回收站"
- ✅ **用户友好**：误删除可撤销
- ✅ **同步友好**：删除状态可同步到其他设备

---

## ⚠️ 为什么 Folder 不使用软删除？

### 问题 1：树结构逻辑断裂

**软删除父文件夹后**：
```
📁 父文件夹（is_deleted = 1）← 从查询中过滤掉
  └─ 📁 子文件夹（is_deleted = 0）← 仍然可见？
```

**结果**：
- 子文件夹的 `parent_id` 指向一个"已删除"的父节点
- 查询树时需要额外过滤 `is_deleted`
- 恢复父文件夹时，子文件夹状态可能不一致

### 问题 2：同步状态冲突

```
设备A: 删除父文件夹（软删除）
设备B: 同步时收到"父文件夹 is_deleted=1"
       但子文件夹仍然是 is_deleted=0
       → 树结构不一致！
```

### 问题 3：代码复杂度

需要手动处理：
- 递归查询子文件夹
- 级联标记删除
- 恢复时的状态一致性
- 同步时的冲突解决

**物理删除的优势**：
```rust
// 只需一行代码，数据库自动处理
conn.execute("DELETE FROM folders WHERE id = ?", params![id])?;
```

---

## 🔒 安全保障

### 用户提示

删除文件夹前应显示警告：

```
⚠️ 确认删除"工作"文件夹？

此操作将：
• 删除"工作"文件夹及其所有子文件夹（不可恢复）
• 保留所有笔记，移至"未分类"

[取消] [确认删除]
```

### 笔记保护机制

| 场景 | 笔记状态 | 说明 |
|------|---------|------|
| 删除文件夹 | ✅ 保留 | `folder_id` 设为 NULL |
| 删除笔记 | ✅ 保留 | `is_deleted = 1`，可恢复 |
| 清空回收站 | ❌ 删除 | 超过保留期，物理删除 |

---

## 🌐 云同步考虑

### Folder 删除事件

```json
{
  "type": "folder_deleted",
  "folder_id": "xxx",
  "timestamp": 1234567890,
  "cascade": true  // 服务器知道是级联删除
}
```

**服务器处理**：
- 删除该文件夹及其所有子文件夹
- 所有笔记的 `folder_id` 设为 NULL
- 同步事件到其他设备

### Note 删除事件

```json
{
  "type": "note_deleted",
  "note_id": "xxx",
  "timestamp": 1234567890,
  "is_soft_delete": true  // 软删除，可恢复
}
```

---

## 📊 对比：软删除 vs 物理删除

| 维度 | Folder 软删除 | Folder 物理删除 | Note 软删除 |
|------|-------------|---------------|------------|
| **树结构完整性** | ❌ 会断裂 | ✅ 保持完整 | N/A |
| **代码复杂度** | ❌ 高（需递归） | ✅ 低（数据库处理） | ✅ 低 |
| **恢复能力** | ✅ 可恢复 | ❌ 不可恢复 | ✅ 可恢复 |
| **同步友好** | ❌ 状态冲突 | ✅ 事件清晰 | ✅ 状态明确 |
| **性能** | ❌ 需递归查询 | ✅ 一次 DELETE | ✅ 简单 UPDATE |
| **用户体验** | ⚠️ 需额外提示 | ⚠️ 需警告提示 | ✅ 回收站 |

---

## 🎓 类似产品的策略

### Notion
- **Page（类似 Folder）**：物理删除
- **Block（类似 Note）**：软删除，回收站保留 30 天

### Obsidian Sync
- **文件夹**：物理删除
- **笔记**：软删除，.obsidian/trash 记录

### Bear
- **文件夹（标签）**：物理删除
- **笔记**：软删除，回收站

---

## ✅ 实现检查清单

### Repository 层
- [x] `FolderRepository::hard_delete()` - 物理删除
- [x] `FolderRepository::soft_delete()` - 标记废弃
- [x] `NoteRepository::soft_delete()` - 软删除

### Service 层
- [x] `FolderService::delete_folder()` - 使用 `hard_delete()`
- [x] `NoteService::delete_note()` - 使用 `soft_delete()`

### Command 层
- [ ] 前端调用 `deleteFolder` 时显示警告提示
- [ ] 提供"清空回收站"功能（物理删除软删除的笔记）

### 前端 UI
- [ ] 删除文件夹前显示确认对话框
- [ ] 显示"未分类"笔记视图
- [ ] 回收站功能（笔记恢复/永久删除）

---

## 🔗 相关代码

### 数据库 Schema

```sql
-- folders 表
CREATE TABLE folders (
  id CHAR(36) PRIMARY KEY,
  parent_id CHAR(36),
  FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
);

-- notes 表
CREATE TABLE notes (
  id CHAR(36) PRIMARY KEY,
  folder_id CHAR(36),
  FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);
```

### Rust 实现

```rust
// FolderRepository
pub fn hard_delete(&self, id: &str) -> Result<()> {
    conn.execute("DELETE FROM folders WHERE id = ?", params![id])?;
    Ok(())
}

// FolderService
pub fn delete_folder(&self, id: &str) -> Result<()> {
    self.get_folder(id)?;  // 验证存在
    self.repo.hard_delete(id)  // 物理删除
}
```

---

## 📝 总结

| 要点 | 说明 |
|------|------|
| **Folder** | 物理删除，外键自动级联，笔记释放到未分类 |
| **Note** | 软删除，可恢复，回收站保护 |
| **优势** | 树结构完整、代码简单、同步友好 |
| **注意** | 文件夹删除不可恢复，需提示用户 |

**核心思想**：文件夹是"容器"，可以删；笔记是"内容"，要保护。
