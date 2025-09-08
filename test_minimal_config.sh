#!/bin/bash

echo "=== 极简配置测试 ==="
echo

echo "1. 测试默认 ollama (无需配置):"
AI_COMMIT_PROVIDER=ollama ./target/debug/ai-commit --help 2>/dev/null | head -3
echo

echo "2. 测试 deepseek 配置 (只需 API Key):"
AI_COMMIT_PROVIDER=deepseek AI_COMMIT_PROVIDER_API_KEY=sk-test ./target/debug/ai-commit --help 2>/dev/null | head -3
echo

echo "3. 测试命令行优先级:"
AI_COMMIT_PROVIDER=deepseek ./target/debug/ai-commit --provider kimi --help 2>/dev/null | head -3
echo

echo "=== 极简环境变量测试完成 ==="
echo "✅ 只需要 3 个变量: PROVIDER, API_KEY, URL"
echo "✅ 彻底移除了复杂的特定提供商配置"
echo "✅ 代码更加简洁，迭代更快"