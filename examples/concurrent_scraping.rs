//! Concurrent scraping example
//! 
//! This example demonstrates how to scrape multiple URLs concurrently with progress reporting.

use ferrisfetcher::{FerrisFetcher, Config};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Create fetcher with configuration for concurrent scraping
    let config = Config::new()
        .with_user_agent("ConcurrentScraper/1.0")
        .with_timeout(Duration::from_secs(20))
        .with_max_concurrent_requests(8)
        .without_rate_limit(); // Disable rate limiting for demo purposes

    let fetcher = FerrisFetcher::with_config(config)?;

    // Example URLs to scrape (replace with actual URLs you want to scrape)
    let urls = vec![
        "https://example.com",
        "https://example.org",
        "https://example.net",
        "https://httpbin.org/html",
        "https://httpbin.org/json",
    ];

    println!("🚀 Starting concurrent scraping of {} URLs", urls.len());
    println!("⚙️  Configuration:");
    println!("  Max Concurrent Requests: {}", fetcher.max_concurrent_requests());
    println!("  Rate Limiting: {}", if fetcher.has_rate_limiting() { "Enabled" } else { "Disabled" });

    let start_time = std::time::Instant::now();

    // Method 1: Simple concurrent scraping
    println!("\n📊 Method 1: Simple concurrent scraping");
    match fetcher.scrape_multiple(&urls).await {
        Ok(results) => {
            let elapsed = start_time.elapsed();
            println!("✅ Completed scraping {} URLs in {:?}", results.len(), elapsed);
            
            // Print summary
            println!("\n📋 Summary:");
            for (i, result) in results.iter().enumerate() {
                println!("  {}. {} - {} ({}ms)", 
                    i + 1, 
                    result.url, 
                    result.title.as_deref().unwrap_or("No title"),
                    result.scrape_time_ms
                );
            }

            // Print statistics
            let stats = fetcher.get_stats().await;
            println!("\n📈 Statistics:");
            println!("  Total Requests: {}", stats.total_requests);
            println!("  Successful: {}", stats.successful_requests);
            println!("  Failed: {}", stats.failed_requests);
            println!("  Success Rate: {:.2}%", stats.success_rate() * 100.0);
            println!("  Average Response Time: {:.2}ms", stats.avg_response_time_ms);
            println!("  Total Bytes: {} bytes", stats.total_bytes);
        }
        Err(e) => {
            eprintln!("❌ Concurrent scraping failed: {}", e);
        }
    }

    // Reset stats for next example
    fetcher.reset_stats().await;

    // Method 2: Concurrent scraping with progress reporting
    println!("\n📊 Method 2: Concurrent scraping with progress reporting");
    
    let progress_callback = Arc::new(|completed: usize, total: usize, data: &ferrisfetcher::ScrapedData| {
        let percentage = (completed as f64 / total as f64) * 100.0;
        println!("  Progress: {}/{} ({:.1}%) - {} - {}", 
            completed, total, percentage, 
            data.url, 
            data.title.as_deref().unwrap_or("No title")
        );
    });

    let start_time = std::time::Instant::now();
    
    // We need to clone the callback for the async call
    let callback = progress_callback.clone();
    match fetcher.scrape_multiple_with_progress(&urls, move |completed, total, data| {
        callback(completed, total, data);
    }).await {
        Ok(results) => {
            let elapsed = start_time.elapsed();
            println!("\n✅ Progress-based scraping completed {} URLs in {:?}", results.len(), elapsed);
        }
        Err(e) => {
            eprintln!("❌ Progress-based scraping failed: {}", e);
        }
    }

    // Method 3: Batch scraping with error handling
    println!("\n📊 Method 3: Batch scraping with individual error handling");
    
    fetcher.reset_stats().await;
    let start_time = std::time::Instant::now();
    
    // Process URLs individually to handle errors per URL
    let mut successful_results = Vec::new();
    let mut failed_urls = Vec::new();

    for (i, url) in urls.iter().enumerate() {
        println!("  Scraping {}/{}: {}", i + 1, urls.len(), url);
        
        match fetcher.scrape(url).await {
            Ok(result) => {
                println!("    ✅ Success - {}", result.title.as_deref().unwrap_or("No title"));
                successful_results.push(result);
            }
            Err(e) => {
                println!("    ❌ Failed - {}", e);
                failed_urls.push((url, e));
            }
        }
    }

    let elapsed = start_time.elapsed();
    println!("\n📋 Batch Results:");
    println!("  ✅ Successful: {}", successful_results.len());
    println!("  ❌ Failed: {}", failed_urls.len());
    println!("  ⏱️  Total Time: {:?}", elapsed);

    if !failed_urls.is_empty() {
        println!("\n❌ Failed URLs:");
        for (url, error) in failed_urls {
            println!("  {}: {}", url, error);
        }
    }

    // Final statistics
    let final_stats = fetcher.get_stats().await;
    println!("\n📈 Final Statistics:");
    println!("  Total Requests: {}", final_stats.total_requests);
    println!("  Successful: {}", final_stats.successful_requests);
    println!("  Failed: {}", final_stats.failed_requests);
    println!("  Success Rate: {:.2}%", final_stats.success_rate() * 100.0);
    println!("  Average Response Time: {:.2}ms", final_stats.avg_response_time_ms);

    Ok(())
}
