use anyhow::Result;
use sqlx::MySqlPool;
use chrono::Utc;
use crate::models::Device;

/// 设备服务
pub struct DeviceService {
    pool: MySqlPool,
}

impl DeviceService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 从 User-Agent 解析设备类型
    pub fn parse_device_type(user_agent: Option<&str>) -> &'static str {
        let ua = user_agent.unwrap_or("").to_lowercase();

        // 平板设备优先判断（因为tablet UA也可能包含"Mobile"）
        if ua.contains("ipad") || (ua.contains("android") && !ua.contains("mobile")) {
            return "tablet";
        }

        // 手机设备
        if ua.contains("iphone") || ua.contains("android") || ua.contains("mobile") {
            return "mobile";
        }

        // 桌面设备
        if ua.contains("windows") || ua.contains("macintosh") || ua.contains("linux") || ua.contains("ubuntu") {
            return "desktop";
        }

        // 默认为桌面设备（适合Tauri应用）
        "desktop"
    }

    /// 注册新设备或更新现有设备
    ///
    /// **多账号支持**：允许同一物理设备被多个账号使用
    /// - 主键为 (user_id, device_id) 复合主键
    /// - 每个用户-设备组合都是独立的记录
    /// - 同一个 device_id 可以关联不同的 user_id
    ///
    /// 示例：
    /// - 用户 A 使用设备 desktop-windows-xxx → 创建记录 (user_a, desktop-windows-xxx)
    /// - 用户 B 使用设备 desktop-windows-xxx → 创建记录 (user_b, desktop-windows-xxx)
    /// - 用户 A 再次登录 → 更新记录 (user_a, desktop-windows-xxx) 的 last_seen_at
    pub async fn register_or_update(
        &self,
        user_id: &str,
        device_id: &str,
        device_name: &str,
        device_type: &str,
    ) -> Result<Device> {
        let now = Utc::now().timestamp();

        // 查找当前用户的该设备记录（使用复合主键：user_id + device_id）
        let existing_device = sqlx::query_as::<_, Device>(
            "SELECT * FROM devices
             WHERE user_id = ? AND id = ?
             AND revoked = false
             LIMIT 1"
        )
        .bind(user_id)
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(mut device) = existing_device {
            // 设备已存在（当前用户之前注册过此设备），更新 last_seen_at 和设备信息
            tracing::info!(
                "更新现有设备记录: user_id={}, device_id={}, device_name={}",
                user_id, device_id, device_name
            );

            sqlx::query(
                "UPDATE devices SET last_seen_at = ?, device_name = ?, device_type = ?
                 WHERE user_id = ? AND id = ?"
            )
            .bind(now)
            .bind(device_name)
            .bind(device_type)
            .bind(user_id)
            .bind(device_id)
            .execute(&self.pool)
            .await?;

            device.last_seen_at = now;
            device.device_name = device_name.to_string();
            device.device_type = device_type.to_string();
            Ok(device)
        } else {
            // 创建新设备记录（该用户首次使用此设备）
            tracing::info!(
                "创建新设备记录: user_id={}, device_id={}, device_name={}",
                user_id, device_id, device_name
            );

            sqlx::query(
                "INSERT INTO devices (id, user_id, device_name, device_type, revoked, last_seen_at, created_at)
                 VALUES (?, ?, ?, ?, false, ?, ?)"
            )
            .bind(device_id)
            .bind(user_id)
            .bind(device_name)
            .bind(device_type)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

            Ok(Device {
                id: device_id.to_string(),
                user_id: user_id.to_string(),
                device_name: device_name.to_string(),
                device_type: device_type.to_string(),
                revoked: false,
                last_seen_at: now,
                created_at: now,
            })
        }
    }

    /// 获取用户的所有设备
    pub async fn list_devices(&self, user_id: &str) -> Result<Vec<Device>> {
        let devices = sqlx::query_as::<_, Device>(
            "SELECT * FROM devices
             WHERE user_id = ? AND revoked = false
             ORDER BY last_seen_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(devices)
    }

    /// 撤销设备
    pub async fn revoke_device(&self, device_id: &str, user_id: &str) -> Result<()> {
        let rows_affected = sqlx::query(
            "UPDATE devices SET revoked = true
             WHERE id = ? AND user_id = ?"
        )
        .bind(device_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Device not found or already revoked"));
        }

        Ok(())
    }

    /// 更新设备心跳
    pub async fn update_heartbeat(&self, device_id: &str, user_id: &str) -> Result<()> {
        let now = Utc::now().timestamp();

        let rows_affected = sqlx::query(
            "UPDATE devices SET last_seen_at = ?
             WHERE id = ? AND user_id = ? AND revoked = false"
        )
        .bind(now)
        .bind(device_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Device not found or revoked"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_device_type_desktop() {
        // Windows
        assert_eq!(
            DeviceService::parse_device_type(Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")),
            "desktop"
        );

        // macOS
        assert_eq!(
            DeviceService::parse_device_type(Some("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")),
            "desktop"
        );

        // Linux
        assert_eq!(
            DeviceService::parse_device_type(Some("Mozilla/5.0 (X11; Linux x86_64)")),
            "desktop"
        );

        println!("✅ Desktop devices recognized correctly");
    }

    #[test]
    fn test_parse_device_type_mobile() {
        // iPhone
        assert_eq!(
            DeviceService::parse_device_type(Some("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)")),
            "mobile"
        );

        // Android手机
        assert_eq!(
            DeviceService::parse_device_type(Some("Mozilla/5.0 (Linux; Android 13; SM-S908B) Mobile")),
            "mobile"
        );

        println!("✅ Mobile devices recognized correctly");
    }

    #[test]
    fn test_parse_device_type_tablet() {
        // iPad
        assert_eq!(
            DeviceService::parse_device_type(Some("Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X)")),
            "tablet"
        );

        // Android平板（无Mobile标识）
        assert_eq!(
            DeviceService::parse_device_type(Some("Mozilla/5.0 (Linux; Android 13; SM-X900)")),
            "tablet"
        );

        println!("✅ Tablet devices recognized correctly");
    }

    #[test]
    fn test_parse_device_type_edge_cases() {
        // 空UA（默认）
        assert_eq!(
            DeviceService::parse_device_type(None),
            "desktop"
        );

        // 大小写混合
        assert_eq!(
            DeviceService::parse_device_type(Some("Mozilla/5.0 (WINDOWS NT 10.0)")),
            "desktop"
        );

        println!("✅ Edge cases handled correctly");
    }
}
