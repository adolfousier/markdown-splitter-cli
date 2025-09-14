use crate::error::{MarkdownSplitterError, Result};
use crate::types::{DocumentMetadata, SourceType};
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};
use url::Url;

pub struct ContentFetcher;

impl ContentFetcher {
    pub async fn fetch_content(source: &str) -> Result<(String, DocumentMetadata)> {
        if Self::is_url(source) {
            Self::fetch_from_url(source).await
        } else {
            Self::fetch_from_file(source).await
        }
    }

    pub async fn fetch_multiple(sources: &[String]) -> Result<Vec<(String, DocumentMetadata)>> {
        let mut results = Vec::new();
        
        for source in sources {
            match Self::fetch_content(source).await {
                Ok(content) => {
                    info!("Successfully fetched content from: {}", source);
                    results.push(content);
                }
                Err(e) => {
                    warn!("Failed to fetch content from {}: {}", source, e);
                    return Err(e);
                }
            }
        }
        
        Ok(results)
    }

    async fn fetch_from_url(url: &str) -> Result<(String, DocumentMetadata)> {
        info!("Fetching content from URL: {}", url);
        
        let parsed_url = Url::parse(url)?;
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(MarkdownSplitterError::HttpStatus {
                status: response.status().as_u16(),
            });
        }
        
        let content = response.text().await?;
        let filename = Self::extract_filename_from_url(&parsed_url);
        let total_lines = content.lines().count();
        
        let metadata = DocumentMetadata {
            filename,
            source_type: SourceType::Url,
            created_at: chrono::Utc::now().to_rfc3339(),
            total_lines,
            page_breaks: Vec::new(), // Will be populated by parser
        };
        
        Ok((content, metadata))
    }

    async fn fetch_from_file(file_path: &str) -> Result<(String, DocumentMetadata)> {
        info!("Reading file: {}", file_path);
        
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(MarkdownSplitterError::FileNotFound {
                path: file_path.to_string(),
            });
        }
        
        let content = fs::read_to_string(path).await?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let total_lines = content.lines().count();
        
        let metadata = DocumentMetadata {
            filename,
            source_type: SourceType::LocalFile,
            created_at: chrono::Utc::now().to_rfc3339(),
            total_lines,
            page_breaks: Vec::new(), // Will be populated by parser
        };
        
        Ok((content, metadata))
    }

    fn is_url(source: &str) -> bool {
        source.starts_with("http://") || source.starts_with("https://")
    }

    fn extract_filename_from_url(url: &Url) -> String {
        url.path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("downloaded.md")
            .to_string()
    }

    pub async fn validate_sources(sources: &[String]) -> Result<Vec<String>> {
        let mut validated = Vec::new();
        
        for source in sources {
            if Self::is_url(source) {
                // Validate URL format
                Url::parse(source)?;
                validated.push(source.clone());
            } else {
                // Check if file exists
                let path = Path::new(source);
                if path.exists() && path.is_file() {
                    validated.push(source.clone());
                } else {
                    return Err(MarkdownSplitterError::FileNotFound {
                        path: source.clone(),
                    });
                }
            }
        }
        
        Ok(validated)
    }
}

// Add chrono dependency to Cargo.toml
// chrono = { version = "0.4", features = ["serde"] }