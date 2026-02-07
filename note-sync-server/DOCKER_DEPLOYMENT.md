# Docker 部署指南

## 构建镜像

### 在 CentOS 7 上构建

```bash
# 进入项目目录
cd note-sync-server

# 构建镜像
docker build -t note-sync-server:latest .

# 查看镜像
docker images | grep note-sync-server
```

### 使用国内镜像加速（可选）

创建 `/etc/docker/daemon.json`:

```json
{
  "registry-mirrors": [
    "https://docker.mirrors.ustc.edu.cn",
    "https://hub-mirror.c.163.com"
  ]
}
```

重启 Docker:

```bash
sudo systemctl restart docker
```

## 运行容器

### 基本运行

```bash
docker run -d \
  --name note-sync-server \
  -p 3000:3000 \
  note-sync-server:latest
```

### 使用自定义配置文件

```bash
# 准备配置文件
mkdir -p /opt/note-sync/config
cp config/production.toml /opt/note-sync/config/
vi /opt/note-sync/config/production.toml

# 运行容器并挂载配置
docker run -d \
  --name note-sync-server \
  -p 3000:3000 \
  -v /opt/note-sync/config:/app/config:ro \
  note-sync-server:latest
```

### 使用环境变量覆盖配置

```bash
docker run -d \
  --name note-sync-server \
  -p 3000:3000 \
  -e CLOUDMEMO_SERVER__PORT=3000 \
  -e CLOUDMEMO_DATABASE__URL="mysql://user:pass@host:3306/db" \
  -e CLOUDMEMO_REDIS__URL="redis://host:6379" \
  note-sync-server:latest
```

### 完整的生产环境配置

```bash
docker run -d \
  --name note-sync-server \
  --restart unless-stopped \
  -p 3000:3000 \
  -v /opt/note-sync/config:/app/config:ro \
  -v /opt/note-sync/logs:/app/logs \
  -e CLOUDMEMO_ENV=production \
  -e CLOUDMEMO_DATABASE__URL="mysql://user:pass@mysql:3306/notes-sync" \
  -e CLOUDMEMO_REDIS__URL="redis://redis:6379" \
  --link mysql:mysql \
  --link redis:redis \
  note-sync-server:latest
```

## Docker Compose 部署

创建 `docker-compose.yml`:

```yaml
version: '3.8'

services:
  mysql:
    image: mysql:8.0
    container_name: note-sync-mysql
    restart: unless-stopped
    environment:
      MYSQL_ROOT_PASSWORD: your-root-password
      MYSQL_DATABASE: notes-sync
      MYSQL_USER: note-user
      MYSQL_PASSWORD: your-password
    volumes:
      - mysql-data:/var/lib/mysql
    ports:
      - "3306:3306"
    networks:
      - note-sync-network

  redis:
    image: redis:7-alpine
    container_name: note-sync-redis
    restart: unless-stopped
    command: redis-server --requirepass your-redis-password
    volumes:
      - redis-data:/data
    ports:
      - "6379:6379"
    networks:
      - note-sync-network

  note-sync-server:
    image: note-sync-server:latest
    container_name: note-sync-server
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      CLOUDMEMO_ENV: production
      CLOUDMEMO_DATABASE__URL: "mysql://note-user:your-password@mysql:3306/notes-sync"
      CLOUDMEMO_REDIS__URL: "redis://:your-redis-password@redis:6379"
      CLOUDMEMO_REDIS__PASSWORD: "your-redis-password"
    volumes:
      - ./config/production.toml:/app/config/production.toml:ro
      - note-sync-logs:/app/logs
    depends_on:
      - mysql
      - redis
    networks:
      - note-sync-network
    healthcheck:
      test: ["CMD", "wget", "-q", "-s", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s

volumes:
  mysql-data:
  redis-data:
  note-sync-logs:

networks:
  note-sync-network:
    driver: bridge
```

启动服务：

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 查看状态
docker-compose ps

# 停止服务
docker-compose down

# 停止并删除数据
docker-compose down -v
```

## 管理命令

### 查看日志

```bash
# 实时日志
docker logs -f note-sync-server

# 最近 100 行日志
docker logs --tail 100 note-sync-server

