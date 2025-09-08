#!/bin/bash

echo "=== 查询历史 TUI 界面演示 ==="
echo
echo "TUI界面提供了一个交互式的查询历史浏览器，类似于GRV的历史视图。"
echo
echo "功能特性："
echo "  📜 可视化历史列表"
echo "  🔍 实时搜索过滤"
echo "  📊 详细信息面板"
echo "  ⌨️  键盘快捷键导航"
echo
echo "快捷键："
echo "  ↑↓/jk    - 上下导航"
echo "  Enter    - 选择并执行查询"
echo "  /        - 搜索模式"
echo "  d        - 切换详情面板"
echo "  g/G      - 跳到开头/结尾"
echo "  f/b      - 翻页"
echo "  q/ESC    - 退出"
echo
echo "按任意键启动TUI界面..."
read -n 1

# 启动TUI
cargo run -- --query-tui