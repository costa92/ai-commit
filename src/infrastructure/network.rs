use reqwest::{Client, ClientBuilder};
use std::time::Duration;
use crate::infrastructure::error::ReviewError;

/// 网络客户端配置
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub max_retries: usize,
    pub retry_delay: Duration,
    pub user_agent: String,
    pub max_redirects: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            user_agent: format!("ai-commit/{}", env!("CARGO_PKG_VERSION")),
            max_redirects: 10,
        }
    }
}

/// 网络客户端管理器
pub struct NetworkManager {
    client: Client,
    config: NetworkConfig,
}

impl NetworkManager {
    pub fn new(config: NetworkConfig) -> Result<Self, ReviewError> {
        let client = ClientBuilder::new()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .user_agent(&config.user_agent)
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects))
            .build()
            .map_err(|e| ReviewError::network(
                format!("Failed to create HTTP client: {}", e),
                None
            ))?;

        Ok(Self { client, config })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn get_with_retry(&self, url: &str) -> Result<reqwest::Response, ReviewError> {
        let mut last_error = None;

        for attempt in 1..=self.config.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        last_error = Some(ReviewError::network(
                            format!("HTTP error: {}", response.status()),
                            Some(url.to_string())
                        ));
                    }
                },
                Err(e) => {
                    last_error = Some(ReviewError::network(
                        format!("Request failed: {}", e),
                        Some(url.to_string())
                    ));
                }
            }

            if attempt < self.config.max_retries {
                tokio::time::sleep(self.config.retry_delay).await;
            }
        }

        Err(last_error.unwrap_or_else(|| ReviewError::network(
            "All retry attempts failed".to_string(),
            Some(url.to_string())
        )))
    }

    pub async fn post_json_with_retry<T: serde::Serialize>(
        &self,
        url: &str,
        body: &T,
        headers: Option<&[(&str, &str)]>,
    ) -> Result<reqwest::Response, ReviewError> {
        let mut last_error = None;

        for attempt in 1..=self.config.max_retries {
            let mut request = self.client.post(url).json(body);

            if let Some(headers) = headers {
                for (key, value) in headers {
                    request = request.header(*key, *value);
                }
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        last_error = Some(ReviewError::network(
                            format!("HTTP error: {}", response.status()),
                            Some(url.to_string())
                        ));
                    }
                },
                Err(e) => {
                    last_error = Some(ReviewError::network(
                        format!("Request failed: {}", e),
                        Some(url.to_string())
                    ));
                }
            }

            if attempt < self.config.max_retries {
                tokio::time::sleep(self.config.retry_delay).await;
            }
        }

        Err(last_error.unwrap_or_else(|| ReviewError::network(
            "All retry attempts failed".to_string(),
            Some(url.to_string())
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert!(config.user_agent.contains("ai-commit"));
    }

    #[tokio::test]
    async fn test_network_manager_creation() {
        let config = NetworkConfig::default();
        let manager = NetworkManager::new(config);
        assert!(manager.is_ok());
    }
}