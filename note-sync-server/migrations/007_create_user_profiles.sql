-- 用户资料表
CREATE TABLE IF NOT EXISTS user_profiles (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL UNIQUE,
    username VARCHAR(100),
    phone VARCHAR(20),
    qq VARCHAR(20),
    wechat VARCHAR(50),
    avatar_data LONGTEXT,  -- 头像图片数据（Base64 编码）
  avatar_mime_type VARCHAR(50),  -- 头像图片类型（image/jpeg, image/png, image/gif）
    bio TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 创建索引以提高查询性能
CREATE INDEX idx_user_profiles_user_id ON user_profiles(user_id);
