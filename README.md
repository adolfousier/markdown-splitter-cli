# Markdown Splitter

A powerful Rust CLI tool for splitting markdown documents into multiple files based on page breaks. Supports both local files and remote URLs with configurable splitting strategies.

## Features

- **Multiple Input Sources**: Support for local files and HTTP/HTTPS URLs
- **Flexible Page Detection**: Automatic detection of page breaks using various patterns
- **Custom Page Markers**: Define your own page break patterns
- **Batch Processing**: Process multiple markdown files simultaneously  
- **Smart Splitting**: Calculate optimal page distribution across splits
- **Metadata Generation**: Optional metadata files with split information
- **Structure Preservation**: Maintain document structure with separators
- **Analysis Mode**: Analyze documents without splitting
- **Validation**: Verify input sources before processing

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd markdown-splitter

# Build the project
cargo build --release

# The binary will be available at target/release/md-split
```

## Usage

### Basic Splitting

Split a single markdown file into 5 parts:
```bash
md-split split document.md --splits 5
```

Split multiple files:
```bash
md-split split file1.md file2.md file3.md --splits 3
```

Split from URLs:
```bash
md-split split https://raw.githubusercontent.com/user/repo/main/README.md --splits 4
```

Mix local files and URLs:
```bash
md-split split local-file.md https://example.com/remote.md --splits 2
```

### Advanced Options

Specify custom output directory:
```bash
md-split split document.md --splits 5 --output ./my-output
```

Use custom page break marker:
```bash
md-split split document.md --splits 3 --page-marker "<!-- SPLIT HERE -->"
```

Force overwrite existing files:
```bash
md-split split document.md --splits 5 --force
```

Disable structure preservation:
```bash
md-split split document.md --splits 5 --preserve-structure false
```

Skip metadata generation:
```bash
md-split split document.md --splits 5 --include-metadata false
```

### Analysis Mode

Analyze documents without splitting:
```bash
md-split analyze document.md
```

Detailed analysis with page information:
```bash
md-split analyze document.md --detailed
```

Save analysis to JSON:
```bash
md-split analyze document.md --json-output analysis.json
```

### Validation

Validate input sources:
```bash
md-split validate file1.md https://example.com/file2.md
```

Check accessibility:
```bash
md-split validate file1.md --check-access
```

## Page Break Detection

The tool automatically detects page breaks using these patterns:

1. **Horizontal Rules**: `---`, `***`, `___`
2. **HTML Comments**: `<!-- page break -->`, `<!-- pagebreak -->`
3. **LaTeX Commands**: `\pagebreak`, `\newpage`  
4. **Headers**: Any markdown header (`#`, `##`, etc.)
5. **Custom Markers**: User-defined regex patterns

### Custom Page Markers

You can define custom page break patterns using regex:

```bash
# Split on custom HTML comments
md-split split document.md --page-marker "<!-- NEW PAGE -->" --splits 3

# Split on specific markdown syntax
md-split split document.md --page-marker "^=== BREAK ===$" --splits 4
```

## Output Structure

When splitting `document.md` into 3 parts, the output structure will be:

```
output/
├── document_split_1_of_3.md
├── document_split_2_of_3.md  
├── document_split_3_of_3.md
└── document_metadata.json    (if --include-metadata)
```

### Metadata File Example

```json
{
  "source": "document.md",
  "total_pages": 15,
  "total_splits": 3,
  "split_files": ["document_split_1_of_3.md", "document_split_2_of_3.md", "document_split_3_of_3.md"],
  "document_metadata": {
    "filename": "document.md",
    "source_type": "LocalFile",
    "created_at": "2025-01-15T10:30:00Z",
    "total_lines": 500,
    "page_breaks": [0, 120, 250, 380, 500]
  },
  "split_info": [
    {
      "split_number": 1,
      "filename": "document_split_1_of_3.md", 
      "path": "./output/document_split_1_of_3.md"
    }
  ]
}
```

## Examples

### Example 1: Academic Paper

Split a large academic paper into sections:

```bash
md-split split research-paper.md --splits 4 --output ./paper-sections
```

### Example 2: Documentation Site

Process multiple documentation files:

```bash
md-split split \
  docs/intro.md \
  docs/tutorial.md \
  docs/advanced.md \
  --splits 2 \
  --preserve-structure true \
  --output ./split-docs
```

### Example 3: Remote Content

Split content directly from GitHub:

```bash
md-split split \
  https://raw.githubusercontent.com/rust-lang/book/main/src/README.md \
  --splits 3 \
  --output ./rust-book-splits
```

### Example 4: Custom Page Breaks

Use custom markers for specialized documents:

```bash
md-split split manual.md \
  --page-marker "^<!-- CHAPTER .* -->$" \
  --splits 5 \
  --detailed
```

### Example 5: Analysis First

Analyze before splitting to determine optimal split count:

```bash
# First analyze
md-split analyze large-document.md --detailed

# Then split based on analysis
md-split split large-document.md --splits 8
```

## Error Handling

The tool provides detailed error messages for common issues:

- **File not found**: Validates local file paths
- **URL access**: Checks remote URL accessibility  
- **Invalid regex**: Validates custom page markers
- **Empty documents**: Handles documents with no detectable pages
- **Output conflicts**: Prevents accidental overwrites (use `--force`)

## Logging

Enable verbose logging for debugging:

```bash
md-split split document.md --splits 5 --verbose
```

## Performance Tips

1. **Large Files**: The tool handles large files efficiently by streaming content
2. **Multiple Files**: Processes files sequentially to manage memory usage
3. **Remote URLs**: Caches remote content temporarily during processing
4. **Output Directory**: Ensure sufficient disk space for split files

## Changelog

### v0.1.1 (2025-09-18)
- **Enhanced Split Markers**: Split markers now include the document name for better context
  - Format changed from `<!-- Split containing pages 1 to 249 -->`
  - To `<!-- EN 1993-1-2-2005 Split containing pages 1 to 249 -->`
  - Automatically removes `_structured_markdown` suffix from document names
  - Provides better contextual information when working with multiple split documents

### v0.1.0 (Initial Release)
- Initial release with core splitting functionality
- Support for local files and HTTP/HTTPS URLs
- Flexible page detection with multiple patterns
- Configurable splitting strategies
- Metadata generation and structure preservation
- Analysis and validation modes

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.