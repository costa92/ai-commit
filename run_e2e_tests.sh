#!/bin/bash

# E2E 测试运行脚本
# 运行所有端到端测试以验证 AI 提供商配置系统

set -e

echo "🚀 开始运行 AI 提供商配置系统 E2E 测试"
echo "================================================"

# 测试文件列表
TEST_FILES=(
    "provider_config_e2e_tests"
    "config_file_loading_e2e_tests" 
    "environment_variables_e2e_tests"
    "multi_provider_switching_e2e_tests"
)

# 如果检测到 Ollama 服务运行，则运行 Ollama 集成测试
if curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
    echo "✅ 检测到 Ollama 服务运行，将包含 Ollama 集成测试"
    TEST_FILES+=("ollama_integration_e2e_tests")
else
    echo "⚠️  未检测到 Ollama 服务，跳过 Ollama 集成测试"
    echo "   启动 Ollama: ollama serve"
fi

echo ""

# 运行测试统计
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
FAILED_TEST_NAMES=()

# 运行每个测试文件
for test_file in "${TEST_FILES[@]}"; do
    echo "📋 运行测试文件: ${test_file}"
    echo "----------------------------------------"
    
    if cargo test --test "${test_file}" --verbose 2>&1; then
        echo "✅ ${test_file} 测试通过"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo "❌ ${test_file} 测试失败"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        FAILED_TEST_NAMES+=("${test_file}")
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo ""
done

# 运行提供商相关的单元测试
echo "📋 运行提供商模块单元测试"
echo "----------------------------------------"

if cargo test providers::tests --lib --verbose 2>&1; then
    echo "✅ 提供商模块单元测试通过"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo "❌ 提供商模块单元测试失败"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    FAILED_TEST_NAMES+=("providers::tests")
fi

TOTAL_TESTS=$((TOTAL_TESTS + 1))
echo ""

# 测试结果总结
echo "================================================"
echo "🎯 E2E 测试结果总结"
echo "================================================"
echo "总测试模块数: ${TOTAL_TESTS}"
echo "通过: ${PASSED_TESTS}"
echo "失败: ${FAILED_TESTS}"

if [ ${FAILED_TESTS} -eq 0 ]; then
    echo ""
    echo "🎉 所有 E2E 测试都通过了！"
    echo ""
    echo "✅ AI 提供商配置系统功能验证完成："
    echo "   • 提供商注册表系统 ✅"
    echo "   • 配置文件加载系统 ✅" 
    echo "   • 环境变量配置系统 ✅"
    echo "   • 多提供商切换系统 ✅"
    if [[ " ${TEST_FILES[*]} " =~ " ollama_integration_e2e_tests " ]]; then
        echo "   • Ollama 集成测试 ✅"
    fi
    echo ""
    echo "📚 使用文档位置: docs/AI_PROVIDER_CONFIG.md"
    echo "🔧 配置文件位置: providers.toml"
    echo ""
    exit 0
else
    echo ""
    echo "❌ 有 ${FAILED_TESTS} 个测试模块失败："
    for failed_test in "${FAILED_TEST_NAMES[@]}"; do
        echo "   • ${failed_test}"
    done
    echo ""
    echo "请检查错误并修复后重新运行测试"
    exit 1
fi