use anyhow::Result;
use r2d2_sqlite::rusqlite::Connection;

/// 初始化数据库表结构
///
/// 创建所有必要的表、索引、触发器和全文搜索虚拟表
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            excerpt TEXT,
            markdown_cache TEXT,
            folder_id TEXT,
            is_favorite BOOLEAN DEFAULT 0,
            is_deleted BOOLEAN DEFAULT 0,
            is_pinned BOOLEAN DEFAULT 0,
            author TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            deleted_at INTEGER,
            word_count INTEGER DEFAULT 0,
            read_time_minutes INTEGER DEFAULT 0,
            -- 云端同步字段（最小侵入：仅 3 个字段）
            server_ver INTEGER DEFAULT 0,
            is_dirty BOOLEAN DEFAULT 0,
            last_synced_at INTEGER,
            FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id TEXT,
            icon TEXT,
            color TEXT,
            sort_order INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_deleted BOOLEAN DEFAULT 0,
            deleted_at INTEGER,
            -- 云端同步字段
            server_ver INTEGER DEFAULT 0,
            is_dirty BOOLEAN DEFAULT 0,
            last_synced_at INTEGER,
            FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_deleted BOOLEAN DEFAULT 0,
            deleted_at INTEGER,
            -- 云端同步字段
            server_ver INTEGER DEFAULT 0,
            is_dirty BOOLEAN DEFAULT 0,
            last_synced_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS note_tags (
            note_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            is_deleted BOOLEAN DEFAULT 0,
            deleted_at INTEGER,
            PRIMARY KEY (note_id, tag_id),
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            note_id, title, content,
            tokenize = 'porter unicode61'
        );

        CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
            INSERT INTO notes_fts(rowid, note_id, title, content)
            VALUES (new.rowid, new.id, new.title, new.content);
        END;

        CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
            DELETE FROM notes_fts WHERE rowid = old.rowid;
        END;

        CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
            UPDATE notes_fts
            SET note_id = new.id, title = new.title, content = new.content
            WHERE rowid = old.rowid;
        END;

        CREATE INDEX IF NOT EXISTS idx_notes_folder_id ON notes(folder_id);
        CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_is_deleted ON notes(is_deleted);
        CREATE INDEX IF NOT EXISTS idx_notes_is_favorite ON notes(is_favorite);
        CREATE INDEX IF NOT EXISTS idx_folders_parent_id ON folders(parent_id);
        CREATE INDEX IF NOT EXISTS idx_folders_is_deleted ON folders(is_deleted);
        CREATE INDEX IF NOT EXISTS idx_tags_is_deleted ON tags(is_deleted);
        CREATE INDEX IF NOT EXISTS idx_note_tags_note_id ON note_tags(note_id);
        CREATE INDEX IF NOT EXISTS idx_note_tags_tag_id ON note_tags(tag_id);
        CREATE INDEX IF NOT EXISTS idx_note_tags_is_deleted ON note_tags(is_deleted);

        CREATE TABLE IF NOT EXISTS editor_settings (
            id INTEGER PRIMARY KEY,
            content_font_family TEXT NOT NULL DEFAULT 'Inter, Avenir, Helvetica, Arial, sans-serif',
            content_font_size INTEGER NOT NULL DEFAULT 16,
            content_font_weight INTEGER NOT NULL DEFAULT 400,
            content_line_height REAL NOT NULL DEFAULT 1.7,
            heading_font_family TEXT NOT NULL DEFAULT 'Inter, Avenir, Helvetica, Arial, sans-serif',
            heading_font_weight INTEGER NOT NULL DEFAULT 600,
            code_font_family TEXT NOT NULL DEFAULT 'JetBrains Mono, Fira Code, Consolas, Courier New, monospace',
            code_font_size INTEGER NOT NULL DEFAULT 14,
            updated_at INTEGER NOT NULL
        );

        -- 手动版本快照表（不同步到云端）
        CREATE TABLE IF NOT EXISTS note_snapshots (
            id TEXT PRIMARY KEY,
            note_id TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            snapshot_name TEXT,
            created_at INTEGER NOT NULL,
            server_ver INTEGER DEFAULT 1,
            is_dirty BOOLEAN DEFAULT 1,
            last_synced_at INTEGER,
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
        );

        -- 用户认证表（加密存储，支持多账号）
        CREATE TABLE IF NOT EXISTS user_auth (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL UNIQUE,
            server_url TEXT NOT NULL,
            email TEXT NOT NULL,
            access_token_encrypted TEXT NOT NULL,
            refresh_token_encrypted TEXT,
            token_expires_at INTEGER,
            device_id TEXT NOT NULL,
            last_sync_at INTEGER,
            is_current BOOLEAN DEFAULT 0,  -- 是否为当前激活的账号
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        -- 创建索引：快速查询当前账号
        CREATE INDEX IF NOT EXISTS idx_user_auth_current ON user_auth(is_current);

        -- 用户资料表（本地数据，不同步到服务器）
        -- 用于存储用户的补充信息（昵称、手机号等）
        CREATE TABLE IF NOT EXISTS user_profiles (
            id INTEGER PRIMARY KEY,
            user_id TEXT NOT NULL UNIQUE,
            username TEXT,
            phone TEXT,
            qq TEXT,
            wechat TEXT,
            avatar_data TEXT,  -- 头像图片数据（Base64 编码）
            avatar_mime_type TEXT,  -- 头像图片类型（image/jpeg, image/png, image/gif）
            bio TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        -- 同步状态表
        CREATE TABLE IF NOT EXISTS sync_state (
            id INTEGER PRIMARY KEY,
            last_sync_at INTEGER,
            pending_count INTEGER DEFAULT 0,
            conflict_count INTEGER DEFAULT 0,
            last_error TEXT
        );

        -- 应用配置表（设备级配置，所有用户共享）
        CREATE TABLE IF NOT EXISTS app_config (
            id INTEGER PRIMARY KEY,
            device_id TEXT NOT NULL UNIQUE,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        -- 应用设置表（全局配置）
        CREATE TABLE IF NOT EXISTS app_settings (
            id INTEGER PRIMARY KEY,
            default_server_url TEXT NOT NULL DEFAULT 'https://api.noteapp.com',
            auto_sync_enabled BOOLEAN DEFAULT 1,
            sync_interval_minutes INTEGER DEFAULT 5,
            theme TEXT DEFAULT 'system',
            language TEXT DEFAULT 'zh-CN',
            updated_at INTEGER NOT NULL
        );

        -- 设置表（键值对存储，用于存储各种配置）
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        -- 初始化默认配置
        INSERT OR IGNORE INTO app_settings (id, default_server_url, auto_sync_enabled, sync_interval_minutes, theme, language, updated_at)
        VALUES (1, 'http://localhost:3000', 0, 5, 'system', 'zh-CN', 1710000000);
    "
    )?;

    log::info!("Database schema initialized successfully");
    Ok(())
}
