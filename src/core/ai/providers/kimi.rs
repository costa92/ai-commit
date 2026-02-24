impl_openai_provider!(
    /// Kimi AI 提供商
    KimiProvider,
    "Kimi",
    None
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kimi_provider_creation() {
        let _provider = KimiProvider::new();
    }

    #[test]
    fn test_kimi_default() {
        let _provider = KimiProvider::default();
    }
}
