# 文件夹删除逻辑修复

## 🎯 问题

**原有行为**：
- 删除文件夹时，子文件夹被物理删除（CASCADE）
- 但笔记的 `folder_id` 被置为 `NULL`（ON DELETE SET NULL）
- 笔记变成"未分类"状态，散落在根目录

**问题**：
- ❌ 笔记无法通过回收站恢复
- ❌ 大量"未分类"笔记难以管理
- ❌ 删除操作不可逆（虽然笔记还在，但找不到原来的分类）

---

## ✅ 修复后行为

**新行为**：
- 删除文件夹时，**软删除该文件夹及所有子文件夹下的笔记**（is_deleted = 1）
- 这些笔记进入回收站，可以被恢复到"已恢复笔记"文件夹
- 文件夹本身仍然是物理删除（CASCADE）

**优势**：
- ✅ 笔记可恢复：通过回收站恢复功能
- ✅ 操作可逆：误删可以找回
- ✅ 用户体验更好：符合直觉的删除行为

---

## 🔧 技术实现

### 修改文件

`src-tauri/src/database/repositories/folder_repository.rs`

### 核心逻辑

```rust
pub fn hard_delete(&self, id: &str) -> Result<()> {
    // 第一步：获取所有子孙文件夹的 ID（包括自己）
    let folder_ids = self.get_all_descendant_ids(id)?;

    // 第二步：软删除这些文件夹下的所有笔记
    let now = chrono::Utc::now().timestamp();
    if !folder_ids.is_empty() {
        let placeholders = folder_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "UPDATE notes SET is_deleted = 1, deleted_at = ? WHERE folder_id IN ({})",
            placeholders
        );

        let mut params_list: Vec<&dyn r2d2_sqlite::rusqlite::ToSql> = vec![&now];
        for folder_id in &folder_ids {
            params_list.push(folder_id);
        }

        conn.execute(&sql, params_list.as_slice())?;
        log::debug!("Soft deleted notes in folders: {:?}", folder_ids);
    }

    // 第三步：物理删除文件夹（级联删除子文件夹）
    conn.execute("DELETE FROM folders WHERE id = ?", params![id])?;

    log::debug!("Folder hard deleted: {} (cascade to children, notes moved to trash)", id);
    Ok(())
}
```

### 辅助方法

#### 1. `get_all_descendant_ids()` - 获取所有子孙文件夹 ID

```rust
fn get_all_descendant_ids(&self, id: &str) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    self.collect_descendant_ids_recursive(id, &mut ids)?;
    Ok(ids)
}
```

#### 2. `collect_descendant_ids_recursive()` - 递归收集文件夹 ID

```rust
fn collect_descendant_ids_recursive(&self, parent_id: &str, ids: &mut Vec<String>) -> Result<()> {
    // 添加自己
    ids.push(parent_id.to_string());

    // 查找直接子文件夹
    let children = self.find_children(Some(parent_id))?;

    // 递归处理每个子文件夹
    for child in children {
        self.collect_descendant_ids_recursive(&child.id, ids)?;
    }

    Ok(())
}
```

---

## 📊 行为对比

### 示例场景

删除前：
```
📁 工作文件夹
  ├─ 📄 工作笔记1
  ├─ 📄 工作笔记2
  └─ 📁 2024项目
      └─ 📄 项目笔记
```

#### 修复前（旧行为）

删除后：
```
📄 工作笔记1（未分类）
📄 工作笔记2（未分类）
📄 项目笔记（未分类）
```

**问题**：
- ❌ 笔记散落在根目录
- ❌ 无法通过回收站恢复
- ❌ 无法恢复原来的文件夹结构

#### 修复后（新行为）

删除后：
```
❌ 工作文件夹（已物理删除）
❌ 2024项目（已物理删除）
📄 工作笔记1（回收站）
📄 工作笔记2（回收站）
📄 项目笔记（回收站）
```

**优势**：
- ✅ 笔记进入回收站
- ✅ 可以恢复到"已恢复笔记"文件夹
- ✅ 不会丢失笔记内容

---

## 🔄 恢复流程

删除文件夹后，用户可以：

1. **打开回收站**
   - 点击侧边栏的"回收站"图标
   - 看到所有被软删除的笔记

