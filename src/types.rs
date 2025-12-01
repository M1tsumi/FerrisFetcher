//! Common data types and structures for FerrisFetcher

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Main structure containing scraped data from a web page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedData {
    /// The URL that was scraped
    pub url: String,
    /// Page title if available
    pub title: Option<String>,
    /// Raw HTML content
    pub content: String,
    /// Extracted metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Structured data extracted using rules
    pub extracted_data: HashMap<String, Vec<String>>,
    /// When the scraping occurred
    pub timestamp: DateTime<Utc>,
    /// HTTP status code
    pub status_code: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Time taken to scrape (in milliseconds)
    pub scrape_time_ms: u64,
}

impl ScrapedData {
    /// Create new scraped data
    pub fn new(url: String) -> Self {
        Self {
            url,
            title: None,
            content: String::new(),
            metadata: HashMap::new(),
            extracted_data: HashMap::new(),
            timestamp: Utc::now(),
            status_code: 0,
            headers: HashMap::new(),
            scrape_time_ms: 0,
        }
    }
    
    /// Add extracted data with a key
    pub fn add_extracted_data(&mut self, key: &str, values: Vec<String>) {
        self.extracted_data.insert(key.to_string(), values);
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
    }
    
    /// Get all extracted values for a key
    pub fn get_extracted_values(&self, key: &str) -> Option<&Vec<String>> {
        self.extracted_data.get(key)
    }
    
    /// Get the first extracted value for a key
    pub fn get_first_value(&self, key: &str) -> Option<&String> {
        self.extracted_data.get(key).and_then(|values| values.first())
    }
}

/// Configuration for retry policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(10000),
            exponential_backoff: true,
            backoff_multiplier: 2.0,
        }
    }
}

/// Rule for extracting data from HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRule {
    /// Name of the extraction rule
    pub name: String,
    /// CSS selector to target elements
    pub selector: String,
    /// What to extract from matched elements
    pub extraction_type: ExtractionType,
    /// Whether to extract multiple values or just the first
    pub multiple: bool,
    /// Optional attribute to extract (for Attribute extraction type)
    pub attribute: Option<String>,
}

/// Types of data extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionType {
    /// Extract text content
    Text,
    /// Extract HTML content
    Html,
    /// Extract a specific attribute
    Attribute,
    /// Extract the element's own HTML
    OuterHtml,
}

/// HTTP method types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

/// Request statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestStats {
    /// Total number of requests made
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Total bytes downloaded
    pub total_bytes: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Total time spent scraping
    pub total_time_ms: u64,
}

impl Default for RequestStats {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestStats {
    /// Create new request stats
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_bytes: 0,
            avg_response_time_ms: 0.0,
            total_time_ms: 0,
        }
    }
    
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Number of requests allowed per period
    pub requests_per_period: u32,
    /// Time period for rate limiting
    pub period: Duration,
    /// Delay between requests to stay within limits
    pub delay_between_requests: Duration,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests_per_period: 10,
            period: Duration::from_secs(60),
            delay_between_requests: Duration::from_millis(1000),
        }
    }
}
