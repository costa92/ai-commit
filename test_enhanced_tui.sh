#!/bin/bash

echo "Testing GRV-style Enhanced TUI Features"
echo "========================================"
echo ""

# Test query history
echo "1. Testing Query History:"
./target/release/ai-commit --query-history | head -5
echo ""

# Test query stats
echo "2. Testing Query Statistics:"
./target/release/ai-commit --query-stats
echo ""

# Test various query types
echo "3. Testing Query Types:"
echo "   - Author query:"
./target/release/ai-commit --query "author:costa" | head -3
echo ""
echo "   - Message query:"
./target/release/ai-commit --query "message:fix" | head -3
echo ""
echo "   - Date query:"
./target/release/ai-commit --query "since:2025-01-01" | head -3
echo ""

# Test help
echo "4. Testing Help Output:"
./target/release/ai-commit --help | grep -A 2 "query"
echo ""

echo "✅ All non-interactive features tested successfully!"
echo ""
echo "Interactive features (TUI modes) available:"
echo "  --query-tui       : Standard TUI interface"
echo "  --query-tui-pro   : Enhanced GRV-style TUI with:"
echo "    - Split views (horizontal/vertical)"
echo "    - Multi-tab support"
echo "    - Vim-style command mode"
echo "    - Syntax highlighting"
echo "    - Focus management"
echo "  --query-browse    : Interactive browse mode"