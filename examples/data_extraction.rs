//! Data extraction example
//! 
//! This example demonstrates how to use extraction rules to extract structured data from web pages.

use ferrisfetcher::{
    FerrisFetcher, Config, 
    ExtractionRuleBuilder, ExtractionType,
    presets
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Create fetcher with article extraction rules
    let fetcher = FerrisFetcher::with_config_and_rules(
        Config::new()
            .with_user_agent("ArticleScraper/1.0")
            .with_timeout(Duration::from_secs(15)),
        presets::article() // Use predefined article extraction rules
    )?;

    // Example URL (you can replace this with any news article URL)
    let url = "https://example.com/news/article";
    println!("🔍 Extracting article data from: {}", url);

    match fetcher.scrape(url).await {
        Ok(result) => {
            println!("✅ Successfully extracted article data!");
            
            // Print basic page info
            println!("\n📄 Page Information:");
            println!("  Title: {:?}", result.title);
            println!("  URL: {}", result.url);
            println!("  Status: {}", result.status_code);

            // Print extracted structured data
            println!("\n📊 Extracted Article Data:");
            for (field_name, values) in &result.extracted_data {
                println!("  {}: {:?}", field_name, values);
            }

            // Print metadata
            println!("\n🔍 Metadata:");
            for (key, value) in &result.metadata {
                println!("  {}: {}", key, value);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to extract data: {}", e);
        }
    }

    // Example with custom extraction rules
    println!("\n🛠️  Custom extraction example:");
    
    let mut custom_fetcher = FerrisFetcher::new()?;
    
    // Add custom extraction rules
    custom_fetcher.add_extraction_rule(
        ExtractionRuleBuilder::new("headings", "h1, h2, h3")
            .extraction_type(ExtractionType::Text)
            .multiple(true)
            .build()
    );
    
    custom_fetcher.add_extraction_rule(
        ExtractionRuleBuilder::new("links", "a[href]")
            .extraction_type(ExtractionType::Attribute)
            .attribute("href")
            .multiple(true)
            .build()
    );

    match custom_fetcher.scrape("https://example.com").await {
        Ok(result) => {
            println!("✅ Custom extraction successful!");
            
            if let Some(headings) = result.extracted_data.get("headings") {
                println!("\n📝 Found {} headings:", headings.len());
                for (i, heading) in headings.iter().enumerate() {
                    println!("  {}. {}", i + 1, heading);
                }
            }
            
            if let Some(links) = result.extracted_data.get("links") {
                println!("\n🔗 Found {} links:", links.len());
                for (i, link) in links.iter().take(5).enumerate() {
                    println!("  {}. {}", i + 1, link);
                }
                if links.len() > 5 {
                    println!("  ... and {} more", links.len() - 5);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Custom extraction failed: {}", e);
        }
    }

    Ok(())
}
