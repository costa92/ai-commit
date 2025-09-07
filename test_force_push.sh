#!/bin/bash

echo "测试 ai-commit 的 force-push 功能"
echo "当前git状态:"
git status --short

echo ""
echo "测试 force-push 功能来解决推送冲突..."

# 创建一个小的测试修改
echo "# 测试 force-push 功能" > test_force_push_demo.txt
git add test_force_push_demo.txt

# 使用新的 force-push 功能
cargo run -- --force-push --push --provider ollama