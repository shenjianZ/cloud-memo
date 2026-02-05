# 云端同步功能

## 概述

本应用现已支持云端同步功能，可以：
- ✅ 在多个设备间同步笔记
- ✅ 自动同步更改到云端
- ✅ 冲突检测和解决
- ✅ 手动创建版本快照
- ✅ 数据加密存储

## 快速开始

### 1. 启动同步服务器

首先需要部署 Axum 同步服务器（详见 `note-sync-server/README.md`）：

```bash
cd note-sync-server
cargo run
```

服务器将在 `http://localhost:3000` 运行。

### 2. 在应用中登录

1. 打开设置页面
2. 找到"账户与同步"部分
3. 点击"注册"创建账户，或"登录"使用现有账户
4. 输入：
   - 服务器地址：`http://localhost:3000`
   - 邮箱和密码
5. 点击登录

### 3. 同步笔记

登录后：
- 笔记会自动同步到云端
- 可以点击"同步"按钮手动触发同步
- 同步状态会实时显示

## 功能特性

### 自动同步

- 创建或修改笔记时自动标记为"需要同步"
- 自动同步到云端（可在设置中关闭）
- 支持离线工作，网络恢复后自动同步

### 冲突解决

当同一笔记在多个设备上同时修改时：
- 服务器版本优先
- 本地版本创建副本（标题后缀："冲突副本 - 本地"）
- 可以手动合并内容

### 版本快照

手动保存笔记的重要版本：
- 点击编辑器工具栏的"创建快照"按钮
- 为快照命名（可选）
- 随时恢复到任意快照
- 快照仅存储在本地，不同步到云端

## 技术架构

### 客户端（Tauri 应用）

**后端（Rust）**：
- 数据库：SQLite + 同步字段（server_ver, is_dirty, last_synced_at）
- 服务层：SyncService, AuthService, SnapshotService
- API 客户端：reqwest + AES-GCM 加密

**前端（React）**：
- API 封装：syncApi.ts, authApi.ts, snapshotApi.ts
- 状态管理：Zustand（syncStore, authStore）
- UI：Settings 页面登录/注册界面

### 云端服务器（Axum + MySQL）

- Web 框架：Axum 0.7
- 数据库：MySQL 8.0
- 认证：JWT + bcrypt
- API：RESTful（/auth/*, /sync/*, /notes/*, /folders/*）

## 数据流

### 同步流程

```
客户端                    云端服务器
  |                          |
  |--[1] 获取脏笔记 -------->|
  |                          |
  |<--[2] 返回服务器版本 -----|
  |                          |
  |--[3] 推送更改 ----------->|
  |     (notes + folders)    |
  |                          |
  |<--[4] 返回结果 -----------|
  |     (新版本号 + 冲突)     |
  |                          |
  |--[5] 拉取服务器更改 ------>|
  |     (since last_sync)    |
  |                          |
  |<--[6] 返回服务器更改 ------|
```

### 冲突检测

```
if local.is_dirty && server.server_ver > local.server_ver:
    发生冲突
    解决：服务器版本 → 正常应用
          本地版本 → 创建副本
```

## 安全性

### 客户端

- Token 加密存储（AES-GCM）
- 加密密钥派生自设备 ID
- HTTPS 强制（生产环境）

### 服务器

- 密码哈希（bcrypt cost=12）
- JWT 签名（HS256）
- CORS 限制
- SQL 注入防护（参数化查询）

## 故障排查

### 无法连接服务器

1. 检查服务器是否运行：
   ```bash
   curl http://localhost:3000
   ```

2. 检查应用中的服务器地址是否正确

3. 查看浏览器控制台错误信息

### 同步失败

1. 检查网络连接
2. 确认已登录
3. 尝试手动触发同步
4. 查看同步状态提示

### 冲突未解决

1. 冲突会自动创建副本
2. 手动合并两个版本的内容
3. 删除不需要的版本

## API 文档

详细的 API 文档请参考 `note-sync-server/README.md`

主要端点：
- `POST /auth/login` - 用户登录
- `POST /auth/register` - 用户注册
- `POST /sync/push` - 推送更改
- `POST /sync/pull` - 拉取更改
- `POST /notes/:id/snapshots` - 创建快照
- `GET /notes/:id/snapshots` - 列出快照

## 开发和贡献

### 客户端开发

```bash
cd note
pnpm tauri:dev
```

### 服务器开发

```bash
cd note-sync-server
cargo run
```

### 运行测试

```bash
# 客户端
cd note
pnpm test

# 服务器
cd note-sync-server
cargo test
```

## 未来改进

- [ ] 多设备管理界面
- [ ] 选择性同步（仅同步特定文件夹）
- [ ] 端到端加密
- [ ] 协作编辑（实时同步）
- [ ] 版本历史可视化
- [ ] 回收站功能
- [ ] 导入/导出云端数据

## 许可证

MIT
