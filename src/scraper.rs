//! Main FerrisFetcher API - the high-level scraping interface

use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::extractor::{DataExtractor};
use crate::types::ExtractionRule;
use crate::html_parser::HtmlParser;
use crate::types::{HttpMethod, ScrapedData, RequestStats};
use futures::stream::{self, StreamExt};
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Main scraper interface - the primary API for FerrisFetcher
#[derive(Debug, Clone)]
pub struct FerrisFetcher {
    /// HTTP client for making requests
    client: HttpClient,
    /// Data extraction engine
    extractor: DataExtractor,
    /// Configuration
    config: Config,
}

impl FerrisFetcher {
    /// Create a new FerrisFetcher with default configuration
    pub fn new() -> Result<Self> {
        let config = Config::default();
        Self::with_config(config)
    }

    /// Create a new FerrisFetcher with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let client = HttpClient::new(config.clone())?;
        let extractor = DataExtractor::new();
        
        Ok(Self {
            client,
            extractor,
            config,
        })
    }

    /// Create a new FerrisFetcher with custom configuration and extraction rules
    pub fn with_config_and_rules(config: Config, rules: Vec<ExtractionRule>) -> Result<Self> {
        let client = HttpClient::new(config.clone())?;
        let extractor = DataExtractor::with_rules(rules);
        
        Ok(Self {
            client,
            extractor,
            config,
        })
    }

    /// Scrape a single URL
    pub async fn scrape(&self, url: &str) -> Result<ScrapedData> {
        self.scrape_with_method(url, HttpMethod::Get, None).await
    }

    /// Scrape a single URL with custom HTTP method
    pub async fn scrape_with_method(&self, url: &str, method: HttpMethod, body: Option<String>) -> Result<ScrapedData> {
        let start_time = Instant::now();
        info!("Starting scrape of: {}", url);

        // Make HTTP request
        let response = self.client.request(url, method, body, None).await?;
        let status_code = response.status().as_u16();

        // Extract headers
        let headers: std::collections::HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
            .collect();

        // Get response body
        let content = response.text().await?;

        // Parse HTML
        let parser = HtmlParser::new(&content)?;

        // Create scraped data structure
        let mut scraped_data = ScrapedData::new(url.to_string());
        scraped_data.status_code = status_code;
        scraped_data.headers = headers;
        scraped_data.content = content.clone();
        scraped_data.scrape_time_ms = start_time.elapsed().as_millis() as u64;

        // Extract basic metadata
        self.extract_basic_metadata(&parser, &mut scraped_data);

        // Extract structured data using rules
        if self.extractor.rule_count() > 0 {
            match self.extractor.extract_all(&parser) {
                Ok(extracted_data) => {
                    scraped_data.extracted_data = extracted_data;
                    debug!("Extracted data for {} fields", scraped_data.extracted_data.len());
                }
                Err(e) => {
                    warn!("Failed to extract structured data: {}", e);
                }
            }
        }

        info!("Successfully scraped: {} ({}ms)", url, scraped_data.scrape_time_ms);
        Ok(scraped_data)
    }

    /// Scrape multiple URLs concurrently
    pub async fn scrape_multiple(&self, urls: &[&str]) -> Result<Vec<ScrapedData>> {
        info!("Starting concurrent scrape of {} URLs", urls.len());
        
        let start_time = Instant::now();
        let concurrency_limit = self.config.max_concurrent_requests;
        
        let results = stream::iter(urls)
            .map(|url| async move {
                let scrape_start = Instant::now();
                match self.scrape(url).await {
                    Ok(data) => {
                        debug!("Successfully scraped: {} ({}ms)", url, scrape_start.elapsed().as_millis());
                        Some(data)
                    }
                    Err(e) => {
                        error!("Failed to scrape {}: {}", url, e);
                        None
                    }
                }
            })
            .buffer_unordered(concurrency_limit)
            .collect::<Vec<_>>()
            .await;

        let successful_results: Vec<ScrapedData> = results.into_iter().flatten().collect();
        let elapsed = start_time.elapsed();
        
        info!("Completed scraping: {}/{} URLs in {}ms", 
              successful_results.len(), 
              urls.len(), 
              elapsed.as_millis());

        Ok(successful_results)
    }

    /// Scrape multiple URLs with a progress callback
    pub async fn scrape_multiple_with_progress<F>(
        &self, 
        urls: &[&str], 
        progress_callback: F
    ) -> Result<Vec<ScrapedData>>
    where
        F: Fn(usize, usize, &ScrapedData) + Send + Sync + 'static,
    {
        info!("Starting concurrent scrape of {} URLs with progress reporting", urls.len());
        
        let concurrency_limit = self.config.max_concurrent_requests;
        let total_urls = urls.len();
        let (tx, mut rx) = mpsc::channel::<(usize, ScrapedData)>(concurrency_limit);
        
        // Spawn progress reporting task
        let progress_callback = Arc::new(progress_callback);
        let progress_task = tokio::spawn(async move {
            let mut _completed = 0;
            while let Some((index, data)) = rx.recv().await {
                _completed += 1;
                progress_callback(index, total_urls, &data);
            }
        });

        // Process URLs concurrently
        let results = stream::iter(urls.iter().enumerate())
            .map(|(index, url)| {
                let tx = tx.clone();
                async move {
                    match self.scrape(url).await {
                        Ok(data) => {
                            let _ = tx.send((index, data.clone())).await;
                            Some(data)
                        }
                        Err(e) => {
                            error!("Failed to scrape {}: {}", url, e);
                            None
                        }
                    }
                }
            })
            .buffer_unordered(concurrency_limit)
            .collect::<Vec<_>>()
            .await;

        // Wait for progress reporting to complete
        drop(tx); // Close the channel
        let _ = progress_task.await;

        let successful_results: Vec<ScrapedData> = results.into_iter().flatten().collect();
        
        info!("Completed scraping: {}/{} URLs", successful_results.len(), total_urls);
        Ok(successful_results)
    }

    /// Add an extraction rule
    pub fn add_extraction_rule(&mut self, rule: ExtractionRule) {
        self.extractor.add_rule(rule);
    }

    /// Remove an extraction rule
    pub fn remove_extraction_rule(&mut self, name: &str) -> Option<ExtractionRule> {
        self.extractor.remove_rule(name)
    }

    /// Get all extraction rules
    pub fn extraction_rules(&self) -> &std::collections::HashMap<String, ExtractionRule> {
        self.extractor.rules()
    }

    /// Get request statistics
    pub async fn get_stats(&self) -> RequestStats {
        self.client.get_stats().await
    }

    /// Reset request statistics
    pub async fn reset_stats(&self) {
        self.client.reset_stats().await;
    }

    /// Get configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Extract basic metadata from the page
    fn extract_basic_metadata(&self, parser: &HtmlParser, scraped_data: &mut ScrapedData) {
        // Extract title
        scraped_data.title = parser.title();

        // Extract description
        if let Some(description) = parser.description() {
            scraped_data.add_metadata("description", description.into());
        }

        // Extract keywords
        if let Some(keywords) = parser.keywords() {
            scraped_data.add_metadata("keywords", keywords.into());
        }

        // Extract canonical URL
        if let Some(canonical_url) = parser.canonical_url() {
            scraped_data.add_metadata("canonical_url", canonical_url.into());
        }

        // Extract JSON-LD structured data
        let json_ld = parser.json_ld();
        if !json_ld.is_empty() {
            scraped_data.add_metadata("json_ld", json_ld.into());
        }

        // Extract links and images counts
        let links_count = parser.links().len();
        let images_count = parser.images().len();
        scraped_data.add_metadata("links_count", (links_count as u64).into());
        scraped_data.add_metadata("images_count", (images_count as u64).into());

        // Extract forms count
        let forms_count = parser.forms().len();
        scraped_data.add_metadata("forms_count", (forms_count as u64).into());
    }

    /// Scrape and extract specific data by rule name
    pub async fn scrape_and_extract(&self, url: &str, rule_name: &str) -> Result<Vec<String>> {
        let scraped_data = self.scrape(url).await?;
        let parser = HtmlParser::new(&scraped_data.content)?;
        self.extractor.extract_by_name(&parser, rule_name)
    }

    /// Scrape and extract a single value by rule name
    pub async fn scrape_and_extract_single(&self, url: &str, rule_name: &str) -> Option<String> {
        match self.scrape_and_extract(url, rule_name).await {
            Ok(values) => values.into_iter().next(),
            Err(_) => None,
        }
    }

    /// Check if the scraper has rate limiting enabled
    pub fn has_rate_limiting(&self) -> bool {
        self.client.has_rate_limiting()
    }

    /// Get the maximum concurrent requests
    pub fn max_concurrent_requests(&self) -> usize {
        self.client.max_concurrent_requests()
    }
}

