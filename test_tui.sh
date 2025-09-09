#!/bin/bash

echo "Testing enhanced TUI with integrated main view features..."

# Build first
echo "Building ai-commit..."
cargo build --release

# Test the enhanced TUI
echo -e "\nLaunching enhanced TUI (--query-tui-pro)..."
echo "You should see:"
echo "  - Multiple tabs: Git Log, Branches, Tags, Remotes"
echo "  - Split view when selecting branches/tags (left: list, right: commits)"
echo "  - Navigation with j/k or arrow keys"
echo "  - Tab switching with Tab/Shift+Tab"
echo "  - 'l' key to toggle left panel"
echo "  - 'c' or Enter to checkout branch/tag"
echo "  - 'p' to pull latest changes"
echo "  - 'q' to quit"
echo ""
echo "Press any key to launch..."
read -n 1

./target/release/ai-commit --query-tui-pro