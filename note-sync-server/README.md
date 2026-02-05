# Note Sync Server

笔记应用的云端同步服务器（基于 Axum + MySQL）

## 功能特性

- ✅ 用户认证（注册/登录/JWT）
- ✅ 增量双向同步（push/pull）
- ✅ 冲突检测和解决
- ✅ 手动版本快照
- ✅ 文件夹管理
- ✅ 多设备支持

## 技术栈

- **Web 框架**: Axum 0.7
- **数据库**: MySQL 8.0+
- **认证**: JWT + bcrypt
- **异步运行时**: Tokio

## 开发环境设置

### 1. 安装 MySQL

#### Windows
```bash
# 下载 MySQL Installer
https://dev.mysql.com/downloads/installer/
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install mysql-server
sudo mysql_secure_installation
```

#### macOS
```bash
brew install mysql
brew services start mysql
```

### 2. 创建数据库

```bash
mysql -u root -p
```

```sql
CREATE DATABASE note_sync CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'note_user'@'localhost' IDENTIFIED BY 'your_password';
GRANT ALL PRIVILEGES ON note_sync.* TO 'note_user'@'localhost';
FLUSH PRIVILEGES;
EXIT;
```

### 3. 运行迁移

```bash
# 手动执行迁移文件
mysql -u root -p note_sync < migrations/001_create_users.sql
mysql -u root -p note_sync < migrations/002_create_devices.sql
mysql -u root -p note_sync < migrations/003_create_notes.sql
mysql -u root -p note_sync < migrations/004_create_folders.sql
mysql -u root -p note_sync < migrations/005_create_note_versions.sql
```

### 4. 配置环境变量

复制 `.env` 文件并修改配置：

```env
DATABASE_URL=mysql://note_user:your_password@localhost:3306/note_sync
JWT_SECRET=your-secret-key-change-this-in-production
SERVER_PORT=3000
HOST=0.0.0.0
```

**重要**: 生产环境请使用强密码作为 JWT_SECRET！

### 5. 运行服务器

```bash
cargo run
```

服务器将在 `http://localhost:3000` 启动。

## API 端点

### 认证

#### 注册
```
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

#### 登录
```
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

#### 登出
```
POST /auth/logout
Authorization: Bearer <token>
```

### 同步

#### 推送更改
```
POST /sync/push
Authorization: Bearer <token>
Content-Type: application/json

{
  "notes": [...],
  "folders": [...],
  "last_sync_at": 1700000000
}
```

#### 拉取更改
```
POST /sync/pull
Authorization: Bearer <token>
Content-Type: application/json

{
  "last_sync_at": 1700000000
}
```

### 笔记快照

#### 创建快照
```
POST /notes/:id/snapshots
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Note Title",
  "content": "Note content",
  "snapshot_name": "Before major changes"
}
```

#### 列出快照
```
GET /notes/:id/snapshots
Authorization: Bearer <token>
```

### 文件夹

#### 列出文件夹
```
GET /folders
Authorization: Bearer <token>
```

#### 创建文件夹
```
POST /folders
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My Folder",
  "parent_id": null
}
```

## 部署

### 使用 Docker（推荐）

```bash
# 构建镜像
docker build -t note-sync-server .

# 运行容器
docker run -d \
  -p 3000:3000 \
  -e DATABASE_URL="mysql://host.docker.internal:3306/note_sync" \
  -e JWT_SECRET="your-production-secret" \
  note-sync-server
```

### 直接部署

```bash
# 编译发布版本
cargo build --release

# 使用 systemd 运行
sudo cp target/release/note-sync-server /usr/local/bin/
sudo useradd -r -s /bin/false notesync
sudo chown notesync:notesync /usr/local/bin/note-sync-server
```

创建 systemd 服务文件 `/etc/systemd/system/note-sync-server.service`:

```ini
[Unit]
Description=Note Sync Server
After=network.target mysql.service

[Service]
Type=simple
User=notesync
ExecStart=/usr/local/bin/note-sync-server
Restart=always
Environment="DATABASE_URL=mysql://note_user:password@localhost:3306/note_sync"
Environment="JWT_SECRET=your-production-secret"

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable note-sync-server
sudo systemctl start note-sync-server
```

### 使用 Nginx 反向代理

```nginx
server {
    listen 80;
    server_name sync.yourdomain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

启用 HTTPS（使用 Let's Encrypt）:

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d sync.yourdomain.com
```

## 备份

### 数据库备份

创建备份脚本 `/usr/local/bin/backup-mysql.sh`:

```bash
#!/bin/bash
BACKUP_DIR="/var/backups/note_sync"
DATE=$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR
mysqldump -u root -p password note_sync > $BACKUP_DIR/note_sync_$DATE.sql
# 保留最近 7 天的备份
find $BACKUP_DIR -name "note_sync_*.sql" -mtime +7 -delete
```

添加到 crontab:

```bash
crontab -e
# 每天凌晨 2 点备份
0 2 * * * /usr/local/bin/backup-mysql.sh
```

## 监控

查看日志：

```bash
journalctl -u note-sync-server -f
```

## 安全建议

1. **使用强密码**: JWT_SECRET 至少 32 字符
2. **启用 HTTPS**: 生产环境必须使用 HTTPS
3. **限制 CORS**: 仅允许可信域名
4. **定期备份**: 每天备份数据库
5. **更新依赖**: 定期运行 `cargo update`
6. **防火墙**: 仅开放必要端口（80, 443）

## 故障排查

### 数据库连接失败
```bash
# 检查 MySQL 是否运行
sudo systemctl status mysql

# 测试连接
mysql -u note_user -p -h localhost note_sync
```

### 端口已被占用
```bash
# 查找占用端口的进程
sudo lsof -i :3000

# 终止进程
sudo kill -9 <PID>
```

## 开发指南

### 添加新端点

1. 在 `src/handlers/` 中创建或修改处理函数
2. 在 `src/main.rs` 中注册路由
3. 添加错误处理和日志
4. 编写测试

### 数据库迁移

创建新迁移文件 `migrations/006_xxx.sql`:

```sql
-- 描述迁移内容
ALTER TABLE notes ADD COLUMN new_column TEXT;
```

### 运行测试

```bash
cargo test
```

## 许可证

MIT
