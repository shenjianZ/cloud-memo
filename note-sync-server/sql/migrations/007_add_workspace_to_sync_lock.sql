-- 迁移 007：为 sync_locks 表添加 workspace_id 字段
--
-- 目的：增强同步锁机制，支持基于工作空间的并发控制
-- 问题：当前同步锁只基于 user_id + device_id，无法防止同一用户的不同工作空间同时同步
-- 解决方案：添加 workspace_id 字段，使同步锁更细粒度

-- 1. 添加 workspace_id 字段
ALTER TABLE sync_locks ADD COLUMN workspace_id VARCHAR(36) NULL AFTER device_id;

-- 2. 添加复合索引以提高查询性能
-- 这个索引支持按 user_id + device_id + workspace_id 快速查询锁
CREATE INDEX idx_sync_locks_workspace ON sync_locks(user_id, device_id, workspace_id);

-- 3. 更新现有数据（将现有锁的 workspace_id 设为 NULL，表示不限制工作空间）
-- 这是一个安全的默认值，因为现有锁可能已经通过其他方式验证
UPDATE sync_locks SET workspace_id = NULL WHERE workspace_id IS NULL;

-- 4. 添加注释说明
ALTER TABLE sync_locks COMMENT = '同步操作锁，支持基于用户、设备和工作空间的并发控制';
