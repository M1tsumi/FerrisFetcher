//! HTTP client module for FerrisFetcher

use crate::config::Config;
use crate::error::{FerrisFetcherError, Result};
use crate::types::{HttpMethod, RequestStats};
use futures::future::BoxFuture;
use reqwest::{Client, Request, Response, Url};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// HTTP client with rate limiting and retry capabilities
#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    config: Config,
    semaphore: Arc<Semaphore>,
    stats: Arc<tokio::sync::Mutex<RequestStats>>,
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            semaphore: Arc::clone(&self.semaphore),
            stats: Arc::clone(&self.stats),
        }
    }
}

impl HttpClient {
    /// Create a new HTTP client with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        
        let mut client_builder = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .tcp_keepalive(config.tcp_keep_alive)
            .pool_max_idle_per_host(config.connection_pool_size / 4)
            .pool_idle_timeout(config.keep_alive_timeout);

        if config.follow_redirects {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::limited(config.max_redirects));
        } else {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::none());
        }

        if config.cookie_jar {
            client_builder = client_builder.cookie_store(true);
        }

        if config.compression {
            // Compression is enabled by default in reqwest
        }

        if let Some(proxy_url) = &config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url.as_str())
                .map_err(|e| FerrisFetcherError::ConfigError(format!("Invalid proxy URL: {}", e)))?;
            client_builder = client_builder.proxy(proxy);
        }

        let client = client_builder
            .default_headers(config.headers.clone())
            .build()
            .map_err(|e| FerrisFetcherError::ConfigError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            stats: Arc::new(tokio::sync::Mutex::new(RequestStats::new())),
            config,
        })
    }

    /// Execute a GET request to the given URL
    pub async fn get(&self, url: &str) -> Result<Response> {
        self.request(url, HttpMethod::Get, None, None).await
    }

    /// Execute a POST request to the given URL
    pub async fn post(&self, url: &str, body: Option<String>) -> Result<Response> {
        self.request(url, HttpMethod::Post, body, None).await
    }

    /// Execute a request with custom method and optional body
    pub async fn request(
        &self,
        url: &str,
        method: HttpMethod,
        body: Option<String>,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<Response> {
        let start_time = Instant::now();
        
        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await
            .map_err(|_| FerrisFetcherError::TaskCancelled)?;

        // Apply rate limiting if configured
        if let Some(rate_limit) = &self.config.rate_limit {
            tokio::time::sleep(rate_limit.delay_between_requests).await;
        }

        let url = Url::parse(url)?;
        let mut request_builder = match method {
            HttpMethod::Get => self.client.get(url.clone()),
            HttpMethod::Post => self.client.post(url.clone()),
            HttpMethod::Put => self.client.put(url.clone()),
            HttpMethod::Delete => self.client.delete(url.clone()),
            HttpMethod::Options => self.client.request(reqwest::Method::OPTIONS, url.clone()),
            HttpMethod::Head => self.client.head(url.clone()),
            HttpMethod::Patch => self.client.request(reqwest::Method::PATCH, url.clone()),
        };

        if let Some(body) = body {
            request_builder = request_builder.body(body);
        }

        if let Some(headers) = headers {
            request_builder = request_builder.headers(headers);
        }

        let request = request_builder.build()
            .map_err(FerrisFetcherError::HttpError)?;

        // Execute request with retry logic
        let response = self.execute_with_retry(request).await?;
        
        // Update statistics
        let elapsed = start_time.elapsed();
        self.update_stats(true, elapsed, response.content_length()).await;

        info!("Request completed: {} {} in {:?}", 
              response.status().as_u16(), 
              url, 
              elapsed);

        Ok(response)
    }

    /// Execute request with retry logic
    async fn execute_with_retry(&self, request: Request) -> Result<Response> {
        let mut last_error = None;
        
        for attempt in 1..=self.config.retry_policy.max_attempts {
            debug!("Attempt {} for request: {}", attempt, request.url());
            
            let request_clone = request.try_clone()
                .ok_or_else(|| FerrisFetcherError::ConfigError("Request body is not cloneable for retry".to_string()))?;

            match self.client.execute(request_clone).await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status().is_server_error() {
                        let error = FerrisFetcherError::NetworkError(format!("Server error: {}", response.status()));
                        last_error = Some(error);
                        
                        if attempt < self.config.retry_policy.max_attempts {
                            let delay = self.calculate_retry_delay(attempt);
                            warn!("Server error, retrying in {:?} (attempt {}/{})", 
                                  delay, attempt, self.config.retry_policy.max_attempts);
                            tokio::time::sleep(delay).await;
                        }
                    } else {
                        // Client errors (4xx) should not be retried
                        return Ok(response);
                    }
                }
                Err(e) => {
                    last_error = Some(FerrisFetcherError::HttpError(e));
                    
                    if attempt < self.config.retry_policy.max_attempts {
                        let delay = self.calculate_retry_delay(attempt);
                        warn!("Request failed, retrying in {:?} (attempt {}/{}): {:?}", 
                              delay, attempt, self.config.retry_policy.max_attempts, last_error);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(FerrisFetcherError::RetryExhausted))
    }

    /// Calculate retry delay based on attempt number and policy
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.config.retry_policy.base_delay;
        
        if self.config.retry_policy.exponential_backoff {
            let delay = self.config.retry_policy.base_delay * self.config.retry_policy.backoff_multiplier.powi(attempt as i32 - 1) as u32;
            std::cmp::min(delay, self.config.retry_policy.max_delay)
        } else {
            base_delay
        }
    }

    /// Update request statistics
    async fn update_stats(&self, success: bool, duration: Duration, bytes: Option<u64>) {
        let mut stats = self.stats.lock().await;
        stats.total_requests += 1;
        
        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }
        
        if let Some(bytes) = bytes {
            stats.total_bytes += bytes;
        }
        
        stats.total_time_ms += duration.as_millis() as u64;
        
        // Update average response time
        if stats.total_requests > 0 {
            stats.avg_response_time_ms = stats.total_time_ms as f64 / stats.total_requests as f64;
        }
    }

    /// Get current request statistics
    pub async fn get_stats(&self) -> RequestStats {
        self.stats.lock().await.clone()
    }

    /// Reset request statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.lock().await;
        *stats = RequestStats::new();
    }

    /// Check if the client is configured for rate limiting
    pub fn has_rate_limiting(&self) -> bool {
        self.config.rate_limit.is_some()
    }

    /// Get the maximum concurrent requests
    pub fn max_concurrent_requests(&self) -> usize {
        self.config.max_concurrent_requests
    }

    /// Create a future for a request (useful for batch operations)
    pub fn request_future<'a>(
        &'a self,
        url: &'a str,
        method: HttpMethod,
        body: Option<String>,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> BoxFuture<'a, Result<Response>> {
        Box::pin(self.request(url, method, body, headers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = Config::default();
        let client = HttpClient::new(config).unwrap();
        assert_eq!(client.max_concurrent_requests(), 10);
        assert!(client.has_rate_limiting());
    }

    // Note: Integration tests temporarily disabled due to mockito version compatibility
    // TODO: Update tests with compatible mocking library
}
