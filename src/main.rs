mod cli;
mod error;
mod services;
mod types;

use anyhow::Context;
use clap::Parser;
use cli::{AnalyzeArgs, Cli, Commands, SplitArgs, ValidateArgs};
use error::{MarkdownSplitterError, Result};
use services::{ContentFetcher, DocumentSplitter, MarkdownParser};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{error, info, Level};
use tracing_subscriber;
use types::SplitConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    let result = match &cli.command {
        Commands::Split(args) => handle_split_command(args, &cli.output).await,
        Commands::Analyze(args) => handle_analyze_command(args).await,
        Commands::Validate(args) => handle_validate_command(args).await,
    };

    if let Err(e) = result {
        error!("Operation failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn handle_split_command(args: &SplitArgs, output_dir: &PathBuf) -> Result<()> {
    info!("Starting split operation with {} sources", args.sources.len());

    // Validate sources first
    let validated_sources = ContentFetcher::validate_sources(&args.sources).await?;
    info!("Validated {} sources", validated_sources.len());

    // Check if output directory exists and handle force flag
    if output_dir.exists() && !args.force {
        let entries = std::fs::read_dir(output_dir)
            .map_err(|e| MarkdownSplitterError::OutputDirectory {
                reason: format!("Cannot read output directory: {}", e),
            })?;

        if entries.count() > 0 {
            return Err(MarkdownSplitterError::OutputDirectory {
                reason: "Output directory is not empty. Use --force to overwrite.".to_string(),
            });
        }
    }

    let config = SplitConfig {
        splits: args.splits,
        output_dir: output_dir.clone(),
        preserve_structure: args.preserve_structure,
        include_metadata: args.include_metadata,
        custom_page_marker: args.page_marker.clone(),
    };

    let parser = MarkdownParser::new(config.custom_page_marker.as_deref())?;

    for (idx, source) in validated_sources.iter().enumerate() {
        info!("Processing source {}/{}: {}", idx + 1, validated_sources.len(), source);

        // Fetch content
        let (content, metadata) = ContentFetcher::fetch_content(source).await?;
        
        // Parse document
        let document = parser.parse_document(&content, metadata)?;
        
        // Calculate split information
        let (pages_per_split, split_ranges) = DocumentSplitter::calculate_split_info(
            document.total_pages, 
            config.splits
        );

        info!(
            "Document '{}' has {} pages, will create {} splits with ~{} pages each",
            document.source, document.total_pages, config.splits, pages_per_split
        );

        // Print split preview
        for (split_idx, (start, end)) in split_ranges.iter().enumerate() {
            info!("  Split {}: Pages {}-{}", split_idx + 1, start, end);
        }

        // Perform the split
        let split_result = DocumentSplitter::split_document(&document, &config).await?;

        // Report results
        info!(
            "Successfully created {} split files for '{}':",
            split_result.output_files.len(), 
            document.source
        );

        for output_file in &split_result.output_files {
            info!("  - {}", output_file.display());
        }

        if let Some(metadata_file) = &split_result.metadata_file {
            info!("  - {} (metadata)", metadata_file.display());
        }
    }

    info!("Split operation completed successfully!");
    Ok(())
}

async fn handle_analyze_command(args: &AnalyzeArgs) -> Result<()> {
    info!("Starting analysis of {} sources", args.sources.len());

    let validated_sources = ContentFetcher::validate_sources(&args.sources).await?;
    let parser = MarkdownParser::new(args.page_marker.as_deref())?;
    
    let mut all_analyses = HashMap::new();

    for source in validated_sources {
        info!("Analyzing: {}", source);

        let (content, metadata) = ContentFetcher::fetch_content(&source).await?;
        let document = parser.parse_document(&content, metadata)?;
        let stats = parser.get_parsing_stats(&document);

        // Print analysis to console
        println!("\n=== Analysis for '{}' ===", document.source);
        println!("Source type: {:?}", document.metadata.source_type);
        println!("Total pages: {}", document.total_pages);
        println!("Total lines: {}", document.metadata.total_lines);
        println!("Page breaks found: {}", document.metadata.page_breaks.len());
        
        if let Some(avg_lines) = stats.get("avg_lines_per_page") {
            println!("Average lines per page: {:.1}", avg_lines.as_f64().unwrap_or(0.0));
        }
        
        if let Some(titled_pages) = stats.get("pages_with_titles") {
            println!("Pages with titles: {}", titled_pages.as_u64().unwrap_or(0));
        }

        if args.detailed {
            println!("\nPage Details:");
            for page in &document.pages {
                let title_info = page.title.as_ref()
                    .map(|t| format!(" ({})", t))
                    .unwrap_or_default();
                println!(
                    "  Page {}: Lines {}-{} ({} lines){}",
                    page.number,
                    page.start_line + 1,
                    page.end_line,
                    page.end_line - page.start_line,
                    title_info
                );
            }
        }

        // Calculate potential splits
        println!("\nPotential Split Scenarios:");
        for splits in [2, 3, 5, 10] {
            if splits <= document.total_pages {
                let (pages_per_split, ranges) = DocumentSplitter::calculate_split_info(
                    document.total_pages, 
                    splits
                );
                println!("  {} splits: ~{} pages per split", splits, pages_per_split);
                if args.detailed {
                    for (idx, (start, end)) in ranges.iter().enumerate() {
                        println!("    Split {}: Pages {}-{}", idx + 1, start, end);
                    }
                }
            }
        }

        // Store for JSON output
        all_analyses.insert(source.clone(), serde_json::json!({
            "document": document,
            "stats": stats
        }));
    }

    // Write JSON output if requested
    if let Some(json_path) = &args.json_output {
        let json_content = serde_json::to_string_pretty(&all_analyses)
            .context("Failed to serialize analysis results")?;
        
        tokio::fs::write(json_path, json_content).await
            .context("Failed to write JSON analysis file")?;
        
        info!("Analysis results written to: {}", json_path.display());
    }

    Ok(())
}

async fn handle_validate_command(args: &ValidateArgs) -> Result<()> {
    info!("Validating {} sources", args.sources.len());

    let mut valid_sources = Vec::new();
    let mut invalid_sources = Vec::new();

    for source in &args.sources {
        match ContentFetcher::validate_sources(&[source.clone()]).await {
            Ok(_) => {
                info!("✓ Valid: {}", source);
                valid_sources.push(source);
                
                if args.check_access {
                    // Try to actually fetch a small portion of the content
                    match ContentFetcher::fetch_content(source).await {
                        Ok((content, _)) => {
                            let lines = content.lines().count();
                            info!("  Accessible, {} lines found", lines);
                        }
                        Err(e) => {
                            error!("  Cannot access content: {}", e);
                            invalid_sources.push((source, format!("Access error: {}", e)));
                        }
                    }
                }
            }
            Err(e) => {
                error!("✗ Invalid: {} - {}", source, e);
                invalid_sources.push((source, e.to_string()));
            }
        }
    }

    println!("\n=== Validation Summary ===");
    println!("Valid sources: {}/{}", valid_sources.len(), args.sources.len());
    
    if !invalid_sources.is_empty() {
        println!("Invalid sources:");
        let invalid_count = invalid_sources.len();
        for (source, error) in invalid_sources {
            println!("  - {}: {}", source, error);
        }
        return Err(MarkdownSplitterError::InvalidMarkdown {
            reason: format!("{} sources failed validation", invalid_count),
        });
    }

    println!("All sources are valid!");
    Ok(())
}