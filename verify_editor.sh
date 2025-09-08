#!/bin/bash

echo "=== ai-commit 编辑器功能验证工具 ==="
echo

# 模拟临时文件创建过程
TEMP_FILE="/tmp/ai_commit_message.txt"
TEST_MESSAGE="feat(test): 这是一个测试消息，用来验证编辑器预填充功能"

echo "步骤1: 创建临时文件并写入测试内容..."
echo "$TEST_MESSAGE" > "$TEMP_FILE"

echo "步骤2: 验证文件创建成功..."
if [ -f "$TEMP_FILE" ]; then
    echo "✅ 临时文件创建成功: $TEMP_FILE"
else
    echo "❌ 临时文件创建失败"
    exit 1
fi

echo "步骤3: 验证文件内容..."
FILE_CONTENT=$(cat "$TEMP_FILE")
if [ "$FILE_CONTENT" = "$TEST_MESSAGE" ]; then
    echo "✅ 文件内容正确: $FILE_CONTENT"
else
    echo "❌ 文件内容不匹配"
    echo "预期: $TEST_MESSAGE"
    echo "实际: $FILE_CONTENT"
    exit 1
fi

echo "步骤4: 测试编辑器打开文件..."
echo "现在将使用你的默认编辑器打开这个文件"
echo "你应该能看到预填充的内容: $TEST_MESSAGE"
echo "请确认内容正确显示，然后保存并退出"
echo
read -p "按回车键继续，或按 Ctrl+C 取消: "

# 检测可用编辑器
if command -v "$EDITOR" >/dev/null 2>&1 && [ -n "$EDITOR" ]; then
    EDITOR_TO_USE="$EDITOR"
elif command -v vim >/dev/null 2>&1; then
    EDITOR_TO_USE="vim"
elif command -v vi >/dev/null 2>&1; then
    EDITOR_TO_USE="vi"  
elif command -v nano >/dev/null 2>&1; then
    EDITOR_TO_USE="nano"
else
    echo "❌ 没有找到可用的编辑器"
    exit 1
fi

echo "使用编辑器: $EDITOR_TO_USE"
echo "启动编辑器..."

$EDITOR_TO_USE "$TEMP_FILE"

echo "步骤5: 验证编辑后的内容..."
FINAL_CONTENT=$(cat "$TEMP_FILE")
echo "编辑后的内容: $FINAL_CONTENT"

# 清理
rm -f "$TEMP_FILE"
echo "✅ 验证完成，临时文件已清理"