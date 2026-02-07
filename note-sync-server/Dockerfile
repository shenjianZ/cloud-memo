# 多阶段构建 Dockerfile
# 构建阶段：使用阿里云 muslrust 构建静态链接的二进制文件
FROM registry.cn-hangzhou.aliyuncs.com/pull-image/muslrust:latest AS builder

WORKDIR /volume

# 复制 Cargo 配置
COPY Cargo.toml Cargo.lock ./

# 复制配置文件（用于 sqlx 编译时验证）
COPY config/ ./config/

# 创建一个虚拟的 src 目录来缓存依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src ./src

# 设置构建时环境变量（可以从 docker build --build-arg 传入）
ARG DATABASE_URL=mysql://root:root@localhost:3306/notes-sync
ARG BUILD_ENV=production

ENV DATABASE_URL=${DATABASE_URL}
ENV CLOUDMEMO_ENV=${BUILD_ENV}

# 构建实际的项目
RUN cargo build --release && \
    # 剥离符号以减小文件大小
    strip /volume/target/x86_64-unknown-linux-musl/release/note-sync-server

# 运行阶段：使用最小化的 Alpine 镜像
FROM alpine:3.19

# 安装必要的运行时依赖
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    && \
    # 清理 APK 缓存
    rm -rf /var/cache/apk/*

# 创建非特权用户
RUN addgroup -g 1000 notesrv && \
    adduser -D -u 1000 -G notesrv -s /bin/sh notesrv

# 创建应用目录
RUN mkdir -p /app/config && \
    chown -R notesrv:notesrv /app

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /volume/target/x86_64-unknown-linux-musl/release/note-sync-server /app/note-sync-server

# 复制默认配置文件
COPY config/production.toml /app/config/production.toml

# 设置权限
RUN chmod +x /app/note-sync-server && \
    chown -R notesrv:notesrv /app

# 切换到非特权用户
USER notesrv

# 暴露端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget -q -s http://localhost:3000/health || exit 1

# 设置环境变量
ENV CLOUDMEMO_ENV=production

# 启动应用
CMD ["./note-sync-server", "--env", "production"]
