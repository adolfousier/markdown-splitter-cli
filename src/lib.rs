//! # Markdown Splitter Library
//! 
//! A library for splitting markdown documents into multiple files based on page breaks.
//! Supports local files, remote URLs, and configurable splitting strategies.
//! 
//! ## Example Usage
//! 
//! ```rust
//! use markdown_splitter::{ContentFetcher, MarkdownParser, DocumentSplitter, SplitConfig};
//! use std::path::PathBuf;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Fetch content
//!     let (content, metadata) = ContentFetcher::fetch_content("document.md").await?;
//!     
//!     // Parse document  
//!     let parser = MarkdownParser::new(None)?;
//!     let document = parser.parse_document(&content, metadata)?;
//!     
//!     // Configure splitting
//!     let config = SplitConfig {
//!         splits: 3,
//!         output_dir: PathBuf::from("./output"),
//!         preserve_structure: true,
//!         include_metadata: true,
//!         custom_page_marker: None,
//!     };
//!     
//!     // Split document
//!     let result = DocumentSplitter::split_document(&document, &config).await?;
//!     
//!     println!("Created {} split files", result.output_files.len());
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod services;
pub mod types;

// Re-export main types and services for easier usage
pub use error::{MarkdownSplitterError, Result};
pub use services::{ContentFetcher, DocumentSplitter, MarkdownParser};
pub use types::{
    DocumentMetadata, MarkdownDocument, MarkdownPage, SourceType, 
    SplitConfig, SplitResult
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library initialization - sets up default configurations
pub fn init() {
    // Initialize any global state if needed
    // Currently no global state required
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio;

    #[tokio::test]
    async fn test_basic_workflow() {
        // Create a sample markdown content
        let content = r#"# Title

Page 1 content here.

---

## Chapter 1

Page 2 content here.

---

## Chapter 2

Page 3 content here."#;

        // Create mock metadata
        let metadata = DocumentMetadata {
            filename: "test.md".to_string(),
            source_type: SourceType::LocalFile,
            created_at: chrono::Utc::now().to_rfc3339(),
            total_lines: content.lines().count(),
            page_breaks: Vec::new(),
        };

        // Parse document
        let parser = MarkdownParser::new(None).unwrap();
        let document = parser.parse_document(content, metadata).unwrap();

        // Verify parsing
        assert!(document.total_pages >= 3);
        assert!(!document.pages.is_empty());
        
        // Test split calculation
        let (pages_per_split, ranges) = DocumentSplitter::calculate_split_info(
            document.total_pages, 
            2
        );
        
        assert!(pages_per_split > 0);
        assert!(!ranges.is_empty());
    }

    #[test]
    fn test_parser_creation() {
        let parser = MarkdownParser::new(None);
        assert!(parser.is_ok());

        let parser_with_marker = MarkdownParser::new(Some("<!-- SPLIT -->"));
        assert!(parser_with_marker.is_ok());
    }

    #[test]
    fn test_split_config_creation() {
        let config = SplitConfig {
            splits: 5,
            output_dir: PathBuf::from("./test-output"),
            preserve_structure: true,
            include_metadata: true,
            custom_page_marker: Some("<!-- PAGE -->".to_string()),
        };

        assert_eq!(config.splits, 5);
        assert!(config.preserve_structure);
        assert!(config.include_metadata);
    }
}