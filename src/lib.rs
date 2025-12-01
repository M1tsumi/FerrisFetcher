//! FerrisFetcher - A cutting-edge, high-level web scraping library crafted in Rust
//! 
//! This library provides a powerful, async web scraping solution with:
//! - High-performance HTTP client with connection pooling
//! - Robust HTML parsing and CSS selector support
//! - Concurrent scraping operations
//! - Configurable rate limiting and respectful scraping practices
//! - Flexible data extraction and export capabilities
//! 
//! # Quick Start
//! 
//! ```rust
//! use ferrisfetcher::{FerrisFetcher, Config};
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let fetcher = FerrisFetcher::new();
//!     let result = fetcher.scrape("https://example.com").await?;
//!     println!("Title: {}", result.title.unwrap_or_default());
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod extractor;
pub mod html_parser;
pub mod scraper;
pub mod types;

pub use client::HttpClient;
pub use config::Config;
pub use error::{FerrisFetcherError, Result};
pub use extractor::{DataExtractor, ExtractionRuleBuilder, presets};
pub use html_parser::HtmlParser;
pub use scraper::{FerrisFetcher, FerrisFetcherBuilder};
pub use types::{ScrapedData, ExtractionRule, ExtractionType, RetryPolicy, HttpMethod, RequestStats, RateLimit};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
