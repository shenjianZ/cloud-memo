use anyhow::Result;
use std::str::FromStr;

/// 设备类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Unknown,
}

impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceType::Desktop => "desktop",
            DeviceType::Mobile => "mobile",
            DeviceType::Tablet => "tablet",
            DeviceType::Unknown => "unknown",
        }
    }
}

impl FromStr for DeviceType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "desktop" => Ok(DeviceType::Desktop),
            "mobile" => Ok(DeviceType::Mobile),
            "tablet" => Ok(DeviceType::Tablet),
            _ => Ok(DeviceType::Unknown),
        }
    }
}

/// 平台枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Android,
    IOS,
    Unknown,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Windows => "windows",
            Platform::MacOS => "macos",
            Platform::Linux => "linux",
            Platform::Android => "android",
            Platform::IOS => "ios",
            Platform::Unknown => "unknown",
        }
    }
}

impl FromStr for Platform {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "windows" => Ok(Platform::Windows),
            "macos" => Ok(Platform::MacOS),
            "linux" => Ok(Platform::Linux),
            "android" => Ok(Platform::Android),
            "ios" => Ok(Platform::IOS),
            _ => Ok(Platform::Unknown),
        }
    }
}

/// 设备信息结构体
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_type: DeviceType,
    pub platform: Platform,
    pub uuid: String,
    pub raw_id: String,
}

/// 设备标识解析服务
pub struct DeviceIdentifierService;

impl DeviceIdentifierService {
    /// 从 device_id 字符串解析设备信息
    ///
    /// 支持的格式：
    /// - `<type>-<platform>-<uuid>` (新格式)
    /// - `default-<md5>` (旧格式，兼容)
    /// - `<platform>-<uuid>` (过渡格式，兼容)
    ///
    /// 示例：
    ///   - "desktop-windows-a1b2c3d4-e5f6-7890-abcd-ef1234567890"
    ///   - "mobile-android-d4e5f6g7-h8i9-0123-def0-123456789abc"
    ///   - "default-a1b2c3d4e5f6789a1b2c3d4e5f6789" (旧格式)
    pub fn parse_device_id(device_id: &str) -> Result<DeviceInfo> {
        let parts: Vec<&str> = device_id.split('-').collect();

        let (device_type, platform, uuid, raw_id) = if parts.len() >= 3 {
            // 新格式: <type>-<platform>-<uuid>
            let device_type = parts[0].parse::<DeviceType>()
                .unwrap_or(DeviceType::Unknown);
            let platform = parts[1].parse::<Platform>()
                .unwrap_or(Platform::Unknown);
            let uuid = parts[2..].join("-");

            (device_type, platform, uuid, device_id.to_string())
        } else if parts.len() == 2 && parts[0] == "default" {
            // 旧格式: default-<md5>
            let uuid = parts[1].to_string();

            (DeviceType::Unknown, Platform::Unknown, uuid, device_id.to_string())
        } else if parts.len() == 2 {
            // 过渡格式: <platform>-<uuid>
            let platform = parts[0].parse::<Platform>()
                .unwrap_or(Platform::Unknown);
            let uuid = parts[1].to_string();

            // 根据平台推断类型
            let device_type = match platform {
                Platform::Windows | Platform::MacOS | Platform::Linux => DeviceType::Desktop,
                Platform::Android | Platform::IOS => DeviceType::Mobile,
                Platform::Unknown => DeviceType::Unknown,
            };

            (device_type, platform, uuid, device_id.to_string())
        } else {
            return Err(anyhow::anyhow!("Invalid device_id format: {}", device_id));
        };

        Ok(DeviceInfo {
            device_type,
            platform,
            uuid,
            raw_id,
        })
    }

