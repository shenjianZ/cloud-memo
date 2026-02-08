-- ============================================
-- Note Sync Server 数据库初始化脚本
-- 版本: 2.2.0
-- 说明: 创建所有表结构（包含所有迁移更改）
-- 更新:
--   - 添加工作空间支持
--   - 允许同一设备多账号（复合主键）
--   - 完整的设备追踪和同步字段
--   - 同步锁支持工作空间隔离（防止并发冲突）
-- ============================================

-- ============================================
-- 数据库创建和选择
-- ============================================
CREATE DATABASE IF NOT EXISTS `notes-sync` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE `notes-sync`;

-- ============================================
-- 1. 用户表
-- ============================================
CREATE TABLE IF NOT EXISTS users (
  id VARCHAR(10) PRIMARY KEY COMMENT '10位数字用户ID',
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  created_at BIGINT NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 2. 设备表（多端管理 + 多账号支持）
-- ============================================
CREATE TABLE IF NOT EXISTS devices (
  id VARCHAR(64) NOT NULL COMMENT '设备ID（UUID或默认格式：default-<md5>）',
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID',
  device_name VARCHAR(255) NOT NULL,
  device_type VARCHAR(50) DEFAULT 'desktop',
  revoked BOOLEAN DEFAULT FALSE,
  last_seen_at BIGINT NOT NULL,
  created_at BIGINT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  PRIMARY KEY (user_id, id) COMMENT '复合主键：允许同一设备被多个账号使用',
  INDEX idx_user_devices (user_id, revoked),
  INDEX idx_device_id (id) COMMENT '设备ID索引（用于优化查询）'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 3. 笔记表（云端精简版 + 设备追踪字段 + 工作空间）
-- ============================================
CREATE TABLE IF NOT EXISTS notes (
  id CHAR(36) PRIMARY KEY,
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID',
  workspace_id VARCHAR(36) DEFAULT NULL COMMENT '工作空间ID',

  title TEXT,
  content MEDIUMTEXT,
  folder_id CHAR(36),

  is_deleted BOOLEAN DEFAULT FALSE,
  deleted_at BIGINT,

  created_at BIGINT,
  updated_at BIGINT,
  server_ver INT NOT NULL DEFAULT 1,

  -- 客户端 UI 特有字段
  excerpt TEXT DEFAULT NULL COMMENT '笔记摘要（前端生成）',
  markdown_cache TEXT DEFAULT NULL COMMENT 'Markdown 缓存',
  is_favorite BOOLEAN DEFAULT FALSE COMMENT '是否收藏',
  is_pinned BOOLEAN DEFAULT FALSE COMMENT '是否置顶',
  author VARCHAR(255) DEFAULT NULL COMMENT '作者',
  word_count INT DEFAULT 0 COMMENT '字数统计',
  read_time_minutes INT DEFAULT 0 COMMENT '阅读时长（分钟）',

  -- 设备追踪字段
  device_id VARCHAR(64) DEFAULT NULL COMMENT '最后修改的设备ID',
  updated_by_device VARCHAR(255) DEFAULT NULL COMMENT '设备描述',

  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_notes (user_id),
  INDEX idx_updated_at (updated_at),
  INDEX idx_folder_id (folder_id),
  INDEX idx_device_id (device_id),
  INDEX idx_workspace_id (workspace_id),
  INDEX idx_is_favorite (is_favorite),
  INDEX idx_is_pinned (is_pinned)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 4. 文件夹表（软删除 + 级联 + 工作空间）
-- ============================================
CREATE TABLE IF NOT EXISTS folders (
  id CHAR(36) PRIMARY KEY,
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID',
  workspace_id VARCHAR(36) DEFAULT NULL COMMENT '工作空间ID',

  name VARCHAR(255) NOT NULL,
  parent_id CHAR(36),

  created_at BIGINT,
  updated_at BIGINT,

  is_deleted BOOLEAN DEFAULT FALSE COMMENT '是否已删除（软删除）',
  deleted_at BIGINT COMMENT '删除时间戳',

  server_ver INT NOT NULL DEFAULT 1,

  -- 设备追踪字段
  device_id VARCHAR(64) DEFAULT NULL COMMENT '最后修改的设备ID',
  updated_by_device VARCHAR(255) DEFAULT NULL COMMENT '设备描述',

  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE,
  INDEX idx_user_folders (user_id),
  INDEX idx_parent_id (parent_id),
  INDEX idx_device_id (device_id),
  INDEX idx_workspace_id (workspace_id),
  INDEX idx_folders_deleted (is_deleted)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 5. 手动版本快照表（含设备追踪 + 工作空间）
-- ============================================
CREATE TABLE IF NOT EXISTS note_versions (
  id CHAR(36) PRIMARY KEY,
  note_id CHAR(36) NOT NULL,
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID',
  workspace_id VARCHAR(36) DEFAULT NULL COMMENT '工作空间ID',

  title TEXT,
  content MEDIUMTEXT,

  snapshot_name VARCHAR(255),
  created_at BIGINT NOT NULL,

  -- 设备追踪字段
  device_id VARCHAR(64) DEFAULT NULL COMMENT '创建快照的设备ID',
  server_ver INT NOT NULL DEFAULT 1 COMMENT '服务器版本号',

  FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_note_versions (note_id, created_at),
  INDEX idx_device_id (device_id),
  INDEX idx_workspace_id (workspace_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 6. 同步历史表
-- ============================================
CREATE TABLE IF NOT EXISTS sync_history (
  id CHAR(36) PRIMARY KEY,
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID',
  sync_type VARCHAR(20) NOT NULL COMMENT 'push, pull, full',
  pushed_count INT DEFAULT 0 COMMENT '推送的笔记/文件夹数量',
  pulled_count INT DEFAULT 0 COMMENT '拉取的笔记/文件夹数量',
  conflict_count INT DEFAULT 0 COMMENT '冲突数量',
  error TEXT COMMENT '错误信息',
  duration_ms BIGINT DEFAULT 0 COMMENT '耗时（毫秒）',
  created_at BIGINT NOT NULL COMMENT '同步时间戳',
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_sync_history (user_id, created_at DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='同步历史记录表';

-- ============================================
-- 7. 用户资料表
-- ============================================
CREATE TABLE IF NOT EXISTS user_profiles (
  id CHAR(36) PRIMARY KEY,
  user_id VARCHAR(10) NOT NULL UNIQUE COMMENT '10位数字用户ID',
  username VARCHAR(100),
  phone VARCHAR(20),
  qq VARCHAR(20),
  wechat VARCHAR(50),
  avatar_data LONGTEXT COMMENT '头像图片数据（Base64 编码）',
  avatar_mime_type VARCHAR(50) COMMENT '头像图片类型（image/jpeg, image/png, image/gif）',
  bio TEXT,
  created_at BIGINT NOT NULL,
  updated_at BIGINT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_profiles_user_id (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 8. Refresh tokens 表（含撤销支持）
-- ============================================
CREATE TABLE IF NOT EXISTS refresh_tokens (
  id CHAR(36) PRIMARY KEY,
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID',
  token_hash VARCHAR(255) NOT NULL UNIQUE,
  device_id VARCHAR(255) NOT NULL,
  expires_at BIGINT NOT NULL,
  created_at BIGINT NOT NULL,
  revoked BOOLEAN DEFAULT FALSE COMMENT '是否已撤销',
  revoked_at BIGINT DEFAULT NULL COMMENT '撤销时间戳',
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_device (user_id, device_id),
  INDEX idx_token_hash (token_hash),
  INDEX idx_revoked (revoked)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 9. 工作空间表
-- ============================================
CREATE TABLE IF NOT EXISTS workspaces (
  id VARCHAR(36) PRIMARY KEY COMMENT '工作空间ID',
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID（与 users.id 类型一致）',
  name VARCHAR(100) NOT NULL COMMENT '空间名称',
  description TEXT COMMENT '空间描述',
  icon VARCHAR(50) COMMENT '空间图标',
  color VARCHAR(20) COMMENT '空间颜色',
  is_default BOOLEAN DEFAULT FALSE COMMENT '是否为默认空间',
  sort_order INTEGER DEFAULT 0 COMMENT '排序顺序',
  is_deleted BOOLEAN DEFAULT FALSE COMMENT '是否删除',
  deleted_at BIGINT DEFAULT NULL COMMENT '删除时间',
  created_at BIGINT NOT NULL COMMENT '创建时间',
  updated_at BIGINT NOT NULL COMMENT '更新时间',

  -- 同步字段
  server_ver INTEGER NOT NULL DEFAULT 0 COMMENT '服务器版本号（用于冲突检测）',
  device_id VARCHAR(50) DEFAULT NULL COMMENT '设备ID（用于追踪哪个设备创建了/修改了记录）',
  updated_by_device VARCHAR(255) DEFAULT NULL COMMENT '最后更新设备的ID（用于跨设备编辑检测）',

  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_id (user_id),
  INDEX idx_is_default (is_default),
  INDEX idx_is_deleted (is_deleted),
  INDEX idx_device_id (device_id),
  INDEX idx_server_ver (server_ver),
  UNIQUE KEY unique_default_workspace (user_id, is_default)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
COMMENT='用户工作空间表';

-- ============================================
-- 10. 标签表（软删除 + 设备追踪 + 工作空间）
-- ============================================
CREATE TABLE IF NOT EXISTS tags (
  id CHAR(36) PRIMARY KEY COMMENT '标签ID（UUID）',
  user_id VARCHAR(10) NOT NULL COMMENT '用户ID',
  workspace_id VARCHAR(36) DEFAULT NULL COMMENT '工作空间ID',
  name VARCHAR(100) NOT NULL COMMENT '标签名称',
  color VARCHAR(20) COMMENT '标签颜色',
  created_at BIGINT NOT NULL COMMENT '创建时间',
  updated_at BIGINT NOT NULL COMMENT '更新时间',

  is_deleted BOOLEAN DEFAULT FALSE COMMENT '是否已删除（软删除）',
  deleted_at BIGINT COMMENT '删除时间戳',

  server_ver INT NOT NULL DEFAULT 1 COMMENT '服务器版本号',

  -- 设备追踪字段
  device_id VARCHAR(64) DEFAULT NULL COMMENT '最后修改的设备ID',
  updated_by_device VARCHAR(255) DEFAULT NULL COMMENT '设备描述',

  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_tags (user_id),
  INDEX idx_updated_at (updated_at),
  INDEX idx_tags_deleted (is_deleted),
  INDEX idx_device_id (device_id),
  INDEX idx_workspace_id (workspace_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='标签表';

-- ============================================
-- 11. 笔记-标签关联表（软删除 + 工作空间）
-- ============================================
CREATE TABLE IF NOT EXISTS note_tags (
  note_id CHAR(36) NOT NULL COMMENT '笔记ID',
  tag_id CHAR(36) NOT NULL COMMENT '标签ID',
  user_id VARCHAR(10) NOT NULL COMMENT '用户ID',
  workspace_id VARCHAR(36) DEFAULT NULL COMMENT '工作空间ID',
  created_at BIGINT NOT NULL COMMENT '创建时间',

  is_deleted BOOLEAN DEFAULT FALSE COMMENT '是否已删除（软删除）',
  deleted_at BIGINT COMMENT '删除时间戳',

  PRIMARY KEY (note_id, tag_id),
  FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
  INDEX idx_note_tags_user (user_id),
  INDEX idx_workspace_id (workspace_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='笔记标签关联表';

-- ============================================
-- 12. 同步操作锁表（防止并发冲突，支持工作空间隔离）
-- ============================================
CREATE TABLE IF NOT EXISTS sync_locks (
  id CHAR(36) PRIMARY KEY,
  user_id VARCHAR(10) NOT NULL COMMENT '用户ID',
  device_id VARCHAR(255) NOT NULL COMMENT '设备ID',
  workspace_id VARCHAR(36) DEFAULT NULL COMMENT '工作空间ID（可选，NULL表示不限制）',
  acquired_at BIGINT NOT NULL COMMENT '获取锁的时间戳',
  expires_at BIGINT NOT NULL COMMENT '锁过期时间戳',
  INDEX idx_user_device_lock (user_id, device_id),
  INDEX idx_workspace_lock (user_id, device_id, workspace_id) COMMENT '支持工作空间的并发控制'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='同步操作锁表（支持工作空间隔离）';

-- ============================================
-- 初始化完成
-- ============================================

-- 显示所有创建的表
SELECT '✅ 数据库初始化完成' AS status;
SELECT '===== 已创建的表 =====' AS info;
SHOW TABLES;

-- 显示关键表的结构信息
SELECT '===== devices 表（多账号支持） =====' AS info;
SHOW INDEX FROM devices WHERE Key_name = 'PRIMARY';

SELECT '===== workspaces 表 =====' AS info;
DESCRIBE workspaces;

SELECT '===== sync_locks 表（工作空间隔离支持） =====' AS info;
DESCRIBE sync_locks;
SHOW INDEX FROM sync_locks;

SELECT '===== notes 表（工作空间支持） =====' AS info;
SHOW COLUMNS FROM notes LIKE 'workspace_id';
