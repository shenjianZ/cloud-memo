# 快速开始 - CentOS 7 构建

## 最快上手方式

### 1. 安装 Docker

```bash
sudo yum install -y yum-utils
sudo yum-config-manager --add-repo https://mirrors.aliyun.com/docker-ce/linux/centos/docker-ce.repo
sudo yum install -y docker-ce
sudo systemctl start docker
sudo systemctl enable docker
```

### 2. 选择构建方式

#### 方式 A：构建 Docker 镜像（推荐）

```bash
cd note-sync-server

# 构建镜像
docker build -t note-sync-server:latest .

# 运行
docker run -d -p 3000:3000 note-sync-server:latest
```

#### 方式 B：构建静态二进制文件

```bash
cd note-sync-server

# 使用脚本构建
chmod +x build-musl.sh
./build-musl.sh

# 运行
./target/x86_64-unknown-linux-musl/release/note-sync-server
```

## 文件说明

| 文件 | 说明 |
|------|------|
| `Dockerfile` | Docker 镜像构建文件 |
| `build-musl.sh` | 静态二进制构建脚本 |
| `docker-build.sh` | Docker 构建和测试脚本 |
| `BUILD_CENTOS7.md` | CentOS 7 详细构建指南 |
| `DOCKER_DEPLOYMENT.md` | Docker 部署详细指南 |
| `DEPLOYMENT.md` | 通用部署指南 |

## 快速参考

### Docker 命令

```bash
# 构建镜像
docker build -t note-sync-server:latest .

# 运行容器
docker run -d -p 3000:3000 note-sync-server:latest

# 查看日志
docker logs -f note-sync-server

# 进入容器
docker exec -it note-sync-server sh

# 停止容器
docker stop note-sync-server
```

### 二进制命令

```bash
# 构建
./build-musl.sh

# 运行
./target/x86_64-unknown-linux-musl/release/note-sync-server --env production

# 查看帮助
./target/x86_64-unknown-linux-musl/release/note-sync-server --help
```

## 配置文件位置

```
config/
├── default.toml          # 默认配置
├── development.toml      # 开发环境
└── production.toml       # 生产环境
```

## 环境变量

```bash
# 指定环境
CLOUDMEMO_ENV=production

# 指定配置文件
CLOUDMEMO_CONFIG_PATH=/path/to/config.toml

# 覆盖配置
CLOUDMEMO_SERVER__PORT=8000
CLOUDMEMO_DATABASE__URL=mysql://...
```

## 验证构建

```bash
# Docker 镜像
docker images | grep note-sync-server

# 二进制文件
ls -lh target/x86_64-unknown-linux-musl/release/note-sync-server
ldd target/x86_64-unknown-linux-musl/release/note-sync-server
```

## 常见问题

**Q: Docker 拉取镜像慢？**
A: 配置国内镜像源，见 `BUILD_CENTOS7.md`

**Q: 权限问题？**
A: 使用 `:z` 或 `:Z` 选项挂载卷

**Q: 端口被占用？**
A: 修改端口映射 `-p 8000:3000`

## 下一步

- 详细构建说明：查看 `BUILD_CENTOS7.md`
- Docker 部署：查看 `DOCKER_DEPLOYMENT.md`
- 通用部署：查看 `DEPLOYMENT.md`
