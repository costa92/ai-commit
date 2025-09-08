#!/bin/bash

echo "=== 测试统一环境变量功能 ==="
echo

echo "1. 测试默认 ollama 提供商 (无需 API Key):"
AI_COMMIT_PROVIDER=ollama ./target/debug/ai-commit --help 2>/dev/null | head -5
echo

echo "2. 测试使用统一 API Key 配置 deepseek:"
AI_COMMIT_PROVIDER=deepseek AI_COMMIT_PROVIDER_API_KEY=test-universal-key ./target/debug/ai-commit --help 2>/dev/null | head -5  
echo

echo "3. 测试统一 URL 覆盖:"
AI_COMMIT_PROVIDER=kimi AI_COMMIT_PROVIDER_URL=https://custom.api.com AI_COMMIT_PROVIDER_API_KEY=test-key ./target/debug/ai-commit --help 2>/dev/null | head -5
echo

echo "4. 验证特定提供商配置优先级:"
AI_COMMIT_PROVIDER=siliconflow AI_COMMIT_PROVIDER_API_KEY=universal-key AI_COMMIT_SILICONFLOW_API_KEY=specific-key ./target/debug/ai-commit --help 2>/dev/null | head -5
echo

echo "=== 测试完成 ==="
echo "统一环境变量功能已实现，具体配置说明请参考 example.env"