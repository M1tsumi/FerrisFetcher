//! HTML parsing module with CSS selector support

use crate::error::{FerrisFetcherError, Result};
use scraper::{Html, ElementRef, Selector};

/// HTML parser with CSS selector capabilities
#[derive(Debug, Clone)]
pub struct HtmlParser {
    /// Parsed HTML document
    document: Html,
}

impl HtmlParser {
    /// Create a new HTML parser from raw HTML content
    pub fn new(html: &str) -> Result<Self> {
        let document = Html::parse_document(html);
        
        Ok(Self {
            document,
        })
    }

    /// Parse HTML from a string
    pub fn parse(html: &str) -> Result<Self> {
        Self::new(html)
    }

    /// Get the page title
    pub fn title(&self) -> Option<String> {
        let title_selector = Selector::parse("title").ok()?;
        
        self.document
            .select(&title_selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .filter(|title| !title.is_empty())
    }

    /// Get meta tags by name
    pub fn meta_tag(&self, name: &str) -> Option<String> {
        let selector_str = &format!("meta[name='{}']", name);
        let selector = Selector::parse(selector_str).ok()?;

        self.document
            .select(&selector)
            .next()
            .and_then(|element| element.value().attr("content"))
            .map(|content| content.to_string())
    }

    /// Get meta tags by property (for Open Graph, Twitter Cards, etc.)
    pub fn meta_property(&self, property: &str) -> Option<String> {
        let selector_str = &format!("meta[property='{}']", property);
        let selector = Selector::parse(selector_str).ok()?;

        self.document
            .select(&selector)
            .next()
            .and_then(|element| element.value().attr("content"))
            .map(|content| content.to_string())
    }

    /// Select elements using a CSS selector
    pub fn select(&self, selector: &str) -> Result<Vec<ElementRef<'_>>> {
        let selector_obj = Selector::parse(selector)
            .map_err(|e| FerrisFetcherError::ParseError(format!("Invalid CSS selector '{}': {}", selector, e)))?;
        Ok(self.document.select(&selector_obj).collect())
    }

