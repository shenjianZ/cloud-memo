-- 设备表（多端管理）
CREATE TABLE IF NOT EXISTS devices (
  id CHAR(36) PRIMARY KEY,
  user_id CHAR(36) NOT NULL,
  device_name VARCHAR(255) NOT NULL,
  device_type VARCHAR(50) DEFAULT 'desktop',
  revoked BOOLEAN DEFAULT FALSE,
  last_seen_at BIGINT NOT NULL,
  created_at BIGINT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_devices (user_id, revoked)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
