# 数据库初始化说明

## 📋 更新内容

**版本**: 2.2.0

### 本次更新包括：

1. ✅ **sync_locks 表增强**
   - 添加 `workspace_id` 字段，支持工作空间级别的并发控制
   - 添加复合索引 `idx_workspace_lock` 以优化查询性能
   - 防止同一用户的不同工作空间同时同步导致数据混淆

2. ✅ **工作空间支持**
   - 所有业务表（notes, folders, tags, note_tags, note_snapshots）都包含 workspace_id 字段
   - 支持数据按工作空间隔离

3. ✅ **多账号设备支持**
   - devices 表使用复合主键 `(user_id, id)`
   - 允许同一物理设备被多个账号使用

## 🚀 执行步骤

### 方法 1：命令行（推荐）

```bash
# Windows (使用 Git Bash 或 PowerShell)
cd D:\VScodeProject\note\note-sync-server
mysql -u root -p < sql/init.sql

# 或者（如果需要指定数据库）
mysql -u root -p notes-sync < sql/init.sql
```

### 方法 2：MySQL 客户端

```bash
# 1. 登录 MySQL
mysql -u root -p

# 2. 执行初始化脚本
source D:/VScodeProject/note/note-sync-server/sql/init.sql

# 或者
source D:\\VScodeProject\\note\\note-sync-server\\sql\\init.sql
```

### 方法 3：图形化工具（MySQL Workbench, phpMyAdmin 等）

1. 打开 MySQL Workbench 或其他图形化工具
2. 连接到 MySQL 服务器
3. 打开 `sql/init.sql` 文件
4. 执行整个脚本（点击闪电图标 ⚡）

## ✅ 验证安装

执行完成后，检查以下内容：

### 1. 检查 sync_locks 表结构

```sql
DESCRIBE sync_locks;
```

应该包含 `workspace_id` 字段：

| Field | Type | Null | Key | Extra |
|-------|------|------|-----|-------|
| id | char(36) | NO | PRI | |
| user_id | varchar(10) | NO | MUL | |
| device_id | varchar(255) | NO | | |
| **workspace_id** | **varchar(36)** | **YES** | **MUL** | |
| acquired_at | bigint | NO | | |
| expires_at | bigint | NO | | |

### 2. 检查索引

```sql
SHOW INDEX FROM sync_locks;
```

应该包含：
- `PRIMARY` on `id`
- `idx_user_device_lock` on `(user_id, device_id)`
- `idx_workspace_lock` on `(user_id, device_id, workspace_id)`

### 3. 检查其他表的工作空间支持

```sql
-- notes 表应该有 workspace_id 字段
SHOW COLUMNS FROM notes LIKE 'workspace_id';

-- folders 表应该有 workspace_id 字段
SHOW COLUMNS FROM folders LIKE 'workspace_id';

-- tags 表应该有 workspace_id 字段
SHOW COLUMNS FROM tags LIKE 'workspace_id';
```

## 🗑️ 删除旧数据库（可选）

如果要完全重新开始：

```bash
mysql -u root -p -e "DROP DATABASE IF EXISTS \`notes-sync\`; CREATE DATABASE \`notes-sync\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
mysql -u root -p notes-sync < sql/init.sql
```

## ⚠️ 注意事项

1. **备份数据**：执行前请备份现有数据库（如果有重要数据）
   ```bash
   mysqldump -u root -p notes-sync > backup_$(date +%Y%m%d_%H%M%S).sql
   ```

2. **字符集**：确保使用 `utf8mb4` 字符集以支持 emoji 和特殊字符

3. **权限**：确保 MySQL 用户有 CREATE DATABASE 和 CREATE TABLE 权限

## 📚 相关文件

- **主脚本**: `sql/init.sql`
- **迁移脚本**: `sql/migrations/` 目录（仅用于增量更新）
- **数据库版本**: 2.2.0

## 🐛 故障排除

### 错误 1：Access denied for user

```
ERROR 1045 (28000): Access denied for user 'root'@'localhost'
```

**解决**: 检查用户名和密码，确保有足够的权限

### 错误 2：Unknown database

```
ERROR 1049 (42000): Unknown database 'notes-sync'
```

**解决**: init.sql 会自动创建数据库，确保执行了完整的脚本

### 错误 3：Table already exists

```
ERROR 1050 (42S01): Table 'xxx' already exists
```

**解决**: 删除旧数据库或表，或使用 `DROP DATABASE IF EXISTS`

## 📞 获取帮助

如果遇到问题：
1. 检查 MySQL 错误日志
2. 确认 MySQL 服务正在运行
3. 验证文件路径是否正确
4. 查看完整的 SQL 错误信息
