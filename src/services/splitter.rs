use crate::error::{MarkdownSplitterError, Result};
use crate::types::{MarkdownDocument, MarkdownPage, SplitConfig, SplitResult};
use serde_json;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info};

pub struct DocumentSplitter;

impl DocumentSplitter {
    pub async fn split_document(
        document: &MarkdownDocument,
        config: &SplitConfig,
    ) -> Result<SplitResult> {
        info!(
            "Splitting document '{}' into {} splits",
            document.source, config.splits
        );

        // Validate split configuration
        Self::validate_split_config(document, config)?;

        // Ensure output directory exists
        Self::ensure_output_directory(&config.output_dir).await?;

        let pages_per_split = (document.total_pages + config.splits - 1) / config.splits; // Ceiling division
        let mut output_files = Vec::new();
        let mut actual_pages = 0;

        // Split the document
        for split_idx in 0..config.splits {
            let start_page = split_idx * pages_per_split;
            let end_page = std::cmp::min(start_page + pages_per_split, document.total_pages);

            if start_page >= document.total_pages {
                break; // No more pages to split
            }

            let split_pages = &document.pages[start_page..end_page];
            actual_pages += split_pages.len();

            let output_file = Self::generate_output_filename(
                &config.output_dir,
                &document.source,
                split_idx + 1,
                config.splits,
            );

            Self::write_split_file(&output_file, split_pages, config).await?;
            output_files.push(output_file);

            debug!(
                "Created split {} with {} pages (pages {}-{})",
                split_idx + 1,
                split_pages.len(),
                start_page + 1,
                end_page
            );
        }

        // Generate metadata file if requested
        let metadata_file = if config.include_metadata {
            let metadata_path = Self::generate_metadata_filename(&config.output_dir, &document.source);
            Self::write_metadata_file(&metadata_path, document, &output_files).await?;
            Some(metadata_path)
        } else {
            None
        };

        let result = SplitResult {
            split_number: output_files.len(),
            pages_per_split,
            actual_pages,
            output_files,
            metadata_file,
        };

        info!(
            "Successfully split document into {} files with {} total pages",
            result.split_number, result.actual_pages
        );

        Ok(result)
    }

    fn validate_split_config(document: &MarkdownDocument, config: &SplitConfig) -> Result<()> {
        if config.splits == 0 {
            return Err(MarkdownSplitterError::SplitConfig {
                reason: "Number of splits must be greater than 0".to_string(),
            });
        }

        if document.total_pages == 0 {
            return Err(MarkdownSplitterError::SplitConfig {
                reason: "Document has no pages to split".to_string(),
            });
        }

        if config.splits > document.total_pages {
            return Err(MarkdownSplitterError::SplitConfig {
                reason: format!(
                    "Number of splits ({}) cannot exceed total pages ({})",
                    config.splits, document.total_pages
                ),
            });
        }

        Ok(())
    }

    async fn ensure_output_directory(output_dir: &PathBuf) -> Result<()> {
        if !output_dir.exists() {
            fs::create_dir_all(output_dir).await.map_err(|e| {
                MarkdownSplitterError::OutputDirectory {
                    reason: format!("Failed to create output directory: {}", e),
                }
            })?;
            info!("Created output directory: {}", output_dir.display());
        }
        Ok(())
    }

    fn generate_output_filename(
        output_dir: &PathBuf,
        source_name: &str,
        split_number: usize,
        total_splits: usize,
    ) -> PathBuf {
        let base_name = std::path::Path::new(source_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");

        let filename = format!(
            "{}_split_{:0width$}_of_{}.md",
            base_name,
            split_number,
            total_splits,
            width = total_splits.to_string().len()
        );

        output_dir.join(filename)
    }

    fn generate_metadata_filename(output_dir: &PathBuf, source_name: &str) -> PathBuf {
        let base_name = std::path::Path::new(source_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");

        let filename = format!("{}_metadata.json", base_name);
        output_dir.join(filename)
    }

    async fn write_split_file(
        output_path: &PathBuf,
        pages: &[MarkdownPage],
        config: &SplitConfig,
    ) -> Result<()> {
        let mut content = String::new();

        // Add header if preserving structure
        if config.preserve_structure {
            content.push_str(&format!(
                "<!-- Split containing pages {} to {} -->\n\n",
                pages.first().map(|p| p.number).unwrap_or(1),
                pages.last().map(|p| p.number).unwrap_or(1)
            ));
        }

        // Combine page contents
        for (idx, page) in pages.iter().enumerate() {
            if idx > 0 && config.preserve_structure {
                content.push_str("\n\n---\n\n"); // Page separator
            }
            content.push_str(&page.content);
        }

        fs::write(output_path, content).await.map_err(|e| {
            MarkdownSplitterError::OutputDirectory {
                reason: format!("Failed to write split file {}: {}", output_path.display(), e),
            }
        })?;

        Ok(())
    }

    async fn write_metadata_file(
        metadata_path: &PathBuf,
        document: &MarkdownDocument,
        output_files: &[PathBuf],
    ) -> Result<()> {
        let metadata = serde_json::json!({
            "source": document.source,
            "total_pages": document.total_pages,
            "total_splits": output_files.len(),
            "split_files": output_files.iter().map(|p| p.file_name().unwrap().to_str().unwrap()).collect::<Vec<_>>(),
            "document_metadata": document.metadata,
            "split_info": output_files.iter().enumerate().map(|(idx, path)| {
                serde_json::json!({
                    "split_number": idx + 1,
                    "filename": path.file_name().unwrap().to_str().unwrap(),
                    "path": path.to_str().unwrap()
                })
            }).collect::<Vec<_>>()
        });

        let json_content = serde_json::to_string_pretty(&metadata).map_err(|e| {
            MarkdownSplitterError::OutputDirectory {
                reason: format!("Failed to serialize metadata: {}", e),
            }
        })?;

        fs::write(metadata_path, json_content).await.map_err(|e| {
            MarkdownSplitterError::OutputDirectory {
                reason: format!("Failed to write metadata file: {}", e),
            }
        })?;

        info!("Generated metadata file: {}", metadata_path.display());
        Ok(())
    }

    pub fn calculate_split_info(total_pages: usize, splits: usize) -> (usize, Vec<(usize, usize)>) {
        let pages_per_split = (total_pages + splits - 1) / splits;
        let mut split_ranges = Vec::new();

        for split_idx in 0..splits {
            let start_page = split_idx * pages_per_split;
            let end_page = std::cmp::min(start_page + pages_per_split, total_pages);

            if start_page >= total_pages {
                break;
            }

            split_ranges.push((start_page + 1, end_page)); // 1-based indexing for display
        }

        (pages_per_split, split_ranges)
    }
}