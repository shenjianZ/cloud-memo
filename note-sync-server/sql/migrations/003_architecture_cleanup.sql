-- ============================================
-- 云同步系统架构清理迁移脚本
-- 版本: 003
-- 说明: 移除客户端同步字段，添加设备追踪字段
-- ============================================

USE `notes-sync`;

-- ===== 步骤 1：添加设备追踪字段 =====

-- notes 表添加设备追踪字段
ALTER TABLE notes
ADD COLUMN IF NOT EXISTS device_id VARCHAR(64) DEFAULT NULL COMMENT '最后修改的设备ID',
ADD COLUMN IF NOT EXISTS updated_by_device VARCHAR(255) DEFAULT NULL COMMENT '设备描述';

ALTER TABLE notes ADD INDEX IF NOT EXISTS idx_device_id (device_id);

-- folders 表添加设备追踪字段
ALTER TABLE folders
ADD COLUMN IF NOT EXISTS device_id VARCHAR(64) DEFAULT NULL COMMENT '最后修改的设备ID',
ADD COLUMN IF NOT EXISTS updated_by_device VARCHAR(255) DEFAULT NULL COMMENT '设备描述';

ALTER TABLE folders ADD INDEX IF NOT EXISTS idx_device_id (device_id);

-- tags 表添加设备追踪字段
ALTER TABLE tags
ADD COLUMN IF NOT EXISTS device_id VARCHAR(64) DEFAULT NULL COMMENT '最后修改的设备ID',
ADD COLUMN IF NOT EXISTS updated_by_device VARCHAR(255) DEFAULT NULL COMMENT '设备描述';

ALTER TABLE tags ADD INDEX IF NOT EXISTS idx_device_id (device_id);

-- note_versions 表添加设备追踪字段
ALTER TABLE note_versions
ADD COLUMN IF NOT EXISTS device_id VARCHAR(64) DEFAULT NULL COMMENT '创建快照的设备ID';

ALTER TABLE note_versions ADD INDEX IF NOT EXISTS idx_device_id (device_id);

-- ===== 步骤 2：refresh_tokens 表安全增强 =====

ALTER TABLE refresh_tokens
ADD COLUMN IF NOT EXISTS revoked BOOLEAN DEFAULT FALSE COMMENT '是否已撤销',
ADD COLUMN IF NOT EXISTS revoked_at BIGINT DEFAULT NULL COMMENT '撤销时间戳';

ALTER TABLE refresh_tokens ADD INDEX IF NOT EXISTS idx_revoked (revoked);

-- ===== 步骤 3：sync_locks 表约束完善 =====

-- 删除旧索引（如果存在）
ALTER TABLE sync_locks DROP INDEX IF EXISTS idx_user_lock;

-- 添加唯一约束：每个用户同时只能有一个锁
ALTER TABLE sync_locks
ADD UNIQUE INDEX idx_user_lock (user_id);

-- ===== 步骤 4：移除客户端同步字段 =====

-- notes 表移除客户端字段
ALTER TABLE notes DROP COLUMN IF EXISTS is_dirty;
ALTER TABLE notes DROP COLUMN IF EXISTS last_synced_at;
ALTER TABLE notes DROP INDEX IF EXISTS idx_notes_dirty;

-- folders 表移除客户端字段
ALTER TABLE folders DROP COLUMN IF EXISTS is_dirty;
ALTER TABLE folders DROP COLUMN IF EXISTS last_synced_at;
ALTER TABLE folders DROP INDEX IF EXISTS idx_folders_dirty;

-- tags 表移除客户端字段
ALTER TABLE tags DROP COLUMN IF EXISTS is_dirty;
ALTER TABLE tags DROP COLUMN IF EXISTS last_synced_at;

-- note_versions 表移除客户端字段
ALTER TABLE note_versions DROP COLUMN IF EXISTS is_dirty;
ALTER TABLE note_versions DROP COLUMN IF EXISTS last_synced_at;
ALTER TABLE note_versions DROP INDEX IF EXISTS idx_versions_dirty;

-- 验证迁移
SELECT '✅ 架构清理迁移完成' AS status;

-- 显示更新后的表结构（用于验证）
SELECT '===== notes 表结构 =====' AS info;
DESCRIBE notes;

SELECT '===== folders 表结构 =====' AS info;
DESCRIBE folders;

SELECT '===== tags 表结构 =====' AS info;
DESCRIBE tags;

SELECT '===== note_versions 表结构 =====' AS info;
DESCRIBE note_versions;

SELECT '===== refresh_tokens 表结构 =====' AS info;
DESCRIBE refresh_tokens;

SELECT '===== sync_locks 表结构 =====' AS info;
DESCRIBE sync_locks;
