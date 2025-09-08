#!/bin/bash

echo "=== 测试查询历史功能 ==="
echo

echo "1. 执行一些测试查询..."
cargo run -- --query "since:2024-01-01" > /dev/null 2>&1
cargo run -- --query "type:fix" > /dev/null 2>&1
cargo run -- --query "author:costa" > /dev/null 2>&1
echo "✅ 已添加测试查询"

echo
echo "2. 显示查询历史..."
cargo run -- --query-history 2>&1 | grep -A 20 "Query History"

echo
echo "3. 显示查询统计..."
cargo run -- --query-stats 2>&1 | grep -A 10 "Query History Statistics"

echo
echo "=== 测试完成 ==="
echo
echo "可用的查询历史命令："
echo "  --query-history   显示查询历史记录"
echo "  --query-stats     显示查询统计信息"
echo "  --query-clear     清空查询历史"
echo "  --query-browse    交互式浏览历史"