# 配置文件说明

## 配置文件查找顺序

服务器启动时会按以下顺序查找配置文件，找到第一个存在的配置文件后使用：

1. **命令行参数指定的路径**：`--config /path/to/config.toml`
2. **环境变量**：`CLOUDMEMO_CONFIG_PATH=/path/to/config.toml`
3. **可执行文件同目录**：`{exe_dir}/config/{environment}.toml`
4. **当前工作目录**：`./config/{environment}.toml`
5. **可执行文件同目录**：`{exe_dir}/config/default.toml`
6. **当前工作目录**：`./config/default.toml`

## 环境变量

- `CLOUDMEMO_ENV`：指定运行环境（`development` 或 `production`），默认为 `development`
- `CLOUDMEMO_CONFIG_PATH`：指定配置文件路径
- `CLOUDMEMO_*`：环境变量覆盖配置，例如：
  - `CLOUDMEMO_SERVER__PORT=8000`
  - `CLOUDMEMO_DATABASE__URL=mysql://user:pass@localhost/db`

## 配置文件示例

### 开发环境配置 (config/development.toml)

```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "mysql://root:root@localhost:3306/notes-sync"
max_connections = 10

[auth]
jwt_secret = "development-secret-key"
jwt_expiration_days = 7

[redis]
url = "redis://localhost:6379"
```

### 生产环境配置 (config/production.toml)

```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "mysql://user:password@localhost:3306/notes-sync"
max_connections = 20

[auth]
jwt_secret = "your-strong-secret-key-here"
jwt_expiration_days = 7

[redis]
url = "redis://localhost:6379"
password = "your-redis-password"
```

## 使用方式

### 方式一：使用默认配置文件

将配置文件放在可执行文件同目录的 `config/` 文件夹下：

```bash
note-sync-server/
├── note-sync-server    # 可执行文件
└── config/
    └── production.toml  # 配置文件
```

然后运行：

```bash
./note-sync-server
```

### 方式二：通过环境变量指定环境

```bash
# 使用 production.toml
CLOUDMEMO_ENV=production ./note-sync-server

# 使用 development.toml
CLOUDMEMO_ENV=development ./note-sync-server
```

### 方式三：通过环境变量指定配置文件路径

```bash
CLOUDMEMO_CONFIG_PATH=/etc/note-sync/config.toml ./note-sync-server
```

### 方式四：通过命令行参数指定配置文件

```bash
./note-sync-server --config /path/to/config.toml
```

或同时指定环境：

```bash
./note-sync-server --env production
```

### 方式五：组合使用

```bash
# 指定配置文件，同时用环境变量覆盖某些设置
CLOUDMEMO_SERVER__PORT=8000 ./note-sync-server --config /etc/app/config.toml
```

## 部署建议

### 目录结构

建议的部署目录结构：

```
/opt/note-sync-server/
├── bin/
│   └── note-sync-server    # 可执行文件
└── config/
    ├── default.toml         # 基础配置
    ├── development.toml     # 开发环境配置
    └── production.toml      # 生产环境配置
```

### Systemd 服务配置示例

创建 `/etc/systemd/system/note-sync-server.service`：

```ini
[Unit]
Description=Note Sync Server
After=network.target mysql.service redis.service

[Service]
Type=simple
User=notesrv
WorkingDirectory=/opt/note-sync-server
ExecStart=/opt/note-sync-server/bin/note-sync-server --env production
Environment=CLOUDMEMO_ENV=production
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable note-sync-server
sudo systemctl start note-sync-server
```

### Docker 部署示例

```dockerfile
FROM alpine:latest

# 创建用户和目录
RUN addgroup -S notesrv && adduser -S notesrv -G notesrv
RUN mkdir -p /app/config

# 复制可执行文件和配置
COPY target/x86_64-unknown-linux-musl/release/note-sync-server /app/
COPY config/production.toml /app/config/

# 设置权限
RUN chown -R notesrv:notesrv /app

USER notesrv
WORKDIR /app

# 暴露端口
EXPOSE 3000

# 启动服务
CMD ["./note-sync-server", "--env", "production"]
```

使用自定义配置：

```bash
docker run -d \
  -v /path/to/custom-config.toml:/app/config/custom.toml:ro \
  -p 3000:3000 \
  note-sync-server \
  --config /app/config/custom.toml
```

## 环境变量完整列表

| 环境变量 | 说明 | 示例 |
|---------|------|------|
| `CLOUDMEMO_ENV` | 运行环境 | `production` |
| `CLOUDMEMO_CONFIG_PATH` | 配置文件路径 | `/etc/app/config.toml` |
| `CLOUDMEMO_SERVER__HOST` | 服务器地址 | `0.0.0.0` |
| `CLOUDMEMO_SERVER__PORT` | 服务器端口 | `3000` |
| `CLOUDMEMO_DATABASE__URL` | 数据库连接 | `mysql://...` |
| `CLOUDMEMO_DATABASE__MAX_CONNECTIONS` | 最大连接数 | `20` |
| `CLOUDMEMO_AUTH__JWT_SECRET` | JWT 密钥 | `your-secret` |
| `CLOUDMEMO_AUTH__JWT_EXPIRATION_DAYS` | Token 过期天数 | `7` |
| `CLOUDMEMO_REDIS__URL` | Redis 地址 | `redis://...` |
| `CLOUDMEMO_REDIS__PASSWORD` | Redis 密码 | `your-password` |

注意：环境变量中的嵌套配置使用双下划线 `__` 分隔。
