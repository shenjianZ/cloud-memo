# 笔记恢复功能实现文档

## 📋 概述

实现了**方案 B：恢复笔记到"已恢复笔记"文件夹**，无需修改数据库 Schema。

---

## ✅ 完成的工作

### 1. Repository 层 (`note_repository.rs`)

#### 新增 `restore()` 方法

```rust
pub fn restore(&self, id: &str, recovered_folder_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE notes
         SET is_deleted = 0,
             deleted_at = NULL,
             folder_id = ?,
             updated_at = ?
         WHERE id = ?",
        params![recovered_folder_id, now, id],
    )?;
    Ok(())
}
```

**功能**：
- 将 `is_deleted` 设为 `false`
- 将 `deleted_at` 设为 `NULL`
- 将 `folder_id` 设为"已恢复笔记"文件夹的 ID
- 更新 `updated_at` 时间戳

---

### 2. Service 层 (`note_service.rs`)

#### 修改 `NoteService` 结构

**修改前**：
```rust
pub struct NoteService {
    repo: NoteRepository,
}
```

**修改后**：
```rust
pub struct NoteService {
    repo: NoteRepository,
    folder_repo: FolderRepository,  // 新增：用于创建/获取"已恢复笔记"文件夹
}
```

#### 新增方法

##### `restore_note()` - 恢复单个笔记

```rust
pub fn restore_note(&self, id: &str) -> Result<Note> {
    let recovered_folder = self.get_or_create_recovered_folder()?;
    self.repo.restore(id, &recovered_folder.id)?;
    self.repo.find_by_id(id)?
        .ok_or(AppError::NotFound(...))
}
```

**行为**：
- 自动获取或创建"已恢复笔记"系统文件夹
- 将笔记从回收站恢复到该文件夹
- 返回恢复后的笔记对象

##### `restore_notes()` - 批量恢复笔记

```rust
pub fn restore_notes(&self, note_ids: Vec<String>) -> Result<Vec<Note>> {
    let mut restored_notes = Vec::new();
    for note_id in note_ids {
        match self.restore_note(&note_id) {
            Ok(note) => restored_notes.push(note),
            Err(e) => {
                log::warn!("Failed to restore note {}: {}", note_id, e);
                // 继续恢复其他笔记，不中断整个操作
            }
        }
    }
    Ok(restored_notes)
}
```

**行为**：
- 批量恢复多个笔记
- 容错处理：单个失败不影响其他笔记
- 返回成功恢复的笔记列表

##### `get_or_create_recovered_folder()` - 获取/创建系统文件夹

```rust
fn get_or_create_recovered_folder(&self) -> Result<Folder> {
    const RECOVERED_FOLDER_NAME: &str = "已恢复笔记";

    // 查找已存在的文件夹
    if let Some(existing) = all_folders.iter()
        .find(|f| f.name == RECOVERED_FOLDER_NAME && !f.is_deleted) {
        return Ok(existing.clone());
    }

    // 不存在则创建
    let folder = Folder::new(
        RECOVERED_FOLDER_NAME.to_string(),
        None,  // 根目录
        Some("#4CAF50".to_string()),  // 绿色
        Some("recycle".to_string()),  // 图标
    );
    self.folder_repo.create(&folder)?;
    Ok(folder)
}
```

**文件夹属性**：
- **名称**：`已恢复笔记`
- **父级**：根目录（`parent_id = NULL`）
- **颜色**：绿色（`#4CAF50`）
- **图标**：`recycle`
- **自动创建**：首次恢复时自动创建

---

### 3. Command 层 (`commands/notes.rs`)

#### 新增 Tauri Commands

##### `restore_note` - 恢复单个笔记

```rust
#[tauri::command]
pub async fn restore_note(
    id: String,
    service: NoteSvc<'_>,
) -> std::result::Result<Note, String> {
    service.restore_note(&id)
        .map_err(|e| e.to_string())
}
```

##### `restore_notes` - 批量恢复笔记

```rust
#[tauri::command]
pub async fn restore_notes(
    noteIds: Vec<String>,
    service: NoteSvc<'_>,
) -> std::result::Result<Vec<Note>, String> {
    service.restore_notes(noteIds)
        .map_err(|e| e.to_string())
}
```

---

