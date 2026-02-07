# 设备唯一标识功能说明

## 📱 设备 ID 格式

```
<type>-<platform>-<uuid>
```

### 示例

| 设备类型 | platform | device_id 示例 | 说明 |
|---------|----------|----------------|------|
| Desktop | Windows | `desktop-windows-a1b2c3d4-e5f6-7890-abcd-ef1234567890` | PC |
| Desktop | macOS | `desktop-macos-b2c3d4e5-f6g7-8901-bcde-f12345678901` | Mac |
| Desktop | Linux | `desktop-linux-c3d4e5f6-g7h8-9012-cdef-123456789012` | Linux |
| Mobile | Android | `mobile-android-d4e5f6g7-h8i9-0123-def0-123456789abc` | Android 手机 |
| Mobile | iOS | `mobile-ios-e5f6g7h8-i9j0-1234-ef01-23456789abcd` | iPhone |

### Tablet 处理说明

**客户端生成**：`mobile-android-xxx` 或 `mobile-ios-xxx`
**服务器识别**：根据 User-Agent 更新为 `tablet`

| User-Agent | 客户端发送 | 服务器识别 | 最终 device_type |
|-----------|-----------|-----------|----------------|
| iPad (iOS) | `mobile-ios-xxx` | ✅ 识别为 iPad | `tablet` |
| Android (无Mobile) | `mobile-android-xxx` | ✅ 识别为平板 | `tablet` |
| Android Phone | `mobile-android-xxx` | ✅ 识别为手机 | `mobile` |

**示例流程**：
```
1. iPad 客户端生成: mobile-ios-a1b2c3d4-...
2. 发送到服务器: device_id = "mobile-ios-a1b2c3d4-..."
3. 服务器解析 User-Agent: iPad → tablet
4. 存储到数据库: device_type = "tablet"
5. 设备列表显示: 📱 mobile-ios-a1b2c... (tablet)
```

---

## 🎯 支持的平台

| Platform | 编译时常量 | 运行时类型 | 说明 |
|----------|-----------|-----------|------|
| Windows | `windows` | `desktop` | Microsoft Windows |
| macOS | `macos` | `desktop` | Apple macOS |
| Linux | `linux` | `desktop` | Linux 发行版 |
| Android | `android` | `mobile`/`tablet` | Android（手机/平板） |
| iOS | `ios` | `mobile`/`tablet` | Apple iOS（iPhone/iPad） |

---

## 💾 持久化

存储在本地 SQLite 数据库：

```sql
INSERT INTO settings (key, value) VALUES
('device_id', 'desktop-windows-a1b2c3d4-e5f6-7890-abcd-ef1234567890');
```

---

## 🔧 使用方式

### 自动（推荐）

客户端自动生成，无需手动传递：

```typescript
await authApi.register({
  email: "user@example.com",
  password: "password123",
  serverUrl: "http://localhost:3000"
  // ✨ device_id 自动添加
});
```

### 手动获取

```rust
use crate::services::DeviceIdentifierService;

let device_service = DeviceIdentifierService::new(pool);
let device_id = device_service.get_or_create_device_id()?;

// Windows 示例
// desktop-windows-a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

---

## 📊 服务器端设备类型识别

服务器根据 User-Agent 精确识别：

```rust
// note-sync-server/src/services/device_service.rs

pub fn parse_device_type(user_agent: Option<&str>) -> &'static str {
    let ua = user_agent.unwrap_or("").to_lowercase();

    // iPad 优先（iPad UA 包含 "Mobile"）
    if ua.contains("ipad") || (ua.contains("android") && !ua.contains("mobile")) {
        return "tablet";
    }

    // 手机
    if ua.contains("iphone") || ua.contains("android") || ua.contains("mobile") {
        return "mobile";
    }

    // 桌面
    "desktop"
}
```

---

## ⚙️ 数据库要求

```sql
-- ✅ 正确（足够长）
CREATE TABLE devices (
  id VARCHAR(64) PRIMARY KEY  -- 支持 50-53 字符的 device_id
);

-- ❌ 错误（太短）
CREATE TABLE devices (
  id CHAR(36) PRIMARY KEY  -- 只能容纳 36 字符
);
```

---

## 🔄 从旧版本迁移

### 选项 1：自动迁移（推荐）

下次登录时自动生成新格式，旧记录不受影响。

### 选项 2：重置设备 ID

```rust
let device_service = DeviceIdentifierService::new(pool);
device_service.reset_device_id()?;  // ⚠️ 谨慎使用
```

---

## 📝 相关文件

- `src-tauri/src/services/device_identifier_service.rs` - 客户端实现
- `note-sync-server/src/services/device_service.rs` - 服务器端识别
- `note-sync-server/sql/init.sql` - 数据库表结构
- `note-sync-server/sql/migrations/002_fix_device_id_type.sql` - 迁移脚本

---

## ✅ 测试清单

- [x] Windows → `desktop-windows-*`
- [x] macOS → `desktop-macos-*`
- [x] Linux → `desktop-linux-*`
- [x] Android → `mobile-android-*`
- [x] iOS → `mobile-ios-*`
- [x] 编译通过
- [ ] 注册/登录集成测试
- [ ] 服务器端 tablet 识别测试
- [ ] 多设备同时在线测试

---

**版本**: v2.1.0
**更新**: 2026-02-06
**支持**: desktop, mobile, tablet
