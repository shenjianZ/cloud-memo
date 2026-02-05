-- 笔记表（云端精简版 + 客户端同步字段）
CREATE TABLE IF NOT EXISTS notes (
  id CHAR(36) PRIMARY KEY,
  user_id CHAR(36) NOT NULL,

  title TEXT,
  content MEDIUMTEXT,
  folder_id CHAR(36),

  is_deleted BOOLEAN DEFAULT FALSE,
  deleted_at BIGINT,

  created_at BIGINT,
  updated_at BIGINT,
  server_ver INT NOT NULL DEFAULT 1,

  -- 客户端同步字段
  is_dirty BOOLEAN DEFAULT FALSE COMMENT '是否需要同步到服务器',
  last_synced_at BIGINT DEFAULT NULL COMMENT '最后同步时间戳',

  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_notes (user_id),
  INDEX idx_updated_at (updated_at),
  INDEX idx_folder_id (folder_id),
  INDEX idx_notes_dirty (is_dirty)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
