use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

// use crate::infrastructure::config::Config; // TODO: Add infrastructure module

/// AI 服务管理器，负责管理多个 AI 提供商
pub struct AIServiceManager {
    providers: HashMap<String, Box<dyn AIProvider>>,
    config: AIConfig,
    client: Arc<reqwest::Client>,
}

/// AI 提供商 trait，定义所有 AI 服务的通用接口
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// 获取提供商名称
    fn name(&self) -> &str;

    /// 分析代码并返回结果
    async fn analyze_code(&self, request: &AIRequest) -> Result<AIResponse>;

    /// 检查服务是否可用
    fn is_available(&self) -> bool;

    /// 获取支持的模型列表
    fn supported_models(&self) -> Vec<String>;

    /// 验证配置是否正确
    fn validate_config(&self, config: &AIConfig) -> Result<()>;
}

/// AI 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// 默认提供商
    pub default_provider: String,

    /// 默认模型
    pub default_model: String,

    /// 请求超时时间（秒）
    pub timeout: u64,

    /// 最大重试次数
    pub max_retries: u32,

    /// 温度参数
    pub temperature: f32,

    /// 最大 token 数
    pub max_tokens: u32,

    /// DeepSeek 配置
    pub deepseek: Option<DeepSeekConfig>,

    /// SiliconFlow 配置
    pub siliconflow: Option<SiliconFlowConfig>,

    /// Ollama 配置
    pub ollama: Option<OllamaConfig>,
}

/// DeepSeek 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

/// SiliconFlow 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiliconFlowConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

/// Ollama 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
}

/// AI 请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    /// 提示词内容
    pub prompt: String,

    /// 指定使用的模型（可选）
    pub model: Option<String>,

    /// 温度参数（可选）
    pub temperature: Option<f32>,

    /// 最大 token 数（可选）
    pub max_tokens: Option<u32>,

    /// 请求上下文
    pub context: AIRequestContext,
}

/// AI 请求上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequestContext {
    /// 文件路径
    pub file_path: String,

    /// 编程语言
    pub language: String,

    /// 请求类型
    pub request_type: AIRequestType,

    /// 额外元数据
    pub metadata: HashMap<String, String>,
}

/// AI 请求类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIRequestType {
    /// 代码审查
    CodeReview,

    /// 语言检测
    LanguageDetection,

    /// 质量评分
    QualityScoring,

    /// 改进建议
    ImprovementSuggestion,
}

/// AI 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    /// 响应内容
    pub content: String,

    /// 使用的模型
    pub model: String,

    /// 提供商名称
    pub provider: String,

    /// 响应时间（毫秒）
    pub response_time_ms: u64,

    /// token 使用情况
    pub token_usage: Option<TokenUsage>,

    /// 响应元数据
    pub metadata: HashMap<String, String>,
}

/// Token 使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 输入 token 数
    pub prompt_tokens: u32,

    /// 输出 token 数
    pub completion_tokens: u32,

    /// 总 token 数
    pub total_tokens: u32,
}

impl AIServiceManager {
    /// 创建新的 AI 服务管理器
    pub fn new(config: AIConfig) -> Result<Self> {
        let client = Arc::new(
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(config.timeout))
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?
        );

        Ok(Self {
            providers: HashMap::new(),
            config,
            client,
        })
    }

    /// 注册 AI 提供商
    pub fn register_provider(&mut self, provider: Box<dyn AIProvider>) -> Result<()> {
        let name = provider.name().to_string();

        // 验证提供商配置
        provider.validate_config(&self.config)?;

        self.providers.insert(name.clone(), provider);

        log::info!("Registered AI provider: {}", name);
        Ok(())
    }

    /// 获取可用的提供商列表
    pub fn available_providers(&self) -> Vec<String> {
        self.providers
            .iter()
            .filter(|(_, provider)| provider.is_available())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// 使用指定提供商分析代码
    pub async fn analyze_with_provider(&self, provider_name: &str, request: &AIRequest) -> Result<AIResponse> {
        let provider = self.providers
            .get(provider_name)
            .ok_or_else(|| anyhow!("Provider '{}' not found", provider_name))?;

        if !provider.is_available() {
            return Err(anyhow!("Provider '{}' is not available", provider_name));
        }

        let start_time = std::time::Instant::now();

        // 执行分析，支持重试
        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            match provider.analyze_code(request).await {
                Ok(mut response) => {
                    response.response_time_ms = start_time.elapsed().as_millis() as u64;
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries {
                        log::warn!("AI request failed (attempt {}), retrying: {}", attempt + 1, last_error.as_ref().unwrap());
                        tokio::time::sleep(std::time::Duration::from_millis(1000 * (attempt + 1) as u64)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("All retry attempts failed")))
    }

    /// 使用默认提供商分析代码
    pub async fn analyze_code(&self, request: &AIRequest) -> Result<AIResponse> {
        self.analyze_with_provider(&self.config.default_provider, request).await
    }

    /// 检测服务可用性
    pub async fn check_availability(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();

        for (name, provider) in &self.providers {
            results.insert(name.clone(), provider.is_available());
        }

        results
    }

    /// 获取配置
    pub fn config(&self) -> &AIConfig {
        &self.config
    }

    /// 获取 HTTP 客户端
    pub fn client(&self) -> Arc<reqwest::Client> {
        self.client.clone()
    }

    /// 更新配置
    pub fn update_config(&mut self, config: AIConfig) -> Result<()> {
        // 验证所有已注册提供商的配置
        for (name, provider) in &self.providers {
            if let Err(e) = provider.validate_config(&config) {
                return Err(anyhow!("Invalid config for provider '{}': {}", name, e));
            }
        }

        self.config = config;
        Ok(())
    }
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            default_provider: "ollama".to_string(),
            default_model: "qwen2.5-coder:7b".to_string(),
            timeout: 30,
            max_retries: 3,
            temperature: 0.7,
            max_tokens: 2048,
            deepseek: None,
            siliconflow: None,
            ollama: Some(OllamaConfig {
                base_url: "http://localhost:11434".to_string(),
                model: "qwen2.5-coder:7b".to_string(),
            }),
        }
    }
}

