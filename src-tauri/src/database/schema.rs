use r2d2_sqlite::rusqlite::Connection;
use anyhow::Result;

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
            FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id TEXT,
            icon TEXT,
            color TEXT,
            sort_order INTEGER DEFAULT 0,
            is_deleted BOOLEAN DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            deleted_at INTEGER,
            FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS note_tags (
            note_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
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
        CREATE INDEX IF NOT EXISTS idx_note_tags_note_id ON note_tags(note_id);
        CREATE INDEX IF NOT EXISTS idx_note_tags_tag_id ON note_tags(tag_id);

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
    "
    )?;

    log::info!("Database schema initialized successfully");
    Ok(())
}
