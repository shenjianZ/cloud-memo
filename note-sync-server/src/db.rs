use sqlx::{MySql, Pool};
use sqlx::mysql::MySqlPoolOptions;
use crate::config::DatabaseConfig;

pub type DbPool = Pool<MySql>;

pub async fn create_pool(config: &DatabaseConfig) -> anyhow::Result<DbPool> {
    let pool = MySqlPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create connection pool: {}", e))?;

    tracing::info!("Connected to database at {}", config.url);

    Ok(pool)
}

