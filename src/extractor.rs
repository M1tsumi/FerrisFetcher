//! Data extraction engine for structured data extraction

use crate::error::{FerrisFetcherError, Result};
use crate::html_parser::HtmlParser;
use crate::types::{ExtractionRule, ExtractionType};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Data extraction engine with configurable rules
#[derive(Debug, Clone)]
pub struct DataExtractor {
    /// Extraction rules indexed by name
    rules: HashMap<String, ExtractionRule>,
}

impl DataExtractor {
    /// Create a new data extractor
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Create a data extractor with predefined rules
    pub fn with_rules(rules: Vec<ExtractionRule>) -> Self {
        let mut extractor = Self::new();
        for rule in rules {
            extractor.add_rule(rule);
        }
        extractor
    }

    /// Add an extraction rule
    pub fn add_rule(&mut self, rule: ExtractionRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Remove an extraction rule by name
    pub fn remove_rule(&mut self, name: &str) -> Option<ExtractionRule> {
        self.rules.remove(name)
    }

    /// Get an extraction rule by name
    pub fn get_rule(&self, name: &str) -> Option<&ExtractionRule> {
        self.rules.get(name)
    }

    /// Get all extraction rules
    pub fn rules(&self) -> &HashMap<String, ExtractionRule> {
        &self.rules
    }

    /// Extract data using all configured rules
    pub fn extract_all(&self, parser: &HtmlParser) -> Result<HashMap<String, Vec<String>>> {
        let mut results = HashMap::new();
        
        for (name, rule) in &self.rules {
            match self.extract_by_rule(parser, rule) {
                Ok(values) => {
                    if !values.is_empty() {
                        results.insert(name.clone(), values.clone());
                        debug!("Extracted {} values for rule '{}'", values.len(), name);
                    }
                }
                Err(e) => {
                    warn!("Failed to extract data for rule '{}': {}", name, e);
                }
            }
        }
        
        info!("Extracted data for {} rules", results.len());
        Ok(results)
    }

    /// Extract data using a specific rule
    pub fn extract_by_rule(&self, parser: &HtmlParser, rule: &ExtractionRule) -> Result<Vec<String>> {
        debug!("Extracting data with rule '{}' using selector '{}'", rule.name, rule.selector);
        
        let values = match rule.extraction_type {
            ExtractionType::Text => {
                if rule.multiple {
                    parser.select_text(&rule.selector)?
                } else {
                    parser.select_first_text(&rule.selector)
                        .map(|text| vec![text])
                        .unwrap_or_default()
                }
            }
            ExtractionType::Html => {
                if rule.multiple {
                    parser.select_html(&rule.selector)?
                } else {
                    parser.select_first_html(&rule.selector)
                        .map(|html| vec![html])
                        .unwrap_or_default()
                }
            }
            ExtractionType::Attribute => {
                let attr_name = rule.attribute.as_ref()
                    .ok_or_else(|| FerrisFetcherError::ExtractionError(
                        format!("Attribute extraction requires attribute name for rule '{}'", rule.name)
                    ))?;
                
                if rule.multiple {
                    parser.select_attr(&rule.selector, attr_name)?
                } else {
                    parser.select_first_attr(&rule.selector, attr_name)
                        .map(|attr| vec![attr])
                        .unwrap_or_default()
                }
            }
            ExtractionType::OuterHtml => {
                if rule.multiple {
                    parser.select_outer_html(&rule.selector)?
                } else {
                    parser.select_first(&rule.selector)
                        .map(|element| element.html())
                        .map(|html| vec![html])
                        .unwrap_or_default()
                }
            }
        };

        Ok(values)
    }

    /// Extract data by rule name
    pub fn extract_by_name(&self, parser: &HtmlParser, rule_name: &str) -> Result<Vec<String>> {
        let rule = self.rules.get(rule_name)
            .ok_or_else(|| FerrisFetcherError::ExtractionError(
                format!("Extraction rule '{}' not found", rule_name)
            ))?;
        
        self.extract_by_rule(parser, rule)
    }

    /// Extract a single value by rule name (convenience method)
    pub fn extract_single(&self, parser: &HtmlParser, rule_name: &str) -> Option<String> {
        self.extract_by_name(parser, rule_name)
            .ok()
            .and_then(|values| values.into_iter().next())
    }

    /// Extract text content using a CSS selector (convenience method)
    pub fn extract_text(&self, parser: &HtmlParser, selector: &str, multiple: bool) -> Result<Vec<String>> {
        let rule = ExtractionRule {
            name: format!("temp_text_{}", selector.len()),
            selector: selector.to_string(),
            extraction_type: ExtractionType::Text,
            multiple,
            attribute: None,
        };
        
        self.extract_by_rule(parser, &rule)
    }

    /// Extract attribute values using a CSS selector (convenience method)
    pub fn extract_attr(&self, parser: &HtmlParser, selector: &str, attr: &str, multiple: bool) -> Result<Vec<String>> {
        let rule = ExtractionRule {
            name: format!("temp_attr_{}", selector.len()),
            selector: selector.to_string(),
            extraction_type: ExtractionType::Attribute,
            multiple,
            attribute: Some(attr.to_string()),
        };
        
        self.extract_by_rule(parser, &rule)
    }

    /// Check if a rule exists
    pub fn has_rule(&self, name: &str) -> bool {
        self.rules.contains_key(name)
    }

    /// Get the number of configured rules
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Clear all rules
    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// Validate all rules
    pub fn validate_rules(&self) -> Result<()> {
        for (name, rule) in &self.rules {
            if rule.selector.is_empty() {
                return Err(FerrisFetcherError::ExtractionError(
                    format!("Rule '{}' has empty selector", name)
                ));
            }
            
            if rule.name.is_empty() {
                return Err(FerrisFetcherError::ExtractionError(
                    "Rule has empty name".to_string()
                ));
            }
            
            if matches!(rule.extraction_type, ExtractionType::Attribute) && rule.attribute.is_none() {
                return Err(FerrisFetcherError::ExtractionError(
                    format!("Rule '{}' with Attribute extraction type requires attribute name", name)
                ));
            }
        }
        
        Ok(())
    }
}

impl Default for DataExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating extraction rules
pub struct ExtractionRuleBuilder {
    name: String,
    selector: String,
    extraction_type: ExtractionType,
    multiple: bool,
    attribute: Option<String>,
}

impl ExtractionRuleBuilder {
    /// Create a new extraction rule builder
    pub fn new(name: &str, selector: &str) -> Self {
        Self {
            name: name.to_string(),
            selector: selector.to_string(),
            extraction_type: ExtractionType::Text,
            multiple: false,
            attribute: None,
        }
    }