# 带时间戳的日志
docker logs -t note-sync-server
```

### 进入容器

```bash
# 进入运行的容器
docker exec -it note-sync-server sh

# 查看环境变量
docker exec note-sync-server env

# 查看进程
docker exec note-sync-server ps aux
```

### 容器管理

```bash
# 停止容器
docker stop note-sync-server

# 启动容器
docker start note-sync-server

# 重启容器
docker restart note-sync-server

# 删除容器
docker rm note-sync-server

# 强制停止并删除
docker rm -f note-sync-server
```

### 镜像管理

```bash
# 查看镜像
docker images

# 删除旧镜像
docker rmi note-sync-server:old-tag

# 清理悬空镜像
docker image prune

# 清理所有未使用的镜像
docker image prune -a
```

## 资源限制

### 限制内存和 CPU

```bash
docker run -d \
  --name note-sync-server \
  -p 3000:3000 \
  --memory="512m" \
  --memory-swap="1g" \
  --cpus="1.0" \
  note-sync-server:latest
```

### 在 Docker Compose 中设置资源限制

```yaml
services:
  note-sync-server:
    image: note-sync-server:latest
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

## 日志管理

### 配置日志驱动

```bash
docker run -d \
  --name note-sync-server \
  -p 3000:3000 \
  --log-driver json-file \
  --log-opt max-size=10m \
  --log-opt max-file=3 \
  note-sync-server:latest
```

### 使用 syslog

```bash
docker run -d \
  --name note-sync-server \
  -p 3000:3000 \
  --log-driver syslog \
  --log-opt syslog-address=tcp://192.168.0.42:514 \
  --log-opt tag=note-sync-server \
  note-sync-server:latest
```

## 备份和恢复

### 备份

```bash
# 导出镜像
docker save note-sync-server:latest | gzip > note-sync-server-latest.tar.gz

# 备份数据卷
docker run --rm \
  --volumes-from note-sync-server \
  -v $(pwd):/backup \
  alpine tar czf /backup/note-sync-data.tar.gz /app
```

### 恢复

```bash
# 导入镜像
gunzip -c note-sync-server-latest.tar.gz | docker load

# 恢复数据卷
docker run --rm \
  -v /opt/note-sync:/app \
  -v $(pwd):/backup \
  alpine tar xzf /backup/note-sync-data.tar.gz -C /
```

## 监控

### 健康检查

```bash
# 查看健康状态
docker inspect --format='{{.State.Health.Status}}' note-sync-server

# 查看健康检查日志
docker inspect --format='{{json .State.Health}}' note-sync-server | jq
```

### 资源监控

```bash
# 查看容器资源使用
docker stats note-sync-server

# 查看所有容器资源使用
docker stats
```

## 故障排查

### 常见问题

1. **容器无法启动**

```bash
# 查看日志
docker logs note-sync-server

# 检查配置文件
docker exec note-sync-server cat /app/config/production.toml
```

2. **无法连接到数据库**

```bash
# 检查网络
docker network inspect note-sync-network

# 检查 DNS
docker exec note-sync-server nslookup mysql
```

3. **健康检查失败**

```bash
# 手动检查
docker exec note-sync-server wget -q -s http://localhost:3000/health

# 查看端口监听
docker exec note-sync-server netstat -tlnp
```

## 安全建议

1. **使用非特权用户运行**（已在 Dockerfile 中配置）
2. **只挂载必要的目录**
3. **使用 secrets 管理敏感信息**
4. **定期更新基础镜像**
5. **扫描镜像漏洞**

```bash
# 使用 Trivy 扫描漏洞
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
    aquasec/trivy image note-sync-server:latest
```

## 性能优化

1. **使用多阶段构建减小镜像大小**（已实现）
2. **启用构建缓存**
3. **使用 .dockerignore 排除不必要的文件**（已创建）
4. **在 Cargo.toml 中启用 strip 选项**

## 更新部署

```bash
# 构建新镜像
docker build -t note-sync-server:v1.0.1 .

# 停止并删除旧容器
docker stop note-sync-server
docker rm note-sync-server

# 启动新容器
docker run -d \
  --name note-sync-server \
  -p 3000:3000 \
  note-sync-server:v1.0.1

# 或使用 Docker Compose
docker-compose pull
docker-compose up -d
```
