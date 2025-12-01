//! Configuration management for FerrisFetcher

use crate::error::{FerrisFetcherError, Result};
use crate::types::{HttpMethod, RateLimit, RetryPolicy};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;
use url::Url;

/// Main configuration for FerrisFetcher
#[derive(Debug, Clone)]
pub struct Config {
    /// Default user agent string
    pub user_agent: String,
    /// Request timeout
    pub timeout: Duration,
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimit>,
    /// Retry policy for failed requests
    pub retry_policy: RetryPolicy,
    /// Custom headers to send with every request
    pub headers: HeaderMap,
    /// Whether to follow redirects
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow
    pub max_redirects: usize,
    /// Cookie jar configuration
    pub cookie_jar: bool,
    /// Proxy configuration
    pub proxy: Option<Url>,
    /// Default HTTP method
    pub default_method: HttpMethod,
    /// Connection pool size
    pub connection_pool_size: usize,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Keep alive timeout
    pub keep_alive_timeout: Duration,
    /// TCP keep alive
    pub tcp_keep_alive: Duration,
    /// HTTP/2 configuration
    pub http2: bool,
    /// Compression
    pub compression: bool,
    /// Brotli compression
    pub brotli: bool,
    /// Gzip compression
    pub gzip: bool,
    /// Deflate compression
    pub deflate: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&format!("FerrisFetcher/{}", env!("CARGO_PKG_VERSION")))
                .expect("Invalid user agent"),
        );
        headers.insert("Accept", HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
        headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.5"));
        headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate, br"));
        headers.insert("DNT", HeaderValue::from_static("1"));
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));
        headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));

        Self {
            user_agent: format!("FerrisFetcher/{}", env!("CARGO_PKG_VERSION")),
            timeout: Duration::from_secs(30),
            max_concurrent_requests: 10,
            rate_limit: Some(RateLimit::default()),
            retry_policy: RetryPolicy::default(),
            headers,
            follow_redirects: true,
            max_redirects: 5,
            cookie_jar: true,
            proxy: None,
            default_method: HttpMethod::Get,
            connection_pool_size: 100,
            connect_timeout: Duration::from_secs(10),
            keep_alive_timeout: Duration::from_secs(60),
            tcp_keep_alive: Duration::from_secs(60),
            http2: true,
            compression: true,
            brotli: true,
            gzip: true,
            deflate: true,
        }
    }
}

impl Config {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a configuration with custom settings
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        if let Ok(value) = HeaderValue::from_str(&self.user_agent) {
            self.headers.insert(USER_AGENT, value);
        }
        self
    }
    
    /// Set timeout for requests
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Set maximum concurrent requests
    pub fn with_max_concurrent_requests(mut self, max: usize) -> Self {
        self.max_concurrent_requests = max;
        self
    }
    
    /// Set rate limiting
    pub fn with_rate_limit(mut self, rate_limit: RateLimit) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }
    
    /// Disable rate limiting
    pub fn without_rate_limit(mut self) -> Self {
        self.rate_limit = None;
        self
    }
    
    /// Set retry policy
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }
    
    /// Add a custom header
    pub fn with_header(mut self, name: &str, value: &str) -> Result<Self> {
        let header_name = name.parse::<reqwest::header::HeaderName>()
            .map_err(|e| FerrisFetcherError::ConfigError(format!("Invalid header name '{}': {}", name, e)))?;
        let header_value = HeaderValue::from_str(value)
            .map_err(|e| FerrisFetcherError::ConfigError(format!("Invalid header value for '{}': {}", name, e)))?;
        self.headers.insert(header_name, header_value);
        Ok(self)
    }
    
    /// Set proxy URL
    pub fn with_proxy(mut self, proxy: Url) -> Self {
        self.proxy = Some(proxy);
        self
    }
    
    /// Disable following redirects
    pub fn without_redirects(mut self) -> Self {
        self.follow_redirects = false;
        self
    }
    
    /// Set maximum number of redirects
    pub fn with_max_redirects(mut self, max: usize) -> Self {
        self.max_redirects = max;
        self
    }
    
    /// Disable cookie jar
    pub fn without_cookies(mut self) -> Self {
        self.cookie_jar = false;
        self
    }
    
    /// Set default HTTP method
    pub fn with_default_method(mut self, method: HttpMethod) -> Self {
        self.default_method = method;
        self
    }
    
    /// Set connection pool size
    pub fn with_connection_pool_size(mut self, size: usize) -> Self {
        self.connection_pool_size = size;
        self
    }
    
    /// Disable HTTP/2
    pub fn without_http2(mut self) -> Self {
        self.http2 = false;
        self
    }
    
    /// Disable compression
    pub fn without_compression(mut self) -> Self {
        self.compression = false;
        self.brotli = false;
        self.gzip = false;
        self.deflate = false;
        self
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.timeout.is_zero() {
            return Err(FerrisFetcherError::ConfigError("Timeout cannot be zero".to_string()));
        }
        
        if self.max_concurrent_requests == 0 {
            return Err(FerrisFetcherError::ConfigError("Max concurrent requests must be greater than 0".to_string()));
        }
        
        if self.max_redirects == 0 && self.follow_redirects {
            return Err(FerrisFetcherError::ConfigError("Max redirects must be greater than 0 when following redirects".to_string()));
        }
        
        if let Some(rate_limit) = &self.rate_limit {
            if rate_limit.requests_per_period == 0 {
                return Err(FerrisFetcherError::ConfigError("Rate limit requests per period must be greater than 0".to_string()));
            }
        }
        
        if self.retry_policy.max_attempts == 0 {
            return Err(FerrisFetcherError::ConfigError("Retry policy max attempts must be greater than 0".to_string()));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.user_agent, format!("FerrisFetcher/{}", crate::VERSION));
        assert!(config.follow_redirects);
        assert!(config.cookie_jar);
        assert!(config.http2);
        assert!(config.compression);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .with_timeout(Duration::from_secs(60))
            .with_max_concurrent_requests(20)
            .without_rate_limit()
            .without_redirects();
        
        assert!(config.validate().is_ok());
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_concurrent_requests, 20);
        assert!(config.rate_limit.is_none());
        assert!(!config.follow_redirects);
    }

    #[test]
    fn test_invalid_config() {
        let config = Config::new().with_timeout(Duration::from_secs(0));
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_custom_headers() {
        let config = Config::new()
            .with_header("X-Custom-Header", "test-value")
            .unwrap();
        
        assert!(config.headers.contains_key("x-custom-header"));
        assert_eq!(
            config.headers.get("x-custom-header").unwrap(),
            "test-value"
        );
    }
}
