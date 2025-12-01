//! Comprehensive demonstration of FerrisFetcher capabilities
//! 
//! This example showcases all major features of the FerrisFetcher library.

use ferrisfetcher::{
    FerrisFetcherBuilder, ExtractionRuleBuilder, ExtractionType,
    Config, RetryPolicy, RateLimit
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("🦀 FerrisFetcher Comprehensive Demo");
    println!("=====================================");

    // Example 1: Basic scraping with default configuration
    println!("\n📄 Example 1: Basic Web Scraping");
    basic_scraping().await?;

    // Example 2: Advanced scraping with custom configuration
    println!("\n⚙️  Example 2: Advanced Configuration");
    advanced_configuration().await?;

    // Example 3: Data extraction with custom rules
    println!("\n🎯 Example 3: Custom Data Extraction");
    custom_extraction().await?;

    // Example 4: Concurrent scraping with progress
    println!("\n🚀 Example 4: Concurrent Scraping");
    concurrent_scraping().await?;

    // Example 5: Error handling and retries
    println!("\n🛡️  Example 5: Error Handling & Retries");
    error_handling().await?;

    // Example 6: Builder pattern demonstration
    println!("\n🏗️  Example 6: Builder Pattern");
    builder_pattern().await?;

    println!("\n✨ Demo completed successfully!");
    Ok(())
}

async fn basic_scraping() -> Result<(), Box<dyn std::error::Error>> {
    println!("  Scraping example.com with default settings...");
    
    let fetcher = FerrisFetcherBuilder::new().build()?;
    let result = fetcher.scrape("https://example.com").await?;
    
    println!("  ✅ Status: {}", result.status_code);
    println!("  📝 Title: {:?}", result.title);
    println!("  📊 Content size: {} bytes", result.content.len());
    println!("  ⏱️  Scrape time: {}ms", result.scrape_time_ms);
    
    // Show metadata
    println!("  📋 Metadata:");
    for (key, value) in &result.metadata {
        println!("    {}: {}", key, value);
    }
    
    Ok(())
}

async fn advanced_configuration() -> Result<(), Box<dyn std::error::Error>> {
    println!("  Using custom configuration with retry policy...");
    
    let config = Config::new()
        .with_user_agent("FerrisFetcher-Demo/1.0")
        .with_timeout(Duration::from_secs(15))
        .with_max_concurrent_requests(5)
        .with_retry_policy(RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            exponential_backoff: true,
            backoff_multiplier: 2.0,
        })
        .with_rate_limit(RateLimit {
            requests_per_period: 2,
            period: Duration::from_secs(1),
            delay_between_requests: Duration::from_millis(500),
        });

    let fetcher = FerrisFetcherBuilder::new()
        .config(config)
        .header("X-Demo-Header", "comprehensive-demo")?
        .build()?;

    let result = fetcher.scrape("https://httpbin.org/user-agent").await?;
    println!("  ✅ Scraped with custom config");
    let content_preview = if result.content.len() > 100 {
        format!("{}...", &result.content[..100])
    } else {
        result.content.clone()
    };
    println!("  📝 Response preview: {}", content_preview);
    
    // Show statistics
    let stats = fetcher.get_stats().await;
    println!("  📈 Stats: {} requests, {} successful, {:.1}% success rate", 
        stats.total_requests, stats.successful_requests, stats.success_rate() * 100.0);
    
    Ok(())
}

async fn custom_extraction() -> Result<(), Box<dyn std::error::Error>> {
    println!("  Extracting structured data with custom rules...");
    
    let fetcher = FerrisFetcherBuilder::new()
        .add_rule(
            ExtractionRuleBuilder::new("headings", "h1, h2, h3")
                .extraction_type(ExtractionType::Text)
                .multiple(true)
                .build()
        )
        .add_rule(
            ExtractionRuleBuilder::new("links", "a[href]")
                .extraction_type(ExtractionType::Attribute)
                .attribute("href")
                .multiple(true)
                .build()
        )
        .add_rule(
            ExtractionRuleBuilder::new("main_content", "body")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build()
        )
        .build()?;

    let result = fetcher.scrape("https://example.com").await?;
    
    println!("  ✅ Extracted data fields:");
    for (field_name, values) in &result.extracted_data {
        println!("    📁 {}: {} value(s)", field_name, values.len());
        for (i, value) in values.iter().take(3).enumerate() {
            let preview = if value.len() > 50 {
                // Safe UTF-8 character boundary slicing
                let mut end = 50;
                while !value.is_char_boundary(end) && end > 0 {
                    end -= 1;
                }
                if end == 0 {
                    value.to_string()
                } else {
                    format!("{}...", &value[..end])
                }
            } else {
                value.clone()
            };
            println!("      {}. {}", i + 1, preview);
        }
        if values.len() > 3 {
            println!("      ... and {} more", values.len() - 3);
        }
    }
    
    Ok(())
}