2. **恢复笔记**
   - 单个恢复：点击笔记的"恢复"按钮
   - 批量恢复：勾选多个笔记，点击"恢复选中"

3. **恢复后**
   - 笔记移动到"已恢复笔记"文件夹
   - 可以手动重新组织到其他文件夹

---

## ⚠️ 注意事项

### 数据库外键约束

**现有约束**（schema.rs:29）：
```sql
FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
```

**为什么不修改这个约束？**

虽然 `ON DELETE SET NULL` 仍然存在，但我们**在代码层面先软删除笔记**，这样：

1. 笔记已经被标记为 `is_deleted = 1`
2. 外键约束触发时，只是将已删除笔记的 `folder_id` 设为 NULL
3. 笔记仍然在回收站中，可以被恢复

**性能考虑**：
- 先软删除笔记：1 条 UPDATE 语句（批量）
- 再删除文件夹：1 条 DELETE 语句（自动级联）
- 总共 2 条 SQL，性能优秀

### 递归查询性能

对于深层嵌套的文件夹树：

- **优点**：利用现有的 `find_children()` 方法，代码简洁
- **缺点**：递归查询，如果文件夹非常多可能稍慢

**优化方案**（如果需要）：
```sql
-- 使用 SQLite 递归 CTE（一次查询获取所有子孙）
WITH RECURSIVE descendant_folders AS (
    SELECT id FROM folders WHERE id = ?
    UNION ALL
    SELECT f.id FROM folders f
    INNER JOIN descendant_folders df ON f.parent_id = df.id
)
SELECT id FROM descendant_folders;
```

---

## 🧪 测试验证

### 测试场景

1. **单层文件夹**
   ```
   📁 测试文件夹
     ├─ 📄 笔记1
     └─ 📄 笔记2
   ```
   删除后：笔记1、2 都在回收站

2. **多层嵌套文件夹**
   ```
   📁 工作文件夹
     ├─ 📄 工作笔记
     └─ 📁 2024项目
         ├─ 📄 项目笔记1
         └─ 📁 子项目
             └─ 📄 子项目笔记
   ```
   删除后：所有 3 篇笔记都在回收站

3. **空文件夹**
   ```
   📁 空文件夹
   ```
   删除后：无笔记进入回收站（正常）

4. **文件夹只有子文件夹，无直接笔记**
   ```
   📁 父文件夹
     └─ 📁 子文件夹
         └─ 📄 笔记
   ```
   删除后：笔记在回收站

### 验证命令

```bash
# 编译检查
cd src-tauri && cargo check

# 运行应用
pnpm tauri:dev

# 测试步骤
# 1. 创建测试文件夹和笔记
# 2. 删除文件夹
# 3. 检查回收站是否有笔记
# 4. 恢复笔记
# 5. 确认笔记恢复成功
```

---

## 📈 性能影响

### 修改前
```
删除文件夹：1 条 DELETE 语句
副作用：notes.folder_id = NULL（外键自动触发）
```

### 修改后
```
1. 查询子孙文件夹：N 条 SELECT（递归）
2. 软删除笔记：1 条 UPDATE（批量）
3. 删除文件夹：1 条 DELETE（级联）
```

**性能估算**：
- 假设文件夹树深度为 5 层
- 递归查询：约 5 条 SELECT
- 更新笔记：1 条 UPDATE（假设有 10 篇笔记）
- 总耗时：< 10ms（SQLite 本地数据库）

**结论**：性能影响可忽略，用户体验提升显著 ✅

---

## 🎉 总结

### 修改内容
- ✅ 修改 `FolderRepository::hard_delete()` 方法
- ✅ 添加递归获取子孙文件夹 ID 的逻辑
- ✅ 在删除文件夹前，先软删除所有相关笔记

### 用户体验
- ✅ 删除文件夹时，笔记进入回收站
- ✅ 可以通过回收站恢复笔记
- ✅ 不会丢失笔记内容

### 技术债务
- ⚠️ 外键约束仍然是 `ON DELETE SET NULL`（但不影响功能）
- 💡 未来可考虑使用 SQLite 递归 CTE 优化查询

---

**修复完成！现在删除文件夹时，笔记会进入回收站，可以被恢复。** 🚀
