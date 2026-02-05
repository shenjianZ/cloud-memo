-- 创建同步历史表
CREATE TABLE IF NOT EXISTS sync_history (
  id CHAR(36) PRIMARY KEY,
  user_id CHAR(36) NOT NULL,
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