    /// 从 device_id 和 User-Agent 综合识别设备类型
    ///
    /// 逻辑：
    /// 1. 从 device_id 解析设备类型
    /// 2. 如果类型为 mobile/unknown，结合 User-Agent 更精确识别
    /// 3. 如果 User-Agent 明确为 tablet，则覆盖为 tablet
    pub fn identify_device(
        device_id: &str,
        user_agent: Option<&str>,
    ) -> Result<DeviceInfo> {
        let mut info = Self::parse_device_id(device_id)?;

        // 如果 device_id 中的类型为 mobile 或 unknown，结合 User-Agent 优化
        if info.device_type == DeviceType::Mobile || info.device_type == DeviceType::Unknown {
            if let Some(ua) = user_agent {
                let ua_lower = ua.to_lowercase();

                // iPad 明确识别
                if ua_lower.contains("ipad") {
                    info.device_type = DeviceType::Tablet;
                }
                // Android 平板（UA 中没有 "Mobile"）
                else if info.platform == Platform::Android
                    && ua_lower.contains("android")
                    && !ua_lower.contains("mobile") {
                    info.device_type = DeviceType::Tablet;
                }
                // 其他情况保持 mobile 或设为 desktop
                else if info.device_type == DeviceType::Unknown {
                    if ua_lower.contains("iphone")
                        || ua_lower.contains("android")
                        || ua_lower.contains("mobile") {
                        info.device_type = DeviceType::Mobile;
                    } else if ua_lower.contains("windows")
                        || ua_lower.contains("macintosh")
                        || ua_lower.contains("linux") {
                        info.device_type = DeviceType::Desktop;
                    }
                }
            }
        }

        Ok(info)
    }

    /// 从 DeviceInfo 生成友好的设备名称
    pub fn get_device_name(info: &DeviceInfo) -> String {
        match (info.device_type, info.platform) {
            (DeviceType::Desktop, Platform::Windows) => "Windows 电脑".to_string(),
            (DeviceType::Desktop, Platform::MacOS) => "Mac 电脑".to_string(),
            (DeviceType::Desktop, Platform::Linux) => "Linux 电脑".to_string(),
            (DeviceType::Mobile, Platform::Android) => "Android 手机".to_string(),
            (DeviceType::Mobile, Platform::IOS) => "iPhone".to_string(),
            (DeviceType::Tablet, Platform::Android) => "Android 平板".to_string(),
            (DeviceType::Tablet, Platform::IOS) => "iPad".to_string(),
            _ => format!("{} {}", info.platform.as_str(), info.device_type.as_str()),
        }
    }

    /// 获取设备图标
    pub fn get_device_icon(info: &DeviceInfo) -> &'static str {
        match (info.device_type, info.platform) {
            (DeviceType::Desktop, Platform::Windows) => "💻",
            (DeviceType::Desktop, Platform::MacOS) => "🍎",
            (DeviceType::Desktop, Platform::Linux) => "🐧",
            (DeviceType::Mobile, Platform::Android) => "🤖",
            (DeviceType::Mobile, Platform::IOS) => "📱",
            (DeviceType::Tablet, Platform::Android) => "📱",
            (DeviceType::Tablet, Platform::IOS) => "📱",
            _ => "📟",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_new_format() {
        let device_id = "desktop-windows-a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        let info = DeviceIdentifierService::parse_device_id(device_id).unwrap();

        assert_eq!(info.device_type, DeviceType::Desktop);
        assert_eq!(info.platform, Platform::Windows);
        assert_eq!(info.raw_id, device_id);
    }

    #[test]
    fn test_parse_old_format() {
        let device_id = "default-a1b2c3d4e5f6789a1b2c3d4e5f6789";
        let info = DeviceIdentifierService::parse_device_id(device_id).unwrap();

        assert_eq!(info.device_type, DeviceType::Unknown);
        assert_eq!(info.platform, Platform::Unknown);
    }

    #[test]
    fn test_identify_tablet() {
        let device_id = "mobile-ios-a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        let user_agent = Some("Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X)");

        let info = DeviceIdentifierService::identify_device(device_id, user_agent).unwrap();

        assert_eq!(info.device_type, DeviceType::Tablet);
        assert_eq!(info.platform, Platform::IOS);
    }

    #[test]
    fn test_device_name() {
        let info = DeviceInfo {
            device_type: DeviceType::Desktop,
            platform: Platform::Windows,
            uuid: "a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
            raw_id: "desktop-windows-a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
        };

        let name = DeviceIdentifierService::get_device_name(&info);
        assert_eq!(name, "Windows 电脑");
    }

    #[test]
    fn test_device_icon() {
        let info = DeviceInfo {
            device_type: DeviceType::Tablet,
            platform: Platform::IOS,
            uuid: "a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
            raw_id: "mobile-ios-a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
        };

        let icon = DeviceIdentifierService::get_device_icon(&info);
        assert_eq!(icon, "📱");
    }
}