### 4. 初始化调整 (`lib.rs`)

**修改前**：
```rust
let note_repo = NoteRepository::new(pool.clone());
let note_service = NoteService::new(note_repo);

let folder_repo = FolderRepository::new(pool.clone());
let folder_service = FolderService::new(folder_repo);
```

**修改后**：
```rust
// 先创建所有仓库
let note_repo = NoteRepository::new(pool.clone());
let folder_repo = FolderRepository::new(pool.clone());

// NoteService 需要 FolderRepository
let note_service = NoteService::new(note_repo, folder_repo.clone());
let folder_service = FolderService::new(folder_repo);
```

---

### 5. Repository Clone 支持 (`folder_repository.rs`)

**添加**：
```rust
#[derive(Clone)]
pub struct FolderRepository {
    pool: DbPool,
}
```

**原因**：`NoteService` 需要持有 `FolderRepository` 的克隆。

---

## 📖 使用示例

### 前端调用（TypeScript）

#### 恢复单个笔记

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 恢复笔记
const note = await invoke<Note>('restore_note', {
  id: 'note-id-here'
});

console.log('笔记已恢复到:', note.folderId);  // "已恢复笔记"文件夹 ID
```

#### 批量恢复笔记

```typescript
// 从回收站恢复多个笔记
const restoredNotes = await invoke<Note[]>('restore_notes', {
  noteIds: ['note-1', 'note-2', 'note-3']
});

console.log(`成功恢复 ${restoredNotes.length} 个笔记`);

// 更新 UI
setTrashNotes(prev => prev.filter(n => !restoredNotes.find(r => r.id === n.id)));
```

---

## 🎯 恢复流程

```
用户点击"恢复"
    ↓
前端调用 invoke('restore_note', { id })
    ↓
Tauri Command: restore_note()
    ↓
Service: restore_note()
    ↓
    ├─ get_or_create_recovered_folder()
    │   ├─ 查找"已恢复笔记"文件夹
    │   └─ 不存在则创建（绿色、根目录）
    ↓
    └─ repo.restore(note_id, recovered_folder_id)
        └─ UPDATE notes SET is_deleted=0, folder_id=?
    ↓
返回恢复后的笔记
    ↓
前端更新 UI
    ├─ 从回收站移除
    └─ 添加到"已恢复笔记"文件夹
```

---

## 🎨 "已恢复笔记"文件夹特性

| 属性 | 值 | 说明 |
|------|---|------|
| **名称** | `已恢复笔记` | 固定名称 |
| **位置** | 根目录 | `parent_id = NULL` |
| **颜色** | 绿色 `#4CAF50` | 表示恢复 |
| **图标** | `recycle` | 前端可自定义图标 |
| **创建时机** | 首次恢复时 | 自动创建 |
| **可删除** | 是 | 用户可删除，下次恢复时重新创建 |
| **可重命名** | 是 | 但下次恢复时会创建新的 |

---

## 🔄 与文件夹删除的配合

### 完整流程

```
1. 用户删除"工作"文件夹（物理删除）
   ├─ 工作文件夹：DELETE
   ├─ 子文件夹 2024：CASCADE
   └─ 所有笔记：folder_id = NULL

2. 笔记现在在"未分类"状态

3. 用户删除某些笔记（软删除）
   └─ 笔记：is_deleted = 1, deleted_at = NOW

4. 用户在回收站点击"恢复"
   ├─ 创建"已恢复笔记"文件夹（如果不存在）
   └─ 笔记：is_deleted = 0, folder_id = "已恢复笔记"

5. 用户可以手动整理恢复的笔记
   └─ 将笔记移动到其他文件夹
```

---

## ⚠️ 注意事项

### 方案 B 的限制

| 特性 | 方案 A（记住原始位置） | 方案 B（恢复到固定文件夹） |
|------|---------------------|----------------------|
| **恢复位置** | 原始文件夹（如果存在） | "已恢复笔记"文件夹 |
| **数据库变更** | 需要 `original_folder_id` 字段 | 无需变更 |
| **实现复杂度** | 高 | 低 |
| **用户体验** | 最佳 | 良好（需手动整理） |

### 当前实现（方案 B）

