impl_openai_provider!(
    /// 通义千问 (Qwen) 提供商
    ///
    /// 阿里云 DashScope OpenAI 兼容模式，复用 OpenAICompatibleBase。
    /// 默认 URL: https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions
    /// 默认 model: qwen-turbo
    /// 环境变量: AI_COMMIT_QWEN_API_KEY
    QwenProvider,
    "Qwen",
    None
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qwen_provider_creation() {
        let _provider = QwenProvider::new();
    }

    #[test]
    fn test_qwen_default() {
        let _provider = QwenProvider::default();
    }
}
