-- ============================================
-- Note Sync Server 数据库初始化脚本
-- 版本: 2.0.0
-- 说明: 创建所有表结构
-- 更新: 移除 folders 和 tags 的 is_deleted 字段
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
-- 2. 设备表（多端管理）
-- ============================================
CREATE TABLE IF NOT EXISTS devices (
  id VARCHAR(64) PRIMARY KEY COMMENT '设备ID（UUID或默认格式：default-<md5>）',
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID',
  device_name VARCHAR(255) NOT NULL,
  device_type VARCHAR(50) DEFAULT 'desktop',
  revoked BOOLEAN DEFAULT FALSE,
  last_seen_at BIGINT NOT NULL,
  created_at BIGINT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_devices (user_id, revoked)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 3. 笔记表（云端精简版 + 设备追踪字段）
-- ============================================
CREATE TABLE IF NOT EXISTS notes (
  id CHAR(36) PRIMARY KEY,
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID',

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
  INDEX idx_is_favorite (is_favorite),
  INDEX idx_is_pinned (is_pinned)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 4. 文件夹表（软删除 + 级联）
-- ============================================
CREATE TABLE IF NOT EXISTS folders (
  id CHAR(36) PRIMARY KEY,
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID',

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
  INDEX idx_folders_deleted (is_deleted)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 5. 手动版本快照表（含设备追踪）
-- ============================================
CREATE TABLE IF NOT EXISTS note_versions (
  id CHAR(36) PRIMARY KEY,
  note_id CHAR(36) NOT NULL,
  user_id VARCHAR(10) NOT NULL COMMENT '10位数字用户ID',

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
  INDEX idx_device_id (device_id)
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
-- 9. 标签表（软删除 + 设备追踪）
-- ============================================
CREATE TABLE IF NOT EXISTS tags (
  id CHAR(36) PRIMARY KEY COMMENT '标签ID（UUID）',
  user_id VARCHAR(10) NOT NULL COMMENT '用户ID',
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
  INDEX idx_device_id (device_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='标签表';

-- ============================================
-- 10. 笔记-标签关联表（软删除）
-- ============================================
CREATE TABLE IF NOT EXISTS note_tags (
  note_id CHAR(36) NOT NULL COMMENT '笔记ID',
  tag_id CHAR(36) NOT NULL COMMENT '标签ID',
  user_id VARCHAR(10) NOT NULL COMMENT '用户ID',
  created_at BIGINT NOT NULL COMMENT '创建时间',

  is_deleted BOOLEAN DEFAULT FALSE COMMENT '是否已删除（软删除）',
  deleted_at BIGINT COMMENT '删除时间戳',

  PRIMARY KEY (note_id, tag_id),
  FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
  INDEX idx_note_tags_user (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='笔记标签关联表';

-- ============================================
-- 11. 同步操作锁表（防止并发冲突，每个用户同时只能有一个锁）
-- ============================================
CREATE TABLE IF NOT EXISTS sync_locks (
  id CHAR(36) PRIMARY KEY,
  user_id VARCHAR(10) NOT NULL COMMENT '用户ID',
  device_id VARCHAR(255) NOT NULL COMMENT '设备ID',
  acquired_at BIGINT NOT NULL COMMENT '获取锁的时间戳',
  expires_at BIGINT NOT NULL COMMENT '锁过期时间戳',
  UNIQUE INDEX idx_user_lock (user_id),
  INDEX idx_device_lock (device_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='同步操作锁表';

-- ============================================
-- 初始化完成
-- ============================================
