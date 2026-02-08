use thiserror::Error;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    #[error("连接池错误: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("数据库错误: {0}")]
    Database(#[from] r2d2_sqlite::rusqlite::Error),

    #[error("笔记不存在: {0}")]
    NoteNotFound(String),

    #[error("文件夹不存在: {0}")]
    FolderNotFound(String),

    #[error("标签不存在: {0}")]
    TagNotFound(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("未认证: {0}")]
    NotAuthenticated(String),

    #[error("认证错误: {0}")]
    AuthenticationError(String),

    #[error("冲突错误: {0}")]
    ConflictError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("同步错误: {0}")]
    SyncError(String),

    #[error("同步已取消: {0}")]
    SyncCancelled(String),

    #[error("加密错误: {0}")]
    EncryptionError(String),

    #[error("无效操作: {0}")]
    InvalidOperation(String),

    #[error("无效输入: {0}")]
    InvalidInput(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

/// 应用结果类型别名
pub type Result<T> = std::result::Result<T, AppError>;
