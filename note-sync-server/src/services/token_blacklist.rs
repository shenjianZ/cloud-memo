use redis::{AsyncCommands, Client};
use redis::aio::ConnectionManager;
use anyhow::Result;
use std::sync::Arc;

/// Redis Token 黑名单服务
/// 用于实现登出功能，将已登出的 token 加入黑名单
pub struct TokenBlacklist {
    manager: Arc<tokio::sync::Mutex<ConnectionManager>>,
}

impl TokenBlacklist {
    /// 创建新的 Token 黑名单服务实例
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;
        Ok(Self {
            manager: Arc::new(tokio::sync::Mutex::new(manager)),
        })
    }

    /// 将 token 加入黑名单
    /// key: "blacklist:{token}"
    /// value: "1"
    /// ttl: 与 token 过期时间一致（秒）
    pub async fn add(&self, token: &str, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.manager.lock().await;
        let key = format!("blacklist:{}", token);

        // 设置 key 并带上过期时间
        conn.set_ex(key, "1", ttl_seconds)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add token to blacklist: {}", e))?;

        tracing::debug!("Token added to blacklist with TTL: {} seconds", ttl_seconds);
        Ok(())
    }

    /// 检查 token 是否在黑名单中
    pub async fn contains(&self, token: &str) -> Result<bool> {
        let mut conn = self.manager.lock().await;
        let key = format!("blacklist:{}", token);

        let exists: bool = conn
            .exists::<_, bool>(key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check blacklist: {}", e))?;

        Ok(exists)
    }

    /// 清理过期的 token（Redis 会自动处理，这里仅作为手动清理接口）
    pub async fn cleanup(&self) -> Result<()> {
        // Redis 会自动删除过期的 key，这个方法仅用于未来可能的批量清理
        tracing::debug!("Cleanup called (Redis auto-expires keys)");
        Ok(())
    }

    /// 从黑名单中移除 token（用于测试或特殊情况）
    pub async fn remove(&self, token: &str) -> Result<()> {
        let mut conn = self.manager.lock().await;
        let key = format!("blacklist:{}", token);

        conn.del::<_, ()>(key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to remove token from blacklist: {}", e))?;

        tracing::debug!("Token removed from blacklist");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blacklist_operations() {
        let blacklist = TokenBlacklist::new("redis://localhost:6379").await.unwrap();
        let token = "test_token_123";

        // 添加到黑名单
        blacklist.add(token, 60).await.unwrap();

        // 检查是否存在
        assert!(blacklist.contains(token).await.unwrap());

        // 移除
        blacklist.remove(token).await.unwrap();

        // 再次检查
        assert!(!blacklist.contains(token).await.unwrap());
    }
}
