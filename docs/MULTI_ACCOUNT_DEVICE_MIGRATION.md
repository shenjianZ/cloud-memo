# 多账号设备支持 - 迁移指南

## 📋 变更说明

本次修改允许**同一物理设备被多个账号使用**，解决了之前注册时出现的 `Duplicate entry` 错误。

### 修改内容

1. **数据库表结构变更**
   - 将 `devices` 表的主键从 `id`（device_id）改为复合主键 `(user_id, id)`
   - 添加了 `idx_device_id` 索引以优化查询性能

2. **业务逻辑优化**
   - 修改了 `device_service.rs` 的 `register_or_update` 方法
   - 添加了详细的注释说明多账号支持机制
   - 优化了日志输出

---

## 🚀 部署步骤

### 步骤 1：执行数据库迁移脚本

连接到你的 MySQL 服务器，执行迁移脚本：

```bash
# 方式 1：使用 MySQL 客户端
mysql -u your_username -p notes-sync < note-sync-server/sql/migrations/006_allow_multi_account_devices.sql

# 方式 2：登录 MySQL 后执行
mysql -u your_username -p
USE notes-sync;
source note-sync-server/sql/migrations/006_allow_multi_account_devices.sql
```

### 步骤 2：重启服务器

迁移完成后，重启 Tauri 同步服务器：

```bash
cd note-sync-server
cargo run --release
```

### 步骤 3：测试注册流程

1. 清空客户端本地数据库（可选，如果要测试新用户注册）：
   ```bash
   # 删除客户端数据库文件
   rm src-tauri/note.db
   ```

2. 启动 Tauri 应用并尝试注册新账号

3. 验证注册成功后，再次注册另一个账号（使用同一设备）

---

## ✅ 验证迁移结果

### 检查表结构

```sql
USE notes-sync;
SHOW COLUMNS FROM devices;
SHOW INDEX FROM devices WHERE Key_name = 'PRIMARY';
```

**预期结果**：
- 主键应为 `PRIMARY KEY (user_id, id)`
- 存在索引 `idx_device_id`（可选）

### 测试多账号注册

在应用中：
1. 使用账号 A 注册 → ✅ 成功
2. 使用账号 B 注册（同一设备）→ ✅ 成功
3. 账号 A 再次登录 → ✅ 成功（更新设备的 last_seen_at）

### 检查数据库数据

```sql
-- 查看所有设备记录
SELECT
    user_id,
    id AS device_id,
    device_name,
    device_type,
    FROM_UNIXTIME(created_at) AS created_at,
    FROM_UNIXTIME(last_seen_at) AS last_seen_at
FROM devices
ORDER BY created_at DESC;

-- 验证同一设备是否关联多个用户
SELECT
    id AS device_id,
    COUNT(DISTINCT user_id) AS user_count,
    GROUP_CONCAT(user_id) AS users
FROM devices
GROUP BY id
HAVING user_count > 1;
```

---

## 📊 数据变化示例

### 迁移前（单账号设备）

| id                        | user_id | device_name   |
|---------------------------|---------|---------------|
| desktop-windows-xxx       | 001     | 我的Windows   |

**问题**：账号 002 尝试使用同一设备注册 → ❌ 主键冲突

### 迁移后（多账号设备）

| id                        | user_id | device_name   |
|---------------------------|---------|---------------|
| desktop-windows-xxx       | 001     | 我的Windows   |
| desktop-windows-xxx       | 002     | 我的Windows   |

**结果**：两个账号都可以使用同一设备 → ✅ 成功

---

## ⚠️ 注意事项

### 1. 数据备份

在执行迁移前，**务必备份数据库**：

```bash
mysqldump -u your_username -p notes-sync > backup_$(date +%Y%m%d_%H%M%S).sql
```

### 2. 向后兼容性

- ✅ 完全向后兼容：现有设备数据不受影响
- ✅ 客户端代码无需修改
- ⚠️ 服务器必须重启以应用新的 Rust 代码

### 3. 性能影响

- 查询性能：几乎无影响（复合主键查询效率高）
- 插入性能：略微提升（减少了唯一性检查冲突）
- 存储空间：可能略微增加（如果多个账号使用同一设备）

### 4. 安全性

- 设备绑定机制仍然有效（每个用户-设备组合独立）
- 撤销设备功能正常工作（只撤销特定用户的设备访问权限）

---

## 🔧 回滚方案

如果需要回滚到单账号设备模式：

1. **停止服务器**

2. **回滚数据库**（从备份恢复）：
   ```bash
   mysql -u your_username -p notes-sync < backup_YYYYMMDD_HHMMSS.sql
   ```

3. **回滚代码**：
   ```bash
   git checkout HEAD -- note-sync-server/src/services/device_service.rs
   ```

4. **重启服务器**

---

## 📝 技术细节

### 复合主键设计

```sql
-- 之前
PRIMARY KEY (id)  -- device_id 全局唯一

-- 之后
PRIMARY KEY (user_id, id)  -- 同一用户下 device_id 唯一
```

### 查询逻辑

```rust
// 查询时使用复合主键
WHERE user_id = ? AND id = ?

// 更新时也使用复合主键
UPDATE devices SET ... WHERE user_id = ? AND id = ?
```

### 业务含义

- **物理设备**：由 `device_id` 标识（如 `desktop-windows-xxx`）
- **逻辑设备**：由 `(user_id, device_id)` 组合标识
- **一个物理设备可以被多个逻辑账号使用**

---

## ❓ 常见问题

### Q1: 迁移会影响现有用户吗？

**A**: 不会。现有用户的设备记录会自动适配新的主键结构，无需任何额外操作。

### Q2: 客户端需要更新吗？

**A**: 不需要。客户端代码完全兼容，无需修改。

### Q3: 如何查看某个设备被哪些账号使用？

**A**:
```sql
SELECT user_id, device_name, created_at
FROM devices
WHERE id = 'desktop-windows-xxx';
```

### Q4: 同一设备多账号会有安全问题吗？

**A**: 不会。每个账号的数据完全隔离，设备只是访问凭证的载体，不会共享数据。

---

## 📞 支持

如果遇到问题，请检查：

1. 数据库迁移是否成功执行
2. 服务器日志是否有错误信息
3. MySQL 错误日志（如果有数据库错误）

---

**迁移日期**: 2026-02-08
**版本**: v0.6.0
**作者**: Claude Code
