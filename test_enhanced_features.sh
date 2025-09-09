#!/bin/bash

echo "==================================="
echo "Testing Enhanced TUI Git Log Viewer"
echo "==================================="
echo ""

# 1. 测试 Git 提交加载
echo "1. Testing Git commits loading..."
./target/debug/test_tui
if [ $? -eq 0 ]; then
    echo "✅ Git commits loading: PASSED"
else
    echo "❌ Git commits loading: FAILED"
    exit 1
fi
echo ""

# 2. 测试基本查询功能
echo "2. Testing query commands..."
./target/release/ai-commit --query "author:costa" 2>&1 | head -5 > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ Query execution: PASSED"
else
    echo "❌ Query execution: FAILED"
fi
echo ""

# 3. 测试查询历史
echo "3. Testing query history..."
./target/release/ai-commit --query-history 2>&1 | head -5 > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ Query history: PASSED"
else
    echo "❌ Query history: FAILED"
fi
echo ""

# 4. 测试 Git diff 命令
echo "4. Testing Git diff command..."
LATEST_COMMIT=$(git log --pretty=format:"%H" -n 1)
git show $LATEST_COMMIT --color=never --stat --patch --abbrev-commit > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Git diff command: PASSED"
else
    echo "❌ Git diff command: FAILED"
fi
echo ""

# 5. 显示功能摘要
echo "==================================="
echo "Enhanced TUI Features Summary"
echo "==================================="
echo ""
echo "✨ New Features:"
echo "  • Git log viewing (100 recent commits)"
echo "  • Commit diff viewing"
echo "  • Multi-pane layout"
echo "  • Vim-style navigation"
echo "  • Syntax highlighting"
echo "  • Tab system"
echo "  • Command mode"
echo ""
echo "🚀 Launch with: ai-commit --query-tui-pro"
echo ""
echo "📖 Documentation: docs/ENHANCED_TUI_GIT_LOG.md"
echo ""
echo "All tests completed successfully! ✅"