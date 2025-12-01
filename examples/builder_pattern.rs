//! Builder pattern example
//! 
//! This example demonstrates how to use the fluent builder API to configure FerrisFetcher.

use ferrisfetcher::{FerrisFetcherBuilder, ExtractionRuleBuilder, ExtractionType};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Example 1: Simple builder usage
    println!("🏗️  Example 1: Simple builder configuration");
    
    let fetcher = FerrisFetcherBuilder::new()
        .user_agent("BuilderExample/1.0")
        .timeout(Duration::from_secs(10))
        .max_concurrent_requests(5)
        .without_rate_limit()
        .header("X-Custom-Header", "test-value")?
        .build()?;

    println!("✅ Created fetcher with custom configuration");
    println!("  User Agent: {}", fetcher.config().user_agent);
    println!("  Timeout: {:?}", fetcher.config().timeout);
    println!("  Max Concurrent: {}", fetcher.config().max_concurrent_requests);
    println!("  Rate Limiting: {:?}", fetcher.config().rate_limit);

    // Example 2: Advanced configuration with extraction rules
    println!("\n🏗️  Example 2: Advanced configuration with extraction rules");
    
    let advanced_fetcher = FerrisFetcherBuilder::new()
        .config(
            ferrisfetcher::Config::new()
                .with_user_agent("AdvancedScraper/2.0")
                .with_timeout(Duration::from_secs(30))
                .with_max_concurrent_requests(10)
                .with_retry_policy(ferrisfetcher::RetryPolicy {
                    max_attempts: 5,
                    base_delay: Duration::from_millis(500),
                    max_delay: Duration::from_secs(10),
                    exponential_backoff: true,
                    backoff_multiplier: 1.5,
                })
        )
        .add_rule(
            ExtractionRuleBuilder::new("titles", "h1, h2, h3")
                .extraction_type(ExtractionType::Text)
                .multiple(true)
                .build()
        )
        .add_rule(
            ExtractionRuleBuilder::new("external_links", "a[href^='http']")
                .extraction_type(ExtractionType::Attribute)
                .attribute("href")
                .multiple(true)
                .build()
        )
        .add_rule(
            ExtractionRuleBuilder::new("images", "img[src]")
                .extraction_type(ExtractionType::Attribute)
                .attribute("src")
                .multiple(true)
                .build()
        )
        .header("Accept-Language", "en-US,en;q=0.9")?
        .header("DNT", "1")?
        .build()?;

    println!("✅ Created advanced fetcher with extraction rules");
    println!("  Extraction Rules: {}", advanced_fetcher.extraction_rules().len());
    for rule_name in advanced_fetcher.extraction_rules().keys() {
        println!("    - {}", rule_name);
    }

    // Example 3: Proxy configuration
    println!("\n🏗️  Example 3: Configuration with proxy (if available)");
    
    // Note: This is just an example - you'll need to provide a real proxy URL
    if let Ok(proxy_url) = url::Url::parse("http://proxy.example.com:8080") {
        let proxy_fetcher = FerrisFetcherBuilder::new()
            .user_agent("ProxyScraper/1.0")
            .proxy(proxy_url)
            .without_cookies()
            .without_redirects()
            .build();

        match proxy_fetcher {
            Ok(fetcher) => {
                println!("✅ Created fetcher with proxy configuration");
                println!("  Proxy: {:?}", fetcher.config().proxy);
                println!("  Cookies: {}", fetcher.config().cookie_jar);
                println!("  Redirects: {}", fetcher.config().follow_redirects);
            }
            Err(e) => {
                println!("⚠️  Proxy configuration failed (expected if proxy is unavailable): {}", e);
            }
        }
    } else {
        println!("⚠️  Invalid proxy URL format");
    }

    // Example 4: Testing the configured fetcher
    println!("\n🧪 Example 4: Testing the configured fetcher");
    
    let test_url = "https://example.com";
    println!("🔍 Testing with URL: {}", test_url);

    match advanced_fetcher.scrape(test_url).await {
        Ok(result) => {
            println!("✅ Scraping successful!");
            println!("  Title: {:?}", result.title);
            println!("  Status: {}", result.status_code);
            println!("  Content Length: {} bytes", result.content.len());
            
            // Show extracted data
            println!("\n📊 Extracted Data:");
            for (field, values) in &result.extracted_data {
                println!("  {}: {} values", field, values.len());
                if values.len() <= 3 {
                    for value in values {
                        println!("    - {}", value);
                    }
                } else {
                    for value in values.iter().take(3) {
                        println!("    - {}", value);
                    }
                    println!("    ... and {} more", values.len() - 3);
                }
            }

            // Show metadata
            println!("\n🔍 Metadata:");
            for (key, value) in &result.metadata {
                if key != "json_ld" { // Skip JSON-LD for cleaner output
                    println!("  {}: {}", key, value);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Scraping failed: {}", e);
        }
    }

    // Example 5: Performance comparison
    println!("\n🏁 Example 5: Performance comparison");
    
    let urls = vec![
        "https://example.com",
        "https://example.org",
        "https://example.net",
    ];

    // Test with default configuration
    println!("📊 Testing with default configuration...");
    let default_fetcher = ferrisfetcher::FerrisFetcher::new()?;
    let start = std::time::Instant::now();
    
    match default_fetcher.scrape_multiple(&urls).await {
        Ok(results) => {
            let default_time = start.elapsed();
            println!("  ✅ Default: {} URLs in {:?}", results.len(), default_time);
        }
        Err(e) => {
            println!("  ❌ Default failed: {}", e);
        }
    }

    // Test with optimized configuration
    println!("📊 Testing with optimized configuration...");
    let start = std::time::Instant::now();
    
    match advanced_fetcher.scrape_multiple(&urls).await {
        Ok(results) => {
            let optimized_time = start.elapsed();
            println!("  ✅ Optimized: {} URLs in {:?}", results.len(), optimized_time);
        }
        Err(e) => {
            println!("  ❌ Optimized failed: {}", e);
        }
    }

    println!("\n🎯 Builder pattern examples completed!");
    Ok(())
}
