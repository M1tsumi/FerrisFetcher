//! Basic web scraping example
//! 
//! This example demonstrates the basic usage of FerrisFetcher to scrape a single page.

use ferrisfetcher::{FerrisFetcher, Config};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Create a new FerrisFetcher instance with custom configuration
    let config = Config::new()
        .with_user_agent("MyScraper/1.0")
        .with_timeout(Duration::from_secs(10))
        .with_max_concurrent_requests(5)
        .without_rate_limit(); // Disable rate limiting for this example

    let fetcher = FerrisFetcher::with_config(config)?;

    // Scrape a single URL
    let url = "https://example.com";
    println!("Scraping: {}", url);

    match fetcher.scrape(url).await {
        Ok(result) => {
            println!("✅ Scraping successful!");
            println!("📄 Title: {:?}", result.title);
            println!("🔗 Status Code: {}", result.status_code);
            println!("⏱️  Scrape Time: {}ms", result.scrape_time_ms);
            println!("📊 Content Length: {} bytes", result.content.len());
            
            // Print metadata
            println!("\n📋 Metadata:");
            for (key, value) in &result.metadata {
                println!("  {}: {}", key, value);
            }

            // Print some basic statistics
            let stats = fetcher.get_stats().await;
            println!("\n📈 Statistics:");
            println!("  Total Requests: {}", stats.total_requests);
            println!("  Successful: {}", stats.successful_requests);
            println!("  Failed: {}", stats.failed_requests);
            println!("  Success Rate: {:.2}%", stats.success_rate() * 100.0);
        }
        Err(e) => {
            eprintln!("❌ Scraping failed: {}", e);
        }
    }

    Ok(())
}
