use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::models::error::{Result, AppError};
use uuid::Uuid;

/// 设备标识服务
///
/// 负责生成和管理设备的唯一标识符
#[derive(Clone)]
pub struct DeviceIdentifierService {
    pool: Pool<SqliteConnectionManager>,
}

impl DeviceIdentifierService {
    /// 创建新的设备标识服务实例
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    /// 初始化设备标识（应用启动时调用）
    ///
    /// 确保设备有唯一标识，如果不存在则生成
    pub fn init(&self) -> Result<()> {
        self.get_or_create_device_id()?;
        Ok(())
    }

    /// 获取或生成设备唯一标识
    ///
    /// 逻辑：
    /// 1. 首次调用时生成新的设备 ID 并持久化
    /// 2. 后续调用返回已存储的设备 ID
    ///
    /// 返回格式：`<type>-<platform>-<uuid>`
    /// 示例：
    ///   - `desktop-windows-a1b2c3d4-e5f6-7890-abcd-ef1234567890`
    ///   - `desktop-macos-b2c3d4e5-f6g7-8901-bcde-f12345678901`
    ///   - `desktop-linux-c3d4e5f6-g7h8-9012-cdef-123456789012`
    ///   - `mobile-android-d4e5f6g7-h8i9-0123-def0-123456789abc`
    ///   - `mobile-ios-e5f6g7h8-i9j0-1234-ef01-23456789abcd`
    ///   - `tablet-ipad-f6g7h8i9-j0k1-2345-f012-3456789abcdef`
    pub fn get_or_create_device_id(&self) -> Result<String> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 尝试从 settings 表获取已有的 device_id
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'device_id' LIMIT 1")?;

        let device_id = match stmt.query_row([], |row| row.get::<_, String>(0)) {
            Ok(id) => {
                log::info!("[DeviceIdentifierService] 使用已存在的 device_id: {}", id);
                id
            }
            Err(_) => {
                // 首次运行，生成新的 device_id
                let device_type = Self::get_device_type();
                let platform = Self::get_platform();
                let uuid = Uuid::new_v4().to_string();
                let device_id = format!("{}-{}-{}", device_type, platform, uuid);

                // 持久化到 settings 表
                conn.execute(
                    "INSERT INTO settings (key, value) VALUES ('device_id', ?)",
                    [&device_id],
                ).map_err(|e| AppError::DatabaseError(format!("保存 device_id 失败: {}", e)))?;

                log::info!("[DeviceIdentifierService] 生成新的 device_id: {} (type={}, platform={})",
                    device_id, device_type, platform);
                device_id
            }
        };

        Ok(device_id)
    }

    /// 获取设备类型
    ///
    /// 注意：在编译时无法完全区分 mobile 和 tablet
    /// - Android/iOS 设备可能是手机或平板
    /// - 这里默认为 mobile，服务器会根据 User-Agent 更精确地判断
    fn get_device_type() -> &'static str {
        #[cfg(target_os = "windows")]
        { "desktop" }

        #[cfg(target_os = "macos")]
        { "desktop" }

        #[cfg(target_os = "linux")]
        { "desktop" }

        #[cfg(target_os = "android")]
        { "mobile" }  // 服务器会根据 User-Agent 区分 tablet

        #[cfg(target_os = "ios")]
        { "mobile" }  // 服务器会根据 User-Agent 区分 tablet

        // 兜底：其他未知平台视为桌面
        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_os = "android",
            target_os = "ios"
        )))]
        { "desktop" }
    }

    /// 获取平台标识
    fn get_platform() -> &'static str {
        #[cfg(target_os = "windows")]
        { "windows" }

        #[cfg(target_os = "macos")]
        { "macos" }

        #[cfg(target_os = "linux")]
        { "linux" }

        #[cfg(target_os = "android")]
        { "android" }

        #[cfg(target_os = "ios")]
        { "ios" }

        // 兜底：其他未知平台
        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_os = "android",
            target_os = "ios"
        )))]
        { "unknown" }
    }

    /// 重置设备标识（谨慎使用！）
    ///
    /// ⚠️ 警告：重置后，服务器会将此设备识别为新设备
    pub fn reset_device_id(&self) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute("DELETE FROM settings WHERE key = 'device_id'", [])
            .map_err(|e| AppError::DatabaseError(format!("删除 device_id 失败: {}", e)))?;

        log::warn!("[DeviceIdentifierService] device_id 已重置，下次将生成新的标识");
        Ok(())
    }
}