    /// Set the extraction type
    pub fn extraction_type(mut self, extraction_type: ExtractionType) -> Self {
        self.extraction_type = extraction_type;
        self
    }

    /// Set whether to extract multiple values
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Set the attribute name (for Attribute extraction type)
    pub fn attribute(mut self, attribute: &str) -> Self {
        self.attribute = Some(attribute.to_string());
        self
    }

    /// Build the extraction rule
    pub fn build(self) -> ExtractionRule {
        ExtractionRule {
            name: self.name,
            selector: self.selector,
            extraction_type: self.extraction_type,
            multiple: self.multiple,
            attribute: self.attribute,
        }
    }
}

/// Predefined extraction rule sets for common use cases
pub mod presets {
    use super::*;

    /// Create rules for basic article extraction
    pub fn article() -> Vec<ExtractionRule> {
        vec![
            ExtractionRuleBuilder::new("title", "h1, .title, .headline")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("author", ".author, [rel='author'], .byline")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("publish_date", ".date, .published, time[datetime], .timestamp")
                .extraction_type(ExtractionType::Attribute)
                .attribute("datetime")
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("content", ".content, .article-body, .post-content, main")
                .extraction_type(ExtractionType::Text)
                .multiple(true)
                .build(),
            ExtractionRuleBuilder::new("summary", ".summary, .excerpt, .description")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
        ]
    }

    /// Create rules for product extraction
    pub fn product() -> Vec<ExtractionRule> {
        vec![
            ExtractionRuleBuilder::new("product_name", ".product-title, .product-name, h1")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("price", ".price, .product-price, [itemprop='price']")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("description", ".product-description, .description, [itemprop='description']")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("image", ".product-image img, [itemprop='image']")
                .extraction_type(ExtractionType::Attribute)
                .attribute("src")
                .multiple(true)
                .build(),
            ExtractionRuleBuilder::new("availability", ".availability, .stock, [itemprop='availability']")
                .extraction_type(ExtractionType::Attribute)
                .attribute("content")
                .multiple(false)
                .build(),
        ]
    }