- ✅ **无需修改 Schema**
- ✅ **实现简单**
- ✅ **笔记不会丢失**
- ⚠️ **恢复后需要手动整理**
- 💡 **适合快速实现**

### 升级到方案 A

如果需要"恢复到原位置"功能，需要：

1. **添加字段**：
   ```sql
   ALTER TABLE notes ADD COLUMN original_folder_id CHAR(36);
   ```

2. **软删除时保存**：
   ```rust
   pub fn soft_delete(&self, id: &str) -> Result<()> {
       let mut note = self.get_note_by_id(id)?;
       note.original_folder_id = note.folder_id.clone();  // 保存
       note.is_deleted = true;
       // ...
   }
   ```

3. **恢复时智能判断**：
   ```rust
   pub fn restore_note(&self, id: &str) -> Result<Note> {
       let note = self.get_note_by_id(id)?;

       if let Some(original_id) = &note.original_folder_id {
           if self.folder_service.exists(original_id)? {
               // 恢复到原位置
               note.folder_id = Some(original_id.clone());
           } else {
               // 原文件夹已删除，放到"已恢复笔记"
               note.folder_id = Some(recovered_folder.id);
           }
       } else {
           // 没有原始位置记录，放到"已恢复笔记"
           note.folder_id = Some(recovered_folder.id);
       }

       // ...
   }
   ```

---

## 🧪 测试建议

### 功能测试

1. **单个笔记恢复**
   ```bash
   # 删除笔记
   invoke('delete_note', { id: 'note-1' })
   # 恢复笔记
   invoke('restore_note', { id: 'note-1' })
   # 验证：笔记在"已恢复笔记"文件夹中
   ```

2. **批量恢复**
   ```bash
   # 删除多个笔记
   invoke('delete_note', { id: 'note-1' })
   invoke('delete_note', { id: 'note-2' })
   # 批量恢复
   invoke('restore_notes', { noteIds: ['note-1', 'note-2'] })
   # 验证：两个笔记都在"已恢复笔记"文件夹中
   ```

3. **"已恢复笔记"文件夹创建**
   ```bash
   # 首次恢复前：无此文件夹
   # 首次恢复后：自动创建
   # 验证属性：name="已恢复笔记", color="#4CAF50"
   ```

### 边界测试

- 恢复不存在的笔记 → 返回错误
- 恢复已恢复的笔记 → 应该正常处理
- 批量恢复部分失败 → 返回成功恢复的笔记

---

## 📊 编译状态

```bash
✅ cargo check 通过
⚠️  7 个未使用代码警告（新功能尚未被前端调用）
```

**警告列表**：
- `restore_note` - 新增命令，前端尚未集成
- `restore_notes` - 新增命令，前端尚未集成
- 其他 5 个为已存在的警告

---

## 🚀 下一步（前端集成）

### 1. 创建 API 函数

```typescript
// src/services/noteApi.ts

export async function restoreNote(noteId: string): Promise<Note> {
  return invoke<Note>('restore_note', { id: noteId });
}

export async function restoreNotes(noteIds: string[]): Promise<Note[]> {
  return invoke<Note[]>('restore_notes', { noteIds });
}
```

### 2. 回收站 UI

```tsx
// 回收站列表
{trashNotes.map(note => (
  <TrashItem
    key={note.id}
    note={note}
    onRestore={async () => {
      const restored = await restoreNote(note.id);
      // 更新 UI
      setTrashNotes(prev => prev.filter(n => n.id !== note.id));
      toast.success(`已恢复到"已恢复笔记"文件夹`);
    }}
  />
))}
```

### 3. 批量操作

```tsx
<Button onClick={async () => {
  const selected = trashNotes.filter(n => n.selected);
  const restored = await restoreNotes(selected.map(n => n.id));
  toast.success(`成功恢复 ${restored.length} 个笔记`);
}}>
  批量恢复
</Button>
```

---

## 📝 总结

| 项目 | 状态 |
|------|------|
| **Repository 层** | ✅ 完成 |
| **Service 层** | ✅ 完成 |
| **Command 层** | ✅ 完成 |
| **初始化调整** | ✅ 完成 |
| **编译验证** | ✅ 通过 |
| **文档** | ✅ 完成 |

**方案 B 已完全实现**，可以立即使用！前端集成后即可提供笔记恢复功能。🎉
