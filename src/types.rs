use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownPage {
    pub number: usize,
    pub content: String,
    pub title: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownDocument {
    pub source: String,
    pub total_pages: usize,
    pub pages: Vec<MarkdownPage>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub filename: String,
    pub source_type: SourceType,
    pub created_at: String,
    pub total_lines: usize,
    pub page_breaks: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    LocalFile,
    Url,
}

#[derive(Debug, Clone)]
pub struct SplitConfig {
    pub splits: usize,
    pub output_dir: PathBuf,
    pub preserve_structure: bool,
    pub include_metadata: bool,
    pub custom_page_marker: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SplitResult {
    pub split_number: usize,
    pub pages_per_split: usize,
    pub actual_pages: usize,
    pub output_files: Vec<PathBuf>,
    pub metadata_file: Option<PathBuf>,
}