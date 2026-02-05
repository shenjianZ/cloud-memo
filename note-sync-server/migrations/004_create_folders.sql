-- 文件夹表 + 客户端同步字段
CREATE TABLE IF NOT EXISTS folders (
  id CHAR(36) PRIMARY KEY,
  user_id CHAR(36) NOT NULL,

  name VARCHAR(255) NOT NULL,
  parent_id CHAR(36),

  is_deleted BOOLEAN DEFAULT FALSE,
  deleted_at BIGINT,

  created_at BIGINT,
  updated_at BIGINT,
  server_ver INT NOT NULL DEFAULT 1,

  -- 客户端同步字段
  is_dirty BOOLEAN DEFAULT FALSE COMMENT '是否需要同步到服务器',
  last_synced_at BIGINT DEFAULT NULL COMMENT '最后同步时间戳',

  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_folders (user_id),
  INDEX idx_parent_id (parent_id),
  INDEX idx_folders_dirty (is_dirty)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
