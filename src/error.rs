//! Error types for FerrisFetcher

use thiserror::Error;
use std::time::Duration;

/// Main error type for FerrisFetcher operations
#[derive(Error, Debug)]
pub enum FerrisFetcherError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("HTML parsing failed: {0}")]
    ParseError(String),
    
    #[error("Data extraction failed: {0}")]
    ExtractionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Rate limit exceeded: retry after {0:?}")]
    RateLimitExceeded(Duration),
    
    #[error("Timeout error: operation timed out after {0:?}")]
    TimeoutError(Duration),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("All retry attempts failed")]
    RetryExhausted,
    
    #[error("Task cancelled")]
    TaskCancelled,
    
    #[error("Invalid selector: {0}")]
    InvalidSelector(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, FerrisFetcherError>;

impl FerrisFetcherError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            FerrisFetcherError::HttpError(_) => true,
            FerrisFetcherError::NetworkError(_) => true,
            FerrisFetcherError::TimeoutError(_) => true,
            FerrisFetcherError::RateLimitExceeded(_) => true,
            FerrisFetcherError::TaskCancelled => false,
            FerrisFetcherError::ParseError(_) => false,
            FerrisFetcherError::ExtractionError(_) => false,
            FerrisFetcherError::ConfigError(_) => false,
            FerrisFetcherError::InvalidUrl(_) => false,
            FerrisFetcherError::IoError(_) => false,
            FerrisFetcherError::JsonError(_) => false,
            FerrisFetcherError::RetryExhausted => false,
            FerrisFetcherError::InvalidSelector(_) => false,
        }
    }
    
    /// Get a human-readable error category
    pub fn category(&self) -> &'static str {
        match self {
            FerrisFetcherError::HttpError(_) => "HTTP",
            FerrisFetcherError::ParseError(_) => "Parsing",
            FerrisFetcherError::ExtractionError(_) => "Extraction",
            FerrisFetcherError::ConfigError(_) => "Configuration",
            FerrisFetcherError::RateLimitExceeded(_) => "Rate Limit",
            FerrisFetcherError::TimeoutError(_) => "Timeout",
            FerrisFetcherError::InvalidUrl(_) => "URL",
            FerrisFetcherError::IoError(_) => "IO",
            FerrisFetcherError::JsonError(_) => "JSON",
            FerrisFetcherError::RetryExhausted => "Retry",
            FerrisFetcherError::TaskCancelled => "Cancellation",
            FerrisFetcherError::InvalidSelector(_) => "Selector",
            FerrisFetcherError::NetworkError(_) => "Network",
        }
    }
}
