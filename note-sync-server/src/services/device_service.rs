use anyhow::Result;
use sqlx::MySqlPool;
use uuid::Uuid;
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
        let ua = user_agent.unwrap_or("");

        if ua.contains("iPhone") || ua.contains("iPad") {
            "mobile"
        } else if ua.contains("Android") && !ua.contains("Mobile") {
            "tablet"
        } else if ua.contains("Android") {
            "mobile"
        } else if ua.contains("iPad") {
            "tablet"
        } else if ua.contains("Windows") || ua.contains("Macintosh") {
            "desktop"
        } else if ua.contains("Linux") {
            "desktop"
        } else if ua.contains("Mobile") || ua.contains("Tablet") {
            "tablet"
        } else {
            // 默认为 desktop
            "desktop"
        }
    }

    /// 注册新设备或更新现有设备
    pub async fn register_or_update(
        &self,
        user_id: &str,
        device_name: &str,
        device_type: &str,
    ) -> Result<Device> {
        let now = Utc::now().timestamp();

        // 首先尝试查找现有设备（通过 user_id 和 device_name）
        let existing_device = sqlx::query_as::<_, Device>(
            "SELECT * FROM devices
             WHERE user_id = ? AND device_name = ?
             AND revoked = false
             LIMIT 1"
        )
        .bind(user_id)
        .bind(device_name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(mut device) = existing_device {
            // 设备已存在，更新 last_seen_at
            sqlx::query(
                "UPDATE devices SET last_seen_at = ?, device_type = ?
                 WHERE id = ?"
            )
            .bind(now)
            .bind(device_type)
            .bind(&device.id)
            .execute(&self.pool)
            .await?;

            device.last_seen_at = now;
            device.device_type = device_type.to_string();
            Ok(device)
        } else {
            // 创建新设备
            let device_id = Uuid::new_v4().to_string();

            sqlx::query(
                "INSERT INTO devices (id, user_id, device_name, device_type, revoked, last_seen_at, created_at)
                 VALUES (?, ?, ?, ?, false, ?, ?)"
            )
            .bind(&device_id)
            .bind(user_id)
            .bind(device_name)
            .bind(device_type)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

            Ok(Device {
                id: device_id,
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
