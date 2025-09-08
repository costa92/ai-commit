#!/bin/bash

echo "=== 优化后的查询历史 TUI 界面演示 ==="
echo
echo "新版TUI提供了更完善的交互体验："
echo
echo "✨ 新特性："
echo "  📜 不会退出的历史浏览"
echo "  🔍 在TUI内执行查询"
echo "  📊 详细的查询信息面板"
echo "  ❓ 内置帮助系统"
echo "  🔄 实时刷新历史"
echo "  💬 执行结果弹窗显示"
echo
echo "🎮 快捷键："
echo "  Enter/x  - 执行选中的查询（不退出）"
echo "  ↑↓/jk    - 上下导航"
echo "  /        - 搜索模式"
echo "  d        - 切换详情面板"
echo "  r        - 刷新历史记录"
echo "  ?        - 显示帮助"
echo "  g/G      - 跳到开头/结尾"
echo "  f/b      - 翻页"
echo "  q/ESC    - 退出"
echo
echo "准备启动TUI界面..."
echo "按任意键继续..."
read -n 1

# 先添加一些测试数据
echo "添加测试查询..."
cargo run -- --query "author:test" 2>/dev/null
cargo run -- --query "message:feat" 2>/dev/null
cargo run -- --query "since:2024-01-01" 2>/dev/null

echo "启动TUI..."
cargo run -- --query-tui