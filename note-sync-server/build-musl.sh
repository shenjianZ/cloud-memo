#!/bin/bash
# 使用 muslrust 构建静态链接的二进制文件
# 适用于 CentOS 7 和其他 Linux 发行版

set -e

IMAGE_NAME="registry.cn-hangzhou.aliyuncs.com/pull-image/muslrust:latest"
CONTAINER_NAME="note-sync-builder"
PROJECT_DIR="$(pwd)"
OUTPUT_DIR="${PROJECT_DIR}/target/x86_64-unknown-linux-musl/release"

echo "========================================="
echo "  Note Sync Server - Musl Build Script"
echo "========================================="
echo ""

# 检查 Docker 是否安装
if ! command -v docker &> /dev/null; then
    echo "错误: Docker 未安装"
    echo "请先安装 Docker: https://docs.docker.com/engine/install/centos/"
    exit 1
fi


echo "拉取 ${IMAGE_NAME} 镜像..."
docker pull ${IMAGE_NAME}


echo ""
echo "开始构建..."
echo "项目目录: ${PROJECT_DIR}"
echo ""

# 创建一个临时的容器来构建项目
docker run --rm \
    -v "${PROJECT_DIR}:/volume:z" \
    -w /volume \
    -e CARGO_TARGET_DIR=/volume/target \
    ${IMAGE_NAME} \
    cargo build --release

echo ""
echo "========================================="
echo "  构建完成!"
echo "========================================="
echo ""
echo "二进制文件位置: ${OUTPUT_DIR}/note-sync-server"
echo ""

# 检查构建结果
if [ -f "${OUTPUT_DIR}/note-sync-server" ]; then
    echo "✓ 二进制文件已生成"
    echo ""
    echo "文件信息:"
    ls -lh "${OUTPUT_DIR}/note-sync-server"
    echo ""
    echo "依赖检查:"
    ldd "${OUTPUT_DIR}/note-sync-server" 2>&1 | head -1 || echo "✓ 静态链接成功 (not a dynamic executable)"
    echo ""
    echo "文件类型:"
    file "${OUTPUT_DIR}/note-sync-server"
    echo ""
    echo "可以使用以下命令运行:"
    echo "  ./${OUTPUT_DIR}/note-sync-server --help"
else
    echo "✗ 构建失败: 未找到二进制文件"
    exit 1
fi
