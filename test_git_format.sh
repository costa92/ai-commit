#!/bin/bash

echo "Testing Git log format..."
echo "=========================="

# Test the exact command used in the code
git log --pretty=format:"%H|%s|%an|%ai|%D" -n 5

echo -e "\n\nTesting with different format..."
echo "================================"

# Alternative format that might work better
git log --pretty=format:"%H|%s|%an|%aI|%D" -n 5

echo -e "\n\nDone!"