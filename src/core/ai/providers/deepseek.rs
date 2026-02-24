impl_openai_provider!(
    /// Deepseek AI 提供商
    DeepseekProvider,
    "Deepseek",
    None
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepseek_provider_creation() {
        let _provider = DeepseekProvider::new();
    }

    #[test]
    fn test_deepseek_default() {
        let _provider = DeepseekProvider::default();
    }
}