impl AIRequest {
    /// 创建代码审查请求
    pub fn code_review(file_path: &str, language: &str, prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            model: None,
            temperature: None,
            max_tokens: None,
            context: AIRequestContext {
                file_path: file_path.to_string(),
                language: language.to_string(),
                request_type: AIRequestType::CodeReview,
                metadata: HashMap::new(),
            },
        }
    }

    /// 创建语言检测请求
    pub fn language_detection(file_path: &str, content: &str) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("content_preview".to_string(), content.chars().take(200).collect());

        Self {
            prompt: format!("Detect the programming language of this file: {}", content),
            model: None,
            temperature: Some(0.1), // 低温度以获得更确定的结果
            max_tokens: Some(100),
            context: AIRequestContext {
                file_path: file_path.to_string(),
                language: "unknown".to_string(),
                request_type: AIRequestType::LanguageDetection,
                metadata,
            },
        }
    }

    /// 创建质量评分请求
    pub fn quality_scoring(file_path: &str, language: &str, code: &str) -> Self {
        Self {
            prompt: format!("Rate the quality of this {} code on a scale of 1-10: {}", language, code),
            model: None,
            temperature: Some(0.3),
            max_tokens: Some(500),
            context: AIRequestContext {
                file_path: file_path.to_string(),
                language: language.to_string(),
                request_type: AIRequestType::QualityScoring,
                metadata: HashMap::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_default() {
        let config = AIConfig::default();
        assert_eq!(config.default_provider, "ollama");
        assert_eq!(config.default_model, "qwen2.5-coder:7b");
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.ollama.is_some());
    }

    #[test]
    fn test_ai_request_creation() {
        let request = AIRequest::code_review("test.rs", "rust", "Review this code");
        assert_eq!(request.prompt, "Review this code");
        assert_eq!(request.context.file_path, "test.rs");
        assert_eq!(request.context.language, "rust");
        assert!(matches!(request.context.request_type, AIRequestType::CodeReview));
    }

    #[test]
    fn test_language_detection_request() {
        let request = AIRequest::language_detection("unknown.txt", "fn main() {}");
        assert!(request.prompt.contains("Detect the programming language"));
        assert_eq!(request.temperature, Some(0.1));
        assert_eq!(request.max_tokens, Some(100));
        assert!(matches!(request.context.request_type, AIRequestType::LanguageDetection));
    }

    #[test]
    fn test_quality_scoring_request() {
        let request = AIRequest::quality_scoring("test.py", "python", "print('hello')");
        assert!(request.prompt.contains("Rate the quality"));
        assert!(request.prompt.contains("python"));
        assert_eq!(request.temperature, Some(0.3));
        assert!(matches!(request.context.request_type, AIRequestType::QualityScoring));
    }

    #[tokio::test]
    async fn test_ai_service_manager_creation() {
        let config = AIConfig::default();
        let manager = AIServiceManager::new(config);
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert_eq!(manager.available_providers().len(), 0); // No providers registered yet
    }

    #[test]
    fn test_token_usage_serialization() {
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };

        let json = serde_json::to_string(&usage).unwrap();
        let deserialized: TokenUsage = serde_json::from_str(&json).unwrap();

        assert_eq!(usage.prompt_tokens, deserialized.prompt_tokens);
        assert_eq!(usage.completion_tokens, deserialized.completion_tokens);
        assert_eq!(usage.total_tokens, deserialized.total_tokens);
    }

    #[test]
    fn test_ai_response_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());

        let response = AIResponse {
            content: "Test response".to_string(),
            model: "test-model".to_string(),
            provider: "test-provider".to_string(),
            response_time_ms: 1000,
            token_usage: Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            }),
            metadata,
        };

        assert_eq!(response.content, "Test response");
        assert_eq!(response.model, "test-model");
        assert_eq!(response.provider, "test-provider");
        assert_eq!(response.response_time_ms, 1000);
        assert!(response.token_usage.is_some());
        assert_eq!(response.metadata.get("test_key"), Some(&"test_value".to_string()));
    }
}