use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use anyhow::Result;
use crate::database::schema;

pub type DbPool = Pool<SqliteConnectionManager>;

/// 初始化数据库连接池
///
/// # 参数
/// * `db_path` - 数据库文件路径
///
/// # 返回
/// 返回数据库连接池
pub fn init_db_pool(db_path: &str) -> Result<DbPool> {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(10)
        .build(manager)?;

    // 初始化 schema
    let conn = pool.get()?;
    schema::init_schema(&conn)?;

    log::info!("Database initialized at: {}", db_path);
    Ok(pool)
}
