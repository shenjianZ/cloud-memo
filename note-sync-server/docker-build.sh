#!/bin/bash
# Docker 镜像构建和测试脚本

set -e

IMAGE_NAME="note-sync-server"
IMAGE_TAG="${1:-latest}"
CONTAINER_NAME="${IMAGE_NAME}-test"

echo "========================================="
echo "  Docker Build and Test Script"
echo "========================================="
echo ""

# 构建镜像
echo "开始构建 Docker 镜像..."
echo "镜像名称: ${IMAGE_NAME}:${IMAGE_TAG}"
echo ""

docker build -t ${IMAGE_NAME}:${IMAGE_TAG} .

echo ""
echo "========================================="
echo "  构建完成!"
echo "========================================="
echo ""

# 显示镜像信息
echo "镜像信息:"
docker images ${IMAGE_NAME}:${IMAGE_TAG}

echo ""
echo "镜像大小:"
docker images ${IMAGE_NAME}:${IMAGE_TAG} --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

echo ""
echo "镜像详情:"
docker inspect ${IMAGE_NAME}:${IMAGE_TAG} | jq -r '.[0] | {
    Id: .Id[:12],
    Created: .Created,
    Size: .Size,
    Architecture: .Architecture,
    Os: .Os
}'

# 询问是否测试运行
echo ""
read -p "是否测试运行容器? (y/N) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "测试运行容器..."

    # 停止并删除旧的测试容器
    if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        echo "删除旧的测试容器..."
        docker rm -f ${CONTAINER_NAME}
    fi

    # 运行新容器
    echo "启动测试容器..."
    docker run -d \
        --name ${CONTAINER_NAME} \
        -p 3000:3000 \
        -e CLOUDMEMO_ENV=development \
        ${IMAGE_NAME}:${IMAGE_TAG}

    echo ""
    echo "等待容器启动..."
    sleep 3

    echo ""
    echo "容器状态:"
    docker ps --filter "name=${CONTAINER_NAME}" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

    echo ""
    echo "容器日志:"
    docker logs ${CONTAINER_NAME}

    echo ""
    echo "健康检查:"
    if command -v wget &> /dev/null; then
        if wget -q -s http://localhost:3000/health; then
            echo "✓ 健康检查通过"
        else
            echo "✗ 健康检查失败"
        fi
    elif command -v curl &> /dev/null; then
        if curl -f -s http://localhost:3000/health > /dev/null; then
            echo "✓ 健康检查通过"
        else
            echo "✗ 健康检查失败"
        fi
    else
        echo "未找到 wget 或 curl，跳过健康检查"
    fi

    echo ""
    echo "容器信息:"
    docker inspect ${CONTAINER_NAME} | jq -r '.[0] | {
        Name: .Name,
        State: .State.Status,
        IP: .NetworkSettings.IPAddress,
        Mounts: [.Mounts[] | .Source + " -> " + .Destination]
    }'

    echo ""
    echo "========================================="
    echo "测试完成!"
    echo "========================================="
    echo ""
    echo "查看日志: docker logs -f ${CONTAINER_NAME}"
    echo "停止容器: docker rm -f ${CONTAINER_NAME}"
    echo ""
fi

# 保存镜像到文件
read -p "是否导出镜像到 tar 文件? (y/N) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    OUTPUT_FILE="${IMAGE_NAME}-${IMAGE_TAG}.tar.gz"
    echo ""
    echo "导出镜像到 ${OUTPUT_FILE}..."
    docker save ${IMAGE_NAME}:${IMAGE_TAG} | gzip > ${OUTPUT_FILE}
    echo "✓ 镜像已导出到 ${OUTPUT_FILE}"

    FILE_SIZE=$(du -h ${OUTPUT_FILE} | cut -f1)
    echo "文件大小: ${FILE_SIZE}"

    echo ""
    echo "在另一台机器上导入:"
    echo "  gunzip -c ${OUTPUT_FILE} | docker load"
fi

echo ""
echo "========================================="
echo "使用说明"
echo "========================================="
echo ""
echo "运行容器:"
echo "  docker run -d -p 3000:3000 ${IMAGE_NAME}:${IMAGE_TAG}"
echo ""
echo "查看日志:"
echo "  docker logs -f ${CONTAINER_NAME}"
echo ""
echo "进入容器:"
echo "  docker exec -it ${CONTAINER_NAME} sh"
echo ""
