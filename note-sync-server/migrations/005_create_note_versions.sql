-- 手动版本快照表
CREATE TABLE IF NOT EXISTS note_versions (
  id CHAR(36) PRIMARY KEY,
  note_id CHAR(36) NOT NULL,
  user_id CHAR(36) NOT NULL,

  title TEXT,
  content MEDIUMTEXT,

  snapshot_name VARCHAR(255),
  created_at BIGINT NOT NULL,

  FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_note_versions (note_id, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