    /// Select the first element matching a CSS selector
    pub fn select_first(&self, selector: &str) -> Option<ElementRef<'_>> {
        if let Ok(selector_obj) = Selector::parse(selector) {
            self.document.select(&selector_obj).next()
        } else {
            None
        }
    }

    /// Extract text content from elements matching a selector
    pub fn select_text(&self, selector: &str) -> Result<Vec<String>> {
        let elements = self.select(selector)?;
        Ok(elements
            .iter()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .filter(|text| !text.is_empty())
            .collect())
    }

    /// Extract the first text content matching a selector
    pub fn select_first_text(&self, selector: &str) -> Option<String> {
        self.select_first(selector)
            .map(|element| element.text().collect::<String>().trim().to_string())
            .filter(|text| !text.is_empty())
    }

    /// Extract attribute values from elements matching a selector
    pub fn select_attr(&self, selector: &str, attr: &str) -> Result<Vec<String>> {
        let elements = self.select(selector)?;
        Ok(elements
            .iter()
            .filter_map(|element| element.value().attr(attr))
            .map(|value| value.to_string())
            .collect())
    }

    /// Extract the first attribute value matching a selector
    pub fn select_first_attr(&self, selector: &str, attr: &str) -> Option<String> {
        self.select_first(selector)
            .and_then(|element| element.value().attr(attr))
            .map(|value| value.to_string())
    }

    /// Extract HTML content from elements matching a selector
    pub fn select_html(&self, selector: &str) -> Result<Vec<String>> {
        let elements = self.select(selector)?;
        Ok(elements
            .iter()
            .map(|element| element.html())
            .collect())
    }

    /// Extract the first HTML content matching a selector
    pub fn select_first_html(&self, selector: &str) -> Option<String> {
        self.select_first(selector)
            .map(|element| element.html())
    }

    /// Extract outer HTML from elements matching a selector
    pub fn select_outer_html(&self, selector: &str) -> Result<Vec<String>> {
        let elements = self.select(selector)?;
        Ok(elements
            .iter()
            .map(|element| element.html())
            .collect())
    }

    /// Get all links (href attributes) from the page
    pub fn links(&self) -> Vec<String> {
        self.select_attr("a[href]", "href")
            .unwrap_or_default()
    }

    /// Get all images (src attributes) from the page
    pub fn images(&self) -> Vec<String> {
        self.select_attr("img[src]", "src")
            .unwrap_or_default()
    }

    /// Get all forms from the page
    pub fn forms(&self) -> Vec<FormInfo> {
        let selector = "form";
        if let Ok(elements) = self.select(selector) {
            elements
                .iter()
                .filter_map(|element| self.extract_form_info(*element))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Extract form information
    fn extract_form_info(&self, form_element: ElementRef) -> Option<FormInfo> {
        let action = form_element.value().attr("action").map(|s| s.to_string());
        let method = form_element.value().attr("method").map(|s| s.to_string()).unwrap_or_else(|| "GET".to_string());
        
        let inputs = self.select("input, textarea, select")
            .unwrap_or_default()
            .into_iter()
            .filter_map(|element| self.extract_input_info(element))
            .collect();

        Some(FormInfo {
            action,
            method,
            inputs,
        })
    }

    /// Extract input field information
    fn extract_input_info(&self, input_element: ElementRef) -> Option<InputInfo> {
        let name = input_element.value().attr("name").map(|s| s.to_string())?;
        let input_type = input_element.value().attr("type").map(|s| s.to_string()).unwrap_or_else(|| "text".to_string());
        let value = input_element.value().attr("value").map(|s| s.to_string());
        let required = input_element.value().attr("required").is_some();

        Some(InputInfo {
            name,
            input_type,
            value,
            required,
        })
    }

    /// Get text content with cleaning (removes extra whitespace)
    pub fn clean_text(&self, selector: &str) -> Result<Vec<String>> {
        let texts = self.select_text(selector)?;
        Ok(texts
            .into_iter()
            .map(|text| self.clean_whitespace(&text))
            .collect())
    }

    /// Clean whitespace from text
    fn clean_whitespace(&self, text: &str) -> String {
        text.split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Check if a selector exists in the document
    pub fn has_selector(&self, selector: &str) -> bool {
        Selector::parse(selector)
            .map(|sel| self.document.select(&sel).next().is_some())
            .unwrap_or(false)
    }

    /// Count elements matching a selector
    pub fn count(&self, selector: &str) -> usize {
        if let Ok(selector_obj) = Selector::parse(selector) {
            self.document.select(&selector_obj).count()
        } else {
            0
        }
    }

    /// Get JSON-LD structured data from the page
    pub fn json_ld(&self) -> Vec<serde_json::Value> {
        if let Ok(script_elements) = self.select("script[type='application/ld+json']") {
            script_elements
                .iter()
                .filter_map(|element| {
                    let json_text = element.text().collect::<String>();
                    serde_json::from_str(&json_text).ok()
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get page description (meta description or og:description)
    pub fn description(&self) -> Option<String> {
        self.meta_tag("description")
            .or_else(|| self.meta_property("og:description"))
    }

    /// Get page keywords
    pub fn keywords(&self) -> Option<String> {
        self.meta_tag("keywords")
    }

    /// Get canonical URL
    pub fn canonical_url(&self) -> Option<String> {
        self.select_first_attr("link[rel='canonical']", "href")
    }

    /// Get the original HTML document
    pub fn document(&self) -> &Html {
        &self.document
    }
}

/// Form information extracted from HTML
#[derive(Debug, Clone)]
pub struct FormInfo {
    pub action: Option<String>,
    pub method: String,
    pub inputs: Vec<InputInfo>,
}

/// Input field information
#[derive(Debug, Clone)]
pub struct InputInfo {
    pub name: String,
    pub input_type: String,
    pub value: Option<String>,
    pub required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_parser_creation() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test Page</title></head>
        <body><h1>Hello World</h1></body>
        </html>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        assert_eq!(parser.title(), Some("Test Page".to_string()));
    }

    #[test]
    fn test_select_text() {
        let html = r#"
        <div class="content">
            <p>First paragraph</p>
            <p>Second paragraph</p>
        </div>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let texts = parser.select_text("p").unwrap();
        assert_eq!(texts, vec!["First paragraph", "Second paragraph"]);
    }

    #[test]
    fn test_select_first_text() {
        let html = r#"
        <div class="content">
            <p>First paragraph</p>
            <p>Second paragraph</p>
        </div>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let text = parser.select_first_text("p");
        assert_eq!(text, Some("First paragraph".to_string()));
    }

    #[test]
    fn test_select_attr() {
        let html = r#"
        <div>
            <a href="https://example.com">Link 1</a>
            <a href="https://test.com">Link 2</a>
        </div>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let hrefs = parser.select_attr("a[href]", "href").unwrap();
        assert_eq!(hrefs, vec!["https://example.com", "https://test.com"]);
    }

    #[test]
    fn test_meta_tags() {
        let html = r#"
        <head>
            <meta name="description" content="Test description">
            <meta property="og:title" content="Test title">
        </head>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        assert_eq!(parser.meta_tag("description"), Some("Test description".to_string()));
        assert_eq!(parser.meta_property("og:title"), Some("Test title".to_string()));
    }

    #[test]
    fn test_links_and_images() {
        let html = r#"
        <div>
            <a href="https://example.com">Link</a>
            <img src="https://example.com/image.jpg" alt="Image">
        </div>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let links = parser.links();
        let images = parser.images();
        assert_eq!(links, vec!["https://example.com"]);
        assert_eq!(images, vec!["https://example.com/image.jpg"]);
    }

    #[test]
    fn test_forms() {
        let html = r#"
        <form action="/submit" method="POST">
            <input type="text" name="username" required>
            <input type="password" name="password" required>
            <input type="submit" value="Submit">
        </form>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let forms = parser.forms();
        assert_eq!(forms.len(), 1);
        
        let form = &forms[0];
        assert_eq!(form.action, Some("/submit".to_string()));
        assert_eq!(form.method, "POST");
        assert_eq!(form.inputs.len(), 3);
    }

    #[test]
    fn test_invalid_selector() {
        let html = "<div>Test</div>";
        let parser = HtmlParser::new(html).unwrap();
        
        let result = parser.select("invalid[selector");
        assert!(result.is_err());
    }

    #[test]
    fn test_clean_text() {
        let html = r#"
        <div>
            <p>   Text   with    extra   spaces   </p>
        </div>
        "#;
        
        let parser = HtmlParser::new(html).unwrap();
        let cleaned = parser.clean_text("p").unwrap();
        assert_eq!(cleaned, vec!["Text with extra spaces"]);
    }
}
