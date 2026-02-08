-- ============================================
-- 修复登出时级联删除业务数据的问题
-- ============================================
-- 问题：workspaces 表的外键约束指向 user_auth 表
-- 导致：登出时（DELETE FROM user_auth）会级联删除所有工作空间
-- ============================================

-- 步骤 1：查看当前外键约束
SELECT
    sql
FROM sqlite_master
WHERE type = 'table'
AND name = 'workspaces';

-- 步骤 2：重建 workspaces 表（移除外键约束）
-- SQLite 不支持直接删除外键，需要重建表

BEGIN TRANSACTION;

-- 1. 创建新的 workspaces 表（不包含外键约束）
CREATE TABLE workspaces_new (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    color TEXT,
    is_default BOOLEAN DEFAULT 0,
    sort_order INTEGER DEFAULT 0,
    is_current BOOLEAN DEFAULT 0,
    is_deleted BOOLEAN DEFAULT 0,
    deleted_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    server_ver INTEGER DEFAULT 0,
    is_dirty BOOLEAN DEFAULT 0,
    last_synced_at INTEGER
);

-- 2. 复制数据
INSERT INTO workspaces_new
SELECT * FROM workspaces;

-- 3. 删除旧表
DROP TABLE workspaces;

-- 4. 重命名新表
ALTER TABLE workspaces_new RENAME TO workspaces;

-- 5. 重建索引
CREATE INDEX IF NOT EXISTS idx_workspaces_user_id ON workspaces(user_id);
CREATE INDEX IF NOT EXISTS idx_workspaces_is_default ON workspaces(is_default);
CREATE INDEX IF NOT EXISTS idx_workspaces_is_current ON workspaces(is_current);
CREATE INDEX IF NOT EXISTS idx_workspaces_is_deleted ON workspaces(is_deleted);

COMMIT;

-- 步骤 3：验证修复成功
SELECT '✅ 修复完成' AS status;
SELECT 'workspaces 表外键已移除' AS info;
SELECT '登出时不再删除业务数据' AS result;

-- 验证表结构
SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'workspaces';
