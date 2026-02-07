# CentOS 7 构建指南

本指南说明如何在 CentOS 7 上使用阿里云 `muslrust` Docker 镜像构建静态链接的二进制文件。

## 前置要求

### 1. 安装 Docker

```bash
# 安装必要的依赖
sudo yum install -y yum-utils device-mapper-persistent-data lvm2

# 添加 Docker 仓库 (使用阿里云镜像)
sudo yum-config-manager --add-repo https://mirrors.aliyun.com/docker-ce/linux/centos/docker-ce.repo

# 更新缓存
sudo yum makecache fast

# 安装 Docker
sudo yum install -y docker-ce

# 启动 Docker 服务
sudo systemctl start docker
sudo systemctl enable docker

# 验证安装
docker --version
```

### 2. 配置 Docker 用户权限（可选）

```bash
# 将当前用户添加到 docker 组
sudo usermod -aG docker $USER

# 重新登录或执行
newgrp docker

# 验证
docker ps
```

## 构建步骤

### 方式一：使用构建脚本（推荐）

```bash
# 1. 进入项目目录
cd note-sync-server

# 2. 给脚本添加执行权限
chmod +x build-musl.sh

# 3. 执行构建
./build-musl.sh
```

### 方式二：手动使用 Docker 命令

```bash
# 1. 拉取 muslrust 镜像（使用阿里云镜像）
docker pull registry.cn-hangzhou.aliyuncs.com/pull-image/muslrust:latest

# 2. 进入项目目录
cd note-sync-server

# 3. 构建
docker run --rm \
    -v "$(pwd):/volume:z" \
    -w /volume \
    -e CARGO_TARGET_DIR=/volume/target \
    registry.cn-hangzhou.aliyuncs.com/pull-image/muslrust:latest \
    cargo build --release
```

### 方式三：进入容器内构建

```bash
# 1. 启动容器并挂载项目目录
docker run --rm -it \
    -v "$(pwd):/volume:z" \
    -w /volume \
    registry.cn-hangzhou.aliyuncs.com/pull-image/muslrust:latest \
    bash

# 2. 在容器内执行构建
cargo build --release

# 3. 退出容器
exit
```

## 构建输出

构建完成后，二进制文件位于：

```
target/x86_64-unknown-linux-musl/release/note-sync-server
```

### 验证构建结果

```bash
# 查看文件信息
ls -lh target/x86_64-unknown-linux-musl/release/note-sync-server

# 检查是否为静态链接
ldd target/x86_64-unknown-linux-musl/release/note-sync-server

# 应该输出: not a dynamic executable
# 或者列出很少的依赖 (仅 linux-vdso 和 ld-musl)

# 查看文件类型
file target/x86_64-unknown-linux-musl/release/note-sync-server
```

## 部署二进制文件

### 1. 准备部署目录

```bash
# 创建部署目录
sudo mkdir -p /opt/note-sync-server/{bin,config}

# 复制二进制文件
sudo cp target/x86_64-unknown-linux-musl/release/note-sync-server /opt/note-sync-server/bin/

# 设置执行权限
sudo chmod +x /opt/note-sync-server/bin/note-sync-server
```

### 2. 准备配置文件

```bash
# 复制配置文件
sudo cp config/production.toml /opt/note-sync-server/config/

# 根据实际情况修改配置
sudo vi /opt/note-sync-server/config/production.toml
```

### 3. 测试运行

```bash
# 测试运行
cd /opt/note-sync-server
./bin/note-sync-server --env production
```

### 4. 创建 systemd 服务

创建服务文件 `/etc/systemd/system/note-sync-server.service`:

```ini
[Unit]
Description=Note Sync Server
After=network.target mysql.service redis.service

[Service]
Type=simple
User=notesrv
Group=notesrv
WorkingDirectory=/opt/note-sync-server
ExecStart=/opt/note-sync-server/bin/note-sync-server --env production
Environment=CLOUDMEMO_ENV=production
Restart=always
RestartSec=10

# 安全设置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/note-sync-server

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
# 创建服务用户
sudo useradd -r -s /bin/false notesrv

# 设置目录权限
sudo chown -R notesrv:notesrv /opt/note-sync-server

# 重新加载 systemd
sudo systemctl daemon-reload

# 启用并启动服务
sudo systemctl enable note-sync-server
sudo systemctl start note-sync-server

# 查看状态
sudo systemctl status note-sync-server

# 查看日志
sudo journalctl -u note-sync-server -f
```

## Docker 镜像加速（可选）

如果 Docker 拉取镜像速度慢，可以配置国内镜像源：

```bash
# 创建或编辑 Docker 配置
sudo vi /etc/docker/daemon.json

# 添加以下内容：
{
  "registry-mirrors": [
    "https://docker.mirrors.ustc.edu.cn",
    "https://hub-mirror.c.163.com",
    "https://mirror.ccs.tencentyun.com"
  ]
}

# 重启 Docker
sudo systemctl daemon-reload
sudo systemctl restart docker
```

## 常见问题

### 1. 权限问题 (Permission denied)

```bash
# 使用 SELinux 安全选项
docker run --rm \
    -v "$(pwd):/volume:z" \
    ...

# 或临时禁用 SELinux (不推荐用于生产)
sudo setenforce 0
```

### 2. 构建失败

```bash
# 清理构建缓存
docker run --rm \
    -v "$(pwd):/volume:z" \
    -w /volume \
    registry.cn-hangzhou.aliyuncs.com/pull-image/muslrust:latest \
    cargo clean

# 重新构建
./build-musl.sh
```

### 3. 端口被占用

```bash
# 查看端口占用
sudo netstat -tlnp | grep 3000

# 或使用 ss 命令
sudo ss -tlnp | grep 3000
```

### 4. 查看日志

```bash
# 服务日志
sudo journalctl -u note-sync-server -n 100

# 实时日志
sudo journalctl -u note-sync-server -f
```

## 性能优化

### 1. 构建优化

```bash
# 使用 cargo 并行构建
docker run --rm \
    -v "$(pwd):/volume:z" \
    -w /volume \
    -e CARGO_TARGET_DIR=/volume/target \
    -e CARGO_BUILD_JOBS=4 \
    registry.cn-hangzhou.aliyuncs.com/pull-image/muslrust:latest \
    cargo build --release
```

### 2. 减小二进制文件大小

在 `Cargo.toml` 中添加：

```toml
[profile.release]
opt-level = "z"     # 优化大小
lto = true          # 链接时优化
codegen-units = 1   # 更好的优化
strip = true        # 自动剥离符号
panic = "abort"     # 减小 panic 处理大小
```

然后重新构建。

## 验证静态链接

```bash
# 完全静态链接的输出应该是:
# not a dynamic executable

# 部分静态链接（只依赖基础库）的输出可能是:
#   linux-vdso.so.1
#   ld-musl-x86_64.so.1
```

## 跨平台部署

构建的二进制文件可以在任何 x86_64 Linux 系统上运行，无需安装依赖：

- CentOS 7/8/9
- Ubuntu 16.04/18.04/20.04/22.04
- Debian 9/10/11
- Alpine Linux
- 其他 Linux 发行版

只需将 `note-sync-server` 二进制文件和配置文件复制到目标系统即可。
