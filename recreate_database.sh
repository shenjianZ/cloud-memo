#!/bin/bash
# 重建数据库脚本
echo "⚠️  警告：此操作将删除所有数据！"
echo "请按 Ctrl+C 取消，或按回车继续..."
read

# 停止服务器
echo "🛑 停止服务器..."
# 根据实际启动方式调整，如：cargo kill 或 pkill -f note-sync-server

# 删除旧数据库
echo "🗑️  删除旧数据库..."
mysql -u root -p -e "DROP DATABASE IF EXISTS \`notes-sync\`;"

# 创建新数据库
echo "✨ 创建新数据库..."
mysql -u root -p < note-sync-server/sql/init.sql

echo "✅ 数据库重建完成！"
echo "🚀 现在可以重新启动服务器：cd note-sync-server && cargo run"
