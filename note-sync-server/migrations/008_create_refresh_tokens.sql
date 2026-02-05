-- Refresh tokens 表
-- 用于长期有效的刷新令牌
CREATE TABLE IF NOT EXISTS refresh_tokens (
  id CHAR(36) PRIMARY KEY,
  user_id CHAR(36) NOT NULL,
  token_hash VARCHAR(255) NOT NULL UNIQUE,
  device_id VARCHAR(255) NOT NULL,
  expires_at BIGINT NOT NULL,
  created_at BIGINT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_user_device (user_id, device_id),
  INDEX idx_token_hash (token_hash)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
