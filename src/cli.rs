use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "md-split")]
#[command(about = "A CLI tool for splitting markdown documents into multiple files based on pages")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output directory for split files
    #[arg(short, long, global = true, default_value = "./output")]
    pub output: PathBuf,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Split markdown files into multiple parts
    Split(SplitArgs),
    
    /// Analyze markdown files without splitting
    Analyze(AnalyzeArgs),
    
    /// Validate input sources
    Validate(ValidateArgs),
}

#[derive(Args)]
pub struct SplitArgs {
    /// Input sources (file paths or URLs)
    #[arg(required = true, value_name = "SOURCE")]
    pub sources: Vec<String>,

    /// Number of splits to create
    #[arg(short, long, default_value = "5")]
    pub splits: usize,

    /// Preserve document structure with separators
    #[arg(long, default_value = "true")]
    pub preserve_structure: bool,

    /// Include metadata file
    #[arg(long, default_value = "true")]
    pub include_metadata: bool,

    /// Custom page break marker (regex pattern)
    #[arg(long, value_name = "PATTERN")]
    pub page_marker: Option<String>,

    /// Force overwrite existing output files
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct AnalyzeArgs {
    /// Input sources (file paths or URLs)
    #[arg(required = true, value_name = "SOURCE")]
    pub sources: Vec<String>,

    /// Custom page break marker (regex pattern)
    #[arg(long, value_name = "PATTERN")]
    pub page_marker: Option<String>,

    /// Output analysis to JSON file
    #[arg(long, value_name = "FILE")]
    pub json_output: Option<PathBuf>,

    /// Show detailed page information
    #[arg(long)]
    pub detailed: bool,
}

#[derive(Args)]
pub struct ValidateArgs {
    /// Input sources (file paths or URLs)
    #[arg(required = true, value_name = "SOURCE")]
    pub sources: Vec<String>,

    /// Check if sources are accessible
    #[arg(long)]
    pub check_access: bool,
}