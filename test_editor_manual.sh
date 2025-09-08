#!/bin/bash

echo "=== ai-commit 编辑器功能手动测试 ==="
echo

# 创建测试文件
echo "创建测试变更..."
echo "# Manual editor test" > manual_editor_test.txt

echo "测试步骤："
echo "1. 运行 ai-commit 并等待 AI 生成消息"
echo "2. 当提示 '确认使用此 commit message? (y)es/(n)o/(e)dit:' 时，输入 'e'"
echo "3. 编辑器应该打开并显示 AI 生成的内容"
echo "4. 你可以编辑这个内容，然后保存退出"
echo "5. 系统会使用你编辑后的内容进行提交"
echo

echo "开始测试..."
echo "按 Ctrl+C 取消测试"
echo

# 设置调试模式
export AI_COMMIT_DEBUG=true

# 运行 ai-commit
make run

# 清理
rm -f manual_editor_test.txt

echo "测试完成！"