impl Default for FerrisFetcher {
    fn default() -> Self {
        Self::new().expect("Failed to create default FerrisFetcher")
    }
}

use std::sync::Arc;

/// Builder for creating FerrisFetcher instances with fluent API
pub struct FerrisFetcherBuilder {
    config: Config,
    rules: Vec<ExtractionRule>,
}

impl FerrisFetcherBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            rules: Vec::new(),
        }
    }

    /// Set custom configuration
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Add an extraction rule
    pub fn add_rule(mut self, rule: ExtractionRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Add multiple extraction rules
    pub fn add_rules(mut self, rules: Vec<ExtractionRule>) -> Self {
        self.rules.extend(rules);
        self
    }

    /// Set user agent
    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.config = self.config.with_user_agent(user_agent);
        self
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config = self.config.with_timeout(timeout);
        self
    }

    /// Set maximum concurrent requests
    pub fn max_concurrent_requests(mut self, max: usize) -> Self {
        self.config = self.config.with_max_concurrent_requests(max);
        self
    }

    /// Disable rate limiting
    pub fn without_rate_limit(mut self) -> Self {
        self.config = self.config.without_rate_limit();
        self
    }

    /// Set retry policy
    pub fn retry_policy(mut self, retry_policy: crate::types::RetryPolicy) -> Self {
        self.config = self.config.with_retry_policy(retry_policy);
        self
    }

    /// Add custom header
    pub fn header(mut self, name: &str, value: &str) -> Result<Self> {
        self.config = self.config.with_header(name, value)?;
        Ok(self)
    }

    /// Set proxy
    pub fn proxy(mut self, proxy: url::Url) -> Self {
        self.config = self.config.with_proxy(proxy);
        self
    }

    /// Disable redirects
    pub fn without_redirects(mut self) -> Self {
        self.config = self.config.without_redirects();
        self
    }

    /// Disable cookies
    pub fn without_cookies(mut self) -> Self {
        self.config = self.config.without_cookies();
        self
    }

    /// Build the FerrisFetcher instance
    pub fn build(self) -> Result<FerrisFetcher> {
        FerrisFetcher::with_config_and_rules(self.config, self.rules)
    }
}

impl Default for FerrisFetcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ferrisfetcher_creation() {
        let fetcher = FerrisFetcher::new().unwrap();
        assert!(fetcher.has_rate_limiting());
        assert_eq!(fetcher.max_concurrent_requests(), 10);
    }

    // Note: Integration tests temporarily disabled due to mockito version compatibility
    // TODO: Update tests with compatible mocking library
}
