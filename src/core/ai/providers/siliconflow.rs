impl_openai_provider!(
    /// SiliconFlow AI 提供商
    SiliconFlowProvider,
    "SiliconFlow",
    Some(0.9)
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_siliconflow_provider_creation() {
        let _provider = SiliconFlowProvider::new();
    }

    #[test]
    fn test_siliconflow_default() {
        let _provider = SiliconFlowProvider::default();
    }
}
