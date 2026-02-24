impl_openai_provider!(
    /// OpenAI 提供商
    ///
    /// 直接使用 OpenAI API，复用 OpenAICompatibleBase。
    /// 默认 URL: https://api.openai.com/v1/chat/completions
    /// 默认 model: gpt-4o-mini
    /// 环境变量: AI_COMMIT_OPENAI_API_KEY
    OpenAIProvider,
    "OpenAI",
    None
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let _provider = OpenAIProvider::new();
    }

    #[test]
    fn test_openai_default() {
        let _provider = OpenAIProvider::default();
    }
}