async fn concurrent_scraping() -> Result<(), Box<dyn std::error::Error>> {
    println!("  Scraping multiple URLs concurrently with progress...");
    
    let urls = vec![
        "https://example.com",
        "https://example.org",
        "https://example.net",
        "https://httpbin.org/html",
        "https://httpbin.org/json",
    ];

    let fetcher = FerrisFetcherBuilder::new()
        .max_concurrent_requests(3)
        .without_rate_limit()
        .build()?;

    let start_time = std::time::Instant::now();
    
    let results = fetcher.scrape_multiple_with_progress(&urls, |completed, total, data| {
        let percentage = (completed as f64 / total as f64) * 100.0;
        println!("    📊 Progress: {}/{} ({:.1}%) - {} - {}ms", 
            completed, total, percentage, 
            data.url, data.scrape_time_ms);
    }).await?;

    let elapsed = start_time.elapsed();
    
    println!("  ✅ Completed scraping {} URLs in {:?}", results.len(), elapsed);
    
    // Show summary
    println!("  📋 Summary:");
    for (i, result) in results.iter().enumerate() {
        println!("    {}. {} - {} ({}ms)", 
            i + 1, 
            result.url, 
            result.title.as_deref().unwrap_or("No title"),
            result.scrape_time_ms
        );
    }
    
    Ok(())
}

async fn error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("  Demonstrating error handling with invalid URLs...");
    
    let fetcher = FerrisFetcherBuilder::new()
        .timeout(Duration::from_secs(5))
        .build()?;

    // Test with invalid URL
    match fetcher.scrape("https://invalid-domain-that-does-not-exist.com").await {
        Ok(_) => println!("  ⚠️  Unexpected success with invalid URL"),
        Err(e) => println!("  ✅ Properly handled error: {}", e),
    }

    // Test with timeout
    match fetcher.scrape("https://httpbin.org/delay/10").await {
        Ok(_) => println!("  ⚠️  Unexpected success with delayed URL"),
        Err(e) => println!("  ✅ Properly handled timeout: {}", e),
    }

    // Show final statistics
    let stats = fetcher.get_stats().await;
    println!("  📈 Final stats: {} total, {} successful, {} failed", 
        stats.total_requests, stats.successful_requests, stats.failed_requests);
    
    Ok(())
}

async fn builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("  Using fluent builder API for complex configuration...");
    
    let fetcher = FerrisFetcherBuilder::new()
        .user_agent("FerrisFetcher-Builder-Demo/1.0")
        .timeout(Duration::from_secs(20))
        .max_concurrent_requests(8)
        .without_rate_limit()
        .header("X-Custom-Header", "builder-demo")?
        .header("X-Feature", "comprehensive")?
        .add_rule(
            ExtractionRuleBuilder::new("page_title", "title")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build()
        )
        .add_rule(
            ExtractionRuleBuilder::new("all_links", "a[href]")
                .extraction_type(ExtractionType::Attribute)
                .attribute("href")
                .multiple(true)
                .build()
        )
        .build()?;

    println!("  ✅ Built fetcher with configuration:");
    println!("    🤖 User Agent: {}", fetcher.config().user_agent);
    println!("    ⏱️  Timeout: {:?}", fetcher.config().timeout);
    println!("    🚀 Max Concurrent: {}", fetcher.config().max_concurrent_requests);
    println!("    📋 Extraction Rules: {}", fetcher.extraction_rules().len());

    let result = fetcher.scrape("https://example.com").await?;
    
    println!("  🎯 Extracted data:");
    for (field, values) in &result.extracted_data {
        println!("    📁 {}: {} values", field, values.len());
    }
    
    Ok(())
}