    /// Create rules for social media extraction
    pub fn social_media() -> Vec<ExtractionRule> {
        vec![
            ExtractionRuleBuilder::new("post_text", ".post-content, .tweet-text, .message")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("author", ".author, .username, .user-name")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("timestamp", ".timestamp, .time, time")
                .extraction_type(ExtractionType::Attribute)
                .attribute("datetime")
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("likes", ".likes, .like-count, [aria-label*='like']")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("comments", ".comments, .comment-count, [aria-label*='comment']")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_parser::HtmlParser;

    #[test]
    fn test_data_extractor_creation() {
        let extractor = DataExtractor::new();
        assert_eq!(extractor.rule_count(), 0);
    }

    #[test]
    fn test_add_rule() {
        let mut extractor = DataExtractor::new();
        let rule = ExtractionRuleBuilder::new("test", "p")
            .extraction_type(ExtractionType::Text)
            .multiple(true)
            .build();
        
        extractor.add_rule(rule);
        assert_eq!(extractor.rule_count(), 1);
        assert!(extractor.has_rule("test"));
    }

    #[test]
    fn test_extract_text() {
        let html = r#"
        <div>
            <p>First paragraph</p>
            <p>Second paragraph</p>
        </div>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let extractor = DataExtractor::new();
        
        let result = extractor.extract_text(&parser, "p", true).unwrap();
        assert_eq!(result, vec!["First paragraph", "Second paragraph"]);
        
        let result = extractor.extract_text(&parser, "p", false).unwrap();
        assert_eq!(result, vec!["First paragraph"]);
    }

    #[test]
    fn test_extract_attr() {
        let html = r#"
        <div>
            <a href="https://example.com">Link 1</a>
            <a href="https://test.com">Link 2</a>
        </div>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let extractor = DataExtractor::new();
        
        let result = extractor.extract_attr(&parser, "a[href]", "href", true).unwrap();
        assert_eq!(result, vec!["https://example.com", "https://test.com"]);
    }

    #[test]
    fn test_extract_by_rule() {
        let html = r#"
        <div>
            <h1>Article Title</h1>
            <p>Article content goes here</p>
        </div>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let rule = ExtractionRuleBuilder::new("title", "h1")
            .extraction_type(ExtractionType::Text)
            .multiple(false)
            .build();
        
        let extractor = DataExtractor::with_rules(vec![rule]);
        let result = extractor.extract_by_name(&parser, "title").unwrap();
        assert_eq!(result, vec!["Article Title"]);
    }

    #[test]
    fn test_extract_all() {
        let html = r#"
        <article>
            <h1>Article Title</h1>
            <div class="content">Article content</div>
        </article>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let rules = vec![
            ExtractionRuleBuilder::new("title", "h1")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
            ExtractionRuleBuilder::new("content", ".content")
                .extraction_type(ExtractionType::Text)
                .multiple(false)
                .build(),
        ];
        
        let extractor = DataExtractor::with_rules(rules);
        let results = extractor.extract_all(&parser).unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results.get("title").unwrap(), &vec!["Article Title".to_string()]);
        assert_eq!(results.get("content").unwrap(), &vec!["Article content".to_string()]);
    }

    #[test]
    fn test_article_preset() {
        let html = r#"
        <article>
            <h1>Test Article</h1>
            <div class="author">John Doe</div>
            <time datetime="2023-01-01">January 1, 2023</time>
            <div class="content">Article content here</div>
        </article>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let extractor = DataExtractor::with_rules(presets::article());
        let results = extractor.extract_all(&parser).unwrap();
        
        assert_eq!(results.get("title").unwrap(), &vec!["Test Article".to_string()]);
        assert_eq!(results.get("author").unwrap(), &vec!["John Doe".to_string()]);
    }

    #[test]
    fn test_invalid_rule() {
        let extractor = DataExtractor::new();
        let html = "<div>Test</div>";
        let parser = HtmlParser::new(html).unwrap();
        
        let result = extractor.extract_by_name(&parser, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rules() {
        let mut extractor = DataExtractor::new();
        
        // Valid rule
        let valid_rule = ExtractionRuleBuilder::new("test", "p")
            .extraction_type(ExtractionType::Text)
            .build();
        extractor.add_rule(valid_rule);
        assert!(extractor.validate_rules().is_ok());
        
        // Invalid rule (empty selector)
        let invalid_rule = ExtractionRuleBuilder::new("invalid", "")
            .extraction_type(ExtractionType::Text)
            .build();
        extractor.add_rule(invalid_rule);
        assert!(extractor.validate_rules().is_err());
    }
}
