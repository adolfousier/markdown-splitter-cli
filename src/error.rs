use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarkdownSplitterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Invalid markdown content: {reason}")]
    InvalidMarkdown { reason: String },
    
    #[error("Split configuration error: {reason}")]
    SplitConfig { reason: String },
    
    #[error("Output directory error: {reason}")]
    OutputDirectory { reason: String },
    
    #[error("Page parsing error: {reason}")]
    PageParsing { reason: String },
    
    #[error("HTTP status error: {status}")]
    HttpStatus { status: u16 },
    
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, MarkdownSplitterError>;