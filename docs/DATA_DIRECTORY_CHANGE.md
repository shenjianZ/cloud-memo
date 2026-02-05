# 数据目录变更文档

## 📁 数据目录位置

### 修改前
```
Windows: C:\Users\<用户名>\AppData\Roaming\markdown-notes
macOS: ~/Library/Application Support/markdown-notes
Linux: ~/.config/markdown-notes
```

### 修改后
```
所有平台: ~\.notes-data

Windows: C:\Users\<用户名>\.notes-data
macOS: /Users/<用户名>/.notes-data
Linux: /home/<用户名>/.notes-data
```

---

## 🎯 变更原因

1. **简化路径**：更短、更易记的目录名称
2. **用户友好**：直接在家目录下，用户容易找到
3. **跨平台一致**：所有平台使用相同的相对路径
4. **易于备份**：用户可以轻松备份整个 `.notes-data` 文件夹

---

## 📂 目录结构

```
~/.notes-data/
├── notes.db              # SQLite 数据库（笔记、文件夹、标签）
├── keybindings.json      # 快捷键配置
└── (其他可能的数据文件)
```

---

## 💻 实现细节

### Rust 代码

**文件**: `src-tauri/src/lib.rs`

```rust
.setup(|app| {
    // 使用用户家目录下的 .notes-data 文件夹
    let home_dir = dirs::home_dir()
        .expect("Failed to get home directory");

    let app_data_dir = home_dir.join(".notes-data");

    // 创建目录（如果不存在）
    std::fs::create_dir_all(&app_data_dir)
        .expect("Failed to create .notes-data directory");

    let db_path = app_data_dir.join("notes.db");

    log::info!("Initializing database at: {:?}", db_path);
    // ...
})
```

### 依赖

**Cargo.toml**:
```toml
[dependencies]
dirs = "5"
```

`dirs` crate 提供跨平台的目录路径获取功能。

---

## ✅ 行为说明

### 首次运行
1. 获取用户家目录（`~`）
2. 创建 `~/.notes-data` 文件夹（如果不存在）
3. 在文件夹中创建 `notes.db` 数据库
4. 在文件夹中创建 `keybindings.json`（如果需要）

### 后续运行
1. 检查 `~/.notes-data` 是否存在（应该已存在）
2. 使用现有的 `notes.db`
3. 正常加载配置和数据

---

## 🔍 数据迁移

### 从旧目录迁移到新目录

如果用户之前使用过旧版本应用，数据仍在旧目录中：

**Windows**:
```bash
# 旧位置
C:\Users\<用户名>\AppData\Roaming\markdown-notes

# 新位置
C:\Users\<用户名>\.notes-data
```

**迁移步骤**:
1. 关闭应用
2. 手动复制旧目录的所有文件到 `~/.notes-data`
3. 重新启动应用

**或者提供迁移工具**（未来功能）:
```rust
// 伪代码
if old_dir.exists() && !new_dir.exists() {
    std::fs::rename(old_dir, new_dir)?;
}
```

---

## 📊 路径示例

### Windows
```rust
home_dir = "C:\\Users\\张三"
app_data_dir = "C:\\Users\\张三\\.notes-data"
db_path = "C:\\Users\\张三\\.notes-data\\notes.db"
```

### macOS
```rust
home_dir = "/Users/zhangsan"
app_data_dir = "/Users/zhangsan/.notes-data"
db_path = "/Users/zhangsan/.notes-data/notes.db"
```

### Linux
```rust
home_dir = "/home/zhangsan"
app_data_dir = "/home/zhangsan/.notes-data"
db_path = "/home/zhangsan/.notes-data/notes.db"
```

---

## 🛠️ 故障排查

### 常见问题

#### Q: 找不到数据目录？
**A**: 检查以下位置：
- Windows: `C:\Users\<你的用户名>\.notes-data`
- macOS/Linux: `~/.notes-data`（在终端中 `ls -la ~` 查看）

#### Q: 权限错误？
**A**: 确保应用有权限访问家目录：
```bash
# macOS/Linux
chmod 755 ~/.notes-data

# Windows（以管理员身份运行）
# 通常是自动处理的
```

#### Q: 数据丢失？
**A**: 检查旧目录：
```bash
# 搜索旧数据
# Windows
dir "%APPDATA%\markdown-notes" /s /b

# macOS/Linux
find ~/Library/Application\ Support -name "notes.db"
```

---

## 🧪 验证方法

### 开发环境测试

```bash
# 1. 清理旧数据（可选）
# rm -rf ~/.notes-data  # macOS/Linux
# rmdir /s "%USERPROFILE%\.notes-data"  # Windows

# 2. 启动应用
pnpm tauri:dev

# 3. 检查日志
# 应该看到类似输出：
# [INFO] Initializing database at: "C:\Users\<用户名>\.notes-data\notes.db"

# 4. 创建测试笔记
# 验证数据是否保存在正确位置

# 5. 检查文件系统
# ls ~/.notes-data  # macOS/Linux
# dir "%USERPROFILE%\.notes-data"  # Windows
```

---

## 📝 注意事项

### ⚠️ Windows 特殊性

- 家目录：`C:\Users\<用户名>`
- 用户配置目录：`C:\Users\<用户名>\AppData\Roaming`
- **我们使用**: `C:\Users\<用户名>\.notes-data`（更简单）

### ⚠️ 隐藏文件夹

- `.notes-data` 是一个隐藏文件夹（以 `.` 开头）
- Windows: 需要启用"显示隐藏文件"才能在资源管理器中看到
- macOS/Linux: 默认隐藏，使用 `ls -a` 查看

### ⚠️ 备份建议

用户备份时只需备份 `.notes-data` 文件夹：
```bash
# 打包备份
tar czf notes-data-backup.tar.gz ~/.notes-data

# 或使用文件管理器直接复制
# Windows: 复制 C:\Users\<用户名>\.notes-data
# macOS/Linux: 复制 ~/.notes-data
```

---

## 🚀 编译验证

```bash
✅ cargo check 通过
✅ 数据目录路径正确
✅ 使用 dirs::home_dir() 跨平台兼容
✅ 自动创建目录
```

---

## 📚 相关代码

### 依赖
```toml
# src-tauri/Cargo.toml
[dependencies]
dirs = "5"
```

### 实现
```rust
// src-tauri/src/lib.rs
use dirs;

let home_dir = dirs::home_dir()
    .expect("Failed to get home directory");

let app_data_dir = home_dir.join(".notes-data");

std::fs::create_dir_all(&app_data_dir)
    .expect("Failed to create .notes-data directory");
```

---

## ✅ 总结

| 项目 | 说明 |
|------|------|
| **新位置** | `~/.notes-data` |
| **跨平台** | ✅ Windows/macOS/Linux |
| **自动创建** | ✅ 首次运行自动创建 |
| **易于查找** | ✅ 用户容易找到 |
| **易于备份** | ✅ 直接复制文件夹 |
| **编译状态** | ✅ 通过 |

**数据目录变更已完成！** 🎉
