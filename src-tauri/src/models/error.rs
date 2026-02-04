use thiserror::Error;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] r2d2_sqlite::rusqlite::Error),

    #[error("Pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("Note not found: {0}")]
    NoteNotFound(String),

    #[error("Folder not found: {0}")]
    FolderNotFound(String),

    #[error("Tag not found: {0}")]
    TagNotFound(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// 应用结果类型别名
pub type Result<T> = std::result::Result<T, AppError>;
