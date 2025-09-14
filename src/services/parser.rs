use crate::error::{MarkdownSplitterError, Result};
use crate::types::{DocumentMetadata, MarkdownDocument, MarkdownPage};
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, info};

pub struct MarkdownParser {
    page_break_patterns: Vec<Regex>,
    title_pattern: Regex,
}

impl MarkdownParser {
    pub fn new(custom_page_marker: Option<&str>) -> Result<Self> {
        let mut patterns = vec![
            // Document-specific page marker format has highest priority
            Regex::new(r"(?m)^---\s*\n#\s+Page\s+\d+").unwrap(), // "---\n# Page 68" format
            // Alternative single-line page markers
            Regex::new(r"(?im)^\s*#\s+page\s+\d+\s*$").unwrap(), // "# Page 123"
            Regex::new(r"(?im)^\s*\(?page\s+\d+\)?\s*$").unwrap(), // "Page 123" or "(Page 123)"
            // Common page break patterns
            Regex::new(r"(?m)^---+\s*$").unwrap(), // Horizontal rules
            Regex::new(r"(?m)^<!--\s*page\s*break?\s*-->").unwrap(), // HTML comments
            Regex::new(r"(?m)^\s*\\pagebreak\s*$").unwrap(), // LaTeX style
            Regex::new(r"(?m)^\s*\\newpage\s*$").unwrap(), // LaTeX newpage
            // Only major headers as page breaks (H1 and H2), not all headers
            Regex::new(r"(?m)^#{1,2}\s+.*$").unwrap(),
        ];

        // Add custom page marker if provided
        if let Some(marker) = custom_page_marker {
            let custom_pattern = Regex::new(&format!(r"(?m)^{}\s*$", regex::escape(marker)))
                .map_err(|e| MarkdownSplitterError::PageParsing {
                    reason: format!("Invalid custom page marker regex: {}", e),
                })?;
            patterns.insert(0, custom_pattern); // Give priority to custom marker
        }

        let title_pattern = Regex::new(r"(?m)^(#{1,6})\s+(.+)$").unwrap();

        Ok(Self {
            page_break_patterns: patterns,
            title_pattern,
        })
    }

    pub fn parse_document(
        &self,
        content: &str,
        mut metadata: DocumentMetadata,
    ) -> Result<MarkdownDocument> {
        info!("Parsing markdown document: {}", metadata.filename);

        let lines: Vec<&str> = content.lines().collect();
        let page_breaks = self.find_page_breaks(&lines);
        
        metadata.page_breaks = page_breaks.clone();

        let pages = self.extract_pages(&lines, &page_breaks)?;
        let total_pages = pages.len();

        debug!("Found {} pages in document", total_pages);

        Ok(MarkdownDocument {
            source: metadata.filename.clone(),
            total_pages,
            pages,
            metadata,
        })
    }

    fn find_page_breaks(&self, lines: &[&str]) -> Vec<usize> {
        let mut breaks = vec![0]; // Always start with line 0

        // First, try to find explicit page markers (highest priority)
        let page_marker_patterns = [
            &self.page_break_patterns[0], // "---\n# Page 68" format
            &self.page_break_patterns[1], // "# Page 123"
            &self.page_break_patterns[2], // "Page 123" or "(Page 123)"
        ];
        
        let mut found_page_markers = false;
        
        for (line_idx, line) in lines.iter().enumerate() {
            for pattern in &page_marker_patterns {
                if pattern.is_match(line) {
                    found_page_markers = true;
                    // Avoid duplicate consecutive breaks
                    if breaks.last() != Some(&line_idx) {
                        breaks.push(line_idx);
                    }
                    break;
                }
            }
        }
        
        // If no explicit page markers found, fall back to other patterns
        if !found_page_markers {
            for (line_idx, line) in lines.iter().enumerate() {
                for pattern in &self.page_break_patterns[3..] { // Skip the page marker patterns
                    if pattern.is_match(line) {
                        // Avoid duplicate consecutive breaks
                        if breaks.last() != Some(&line_idx) {
                            breaks.push(line_idx);
                        }
                        break; // Found a match, no need to check other patterns
                    }
                }
            }
        }

        // Ensure we end with the last line
        if breaks.last() != Some(&lines.len()) {
            breaks.push(lines.len());
        }

        breaks
    }

    fn extract_pages(&self, lines: &[&str], page_breaks: &[usize]) -> Result<Vec<MarkdownPage>> {
        let mut pages = Vec::new();

        for (page_idx, window) in page_breaks.windows(2).enumerate() {
            let start_line = window[0];
            let end_line = window[1];

            if start_line >= lines.len() {
                break;
            }

            let actual_end = std::cmp::min(end_line, lines.len());
            let page_lines: Vec<&str> = lines[start_line..actual_end].to_vec();
            
            if page_lines.is_empty() {
                continue;
            }

            let content = page_lines.join("\n");
            let title = self.extract_title(&page_lines);
            let line_count = actual_end - start_line;

            let page = MarkdownPage {
                number: page_idx + 1,
                content,
                title,
                start_line,
                end_line: actual_end,
            };

            pages.push(page);
        }

        if pages.is_empty() {
            return Err(MarkdownSplitterError::PageParsing {
                reason: "No valid pages found in document".to_string(),
            });
        }

        // Merge small pages (likely gaps between real pages) into the previous page
        let mut merged_pages: Vec<MarkdownPage> = Vec::new();
        
        for page in pages {
            let line_count = page.end_line - page.start_line;
            
            // If this is a small page (≤10 lines) and has no page marker title, merge it with previous
            if line_count <= 10 && !self.has_page_marker_title(&page.title) && !merged_pages.is_empty() {
                // Merge with the previous page
                let prev_idx = merged_pages.len() - 1;
                
                // Append content with a separator
                merged_pages[prev_idx].content.push_str("\n\n");
                merged_pages[prev_idx].content.push_str(&page.content);
                merged_pages[prev_idx].end_line = page.end_line;
            } else {
                merged_pages.push(page);
            }
        }
        
        // Renumber pages after merging
        for (idx, page) in merged_pages.iter_mut().enumerate() {
            page.number = idx + 1;
        }

        Ok(merged_pages)
    }

    fn extract_title(&self, lines: &[&str]) -> Option<String> {
        for line in lines.iter().take(10) { // Check first 10 lines for title
            if let Some(captures) = self.title_pattern.captures(line) {
                if let Some(title) = captures.get(2) {
                    return Some(title.as_str().trim().to_string());
                }
            }
        }
        None
    }

    fn has_page_marker_title(&self, title: &Option<String>) -> bool {
        if let Some(title_text) = title {
            // Check if the title looks like a page marker
            title_text.to_lowercase().contains("page ") && 
            title_text.chars().any(|c| c.is_ascii_digit())
        } else {
            false
        }
    }

    pub fn get_parsing_stats(&self, document: &MarkdownDocument) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("total_pages".to_string(), document.total_pages.into());
        stats.insert("total_lines".to_string(), document.metadata.total_lines.into());
        stats.insert("page_breaks".to_string(), document.metadata.page_breaks.len().into());
        
        let pages_with_titles = document.pages.iter().filter(|p| p.title.is_some()).count();
        stats.insert("pages_with_titles".to_string(), pages_with_titles.into());
        
        let avg_lines_per_page = if document.total_pages > 0 {
            document.metadata.total_lines as f64 / document.total_pages as f64
        } else {
            0.0
        };
        stats.insert("avg_lines_per_page".to_string(), avg_lines_per_page.into());
        
        stats
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new(None).unwrap()
    }
}