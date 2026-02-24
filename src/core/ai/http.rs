use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;

/// 全局共享 HTTP 客户端
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client")
});

/// 获取共享的 HTTP 客户端引用
pub fn shared_client() -> &'static Client {
    &HTTP_CLIENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_client_returns_same_instance() {
        let c1 = shared_client();
        let c2 = shared_client();
        assert!(std::ptr::eq(c1, c2));
    }
}
