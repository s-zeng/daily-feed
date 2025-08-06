# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with 
code in this repository.

## Project Overview

This is a Rust CLI application called `daily-feed` that aggregates content from multiple sources 
(RSS feeds, specialized sources like Ars Technica) and generates EPUB or Markdown files for offline reading. 
The application fetches content asynchronously, processes it through a compiler-style pipeline, 
and can generate AI-powered front pages using LLM providers.

## Style

Try to keep the style as functional as possible ("Ocaml with manual garbage 
collection", as opposed to "C++ with borrow checker"). Use features like 
Algebraic Data Types and Traits liberally, with an algebra-oriented design 
mindset

When writing new documentation files, ensure to clarify that "Documentation written 
by Claude Code" somewhere in the file.

ALL tests should be in the `tests/` directory, and should follow the snapshot 
testing instructions in the `## Testing` section.

This project is in heavy development. Whenever you make a change, make sure to 
check `CLAUDE.md` and update it if necessary to reflect any newly added/changed 
features or structures

## Error Handling & Safety Guidelines

Based on comprehensive bug audits, follow these critical safety practices:

### Never Use `unwrap()` in Production Code
- **NEVER** use `.unwrap()` on `Option` or `Result` types in production paths
- Use proper error handling with `?`, `.ok_or()`, `.map_err()`, or pattern matching
- Example: Replace `tag_name.chars().nth(1).unwrap()` with proper error handling
- Exception: Only use `unwrap()` in tests or when preceded by explicit checks that guarantee safety

### Network Operations Safety
- **ALWAYS** set timeouts for HTTP requests - use the shared `http_utils` module
- Use the provided HTTP client utilities: `create_http_client()`, `create_ai_http_client()`
- Handle network failures gracefully with proper error messages
- Implement concurrent operations where possible for performance

### CSS Selector & Regex Safety
- **NEVER** use `.unwrap()` on `Selector::parse()` - CSS selectors can be invalid
- Move regex compilation out of hot paths using `OnceLock` or similar
- Pre-compile regexes at module or function level, not in loops
- Use `lazy_static!` or `OnceLock` for expensive operations

### Error Message Quality
- Include contextual information in error messages (URLs, file paths, etc.)
- Use structured error types instead of plain strings where possible
- Provide actionable information for debugging
- Example: `"Invalid CSS selector '.message': {}"` instead of generic "Parse error"

## Architecture

The codebase follows a true compiler-style architecture with distinct phases of data transformation:

### Compiler-Style Structure

The application processes data through several transformation phases, similar to a compiler pipeline:

1. **Lexing/Input Phase**: JSON Config + Source URLs → Raw Network Data
   - Configuration is parsed and validated (`config.rs`)
   - Multiple source types are supported via `sources.rs` trait system
   - RSS feeds and specialized sources (Ars Technica) are fetched from network (`fetch.rs`)
   - Raw RSS XML and HTML content is retrieved

2. **Parsing Phase**: Raw Content → Structured AST (`parser.rs`)
   - `DocumentParser` converts RSS channels into the unified AST structure
   - HTML content is parsed into structured `ContentBlock` enum variants
   - Comments are fetched and integrated for Ars Technica articles via `ars_comments.rs`
   - Creates strongly-typed `Document` AST with feeds, articles, and metadata

3. **AI Enhancement Phase**: Document AST → Enhanced AST (`front_page.rs`, `ai_client.rs`)
   - Optional AI-powered front page generation using LLM providers (Anthropic, Ollama)
   - `FrontPageGenerator` creates structured summaries and themes
   - AI client with retry logic and error handling

4. **AST Transformation Phase**: Document AST ↔ JSON Serialization
   - AST can be exported to JSON format (`--export-ast` flag)
   - AST can be loaded from JSON (via `ast-converter` binary)
   - Enables intermediate representation persistence and debugging

5. **Code Generation Phase**: Document AST → Output Formats
   - `EpubOutputter` transforms AST into EPUB-compatible HTML (`epub_outputter.rs`)
   - `MarkdownOutputter` transforms AST into Markdown format (`markdown_outputter.rs`)
   - CSS styling and metadata are applied for EPUB
   - Table of contents is generated from content hierarchy

### Module Structure

- `src/main.rs`: CLI entry point and pipeline orchestration (compiler driver)
- `src/lib.rs`: Module exports for library usage
- `src/config.rs`: Configuration parsing and validation with multiple source support
- `src/sources.rs`: Source trait system for different content providers (RSS, Ars Technica)
- `src/fetch.rs`: Network operations and high-level pipeline functions with concurrent source fetching
- `src/parser.rs`: Content → AST transformation (main parser)
- `src/ast.rs`: Core AST data structures with algebraic types
- `src/epub_outputter.rs`: AST → EPUB code generation (backend)
- `src/markdown_outputter.rs`: AST → Markdown code generation (backend)
- `src/ars_comments.rs`: Specialized comment extraction (domain-specific parser)
- `src/front_page.rs`: AI-powered front page generation
- `src/ai_client.rs`: Generic AI client with retry logic and provider abstraction
- `src/http_utils.rs`: Shared HTTP client utilities with timeout configuration
- `src/bin/ast-converter.rs`: Standalone AST → EPUB/Markdown converter

### Data Flow Pipeline

```
JSON Config → Sources → Raw Content → Document AST → [AI Enhancement] → Output Formats
    ↓           ↓          ↓              ↓              ↓                ↓
 Config     Sources    Parsing         AST        Front Page        Code Gen
 Parser    (RSS/AT)   (parser.rs)   (ast.rs)   (front_page.rs)  (outputters)
```

### AST-Centric Design

The `ast.rs` module defines the core intermediate representation using Rust's algebraic data types:

- `Document`: Root AST node containing metadata, optional front page, and feeds
- `Feed`: Collection of articles from a single source
- `Article`: Individual content items with metadata and comments
- `ContentBlock`: Enum representing different content types (paragraphs, headings, lists, quotes, code, links, images, raw HTML)
- `TextContent`/`TextSpan`: Rich text with formatting information
- `Comment`: Comment structures with voting scores and timestamps
- `Headline`: Individual news headline structure for front page generation

This compiler-like approach with a central AST enables:
- Clean separation between parsing, transformation, and output generation
- Easy extension with new input formats or output targets
- Intermediate representation inspection and debugging
- Testable, composable pipeline stages
- AI enhancement as an optional transformation pass

## Common Commands

### Development
```bash
# Run the application
just run

# Run with custom config
just run -c path/to/config.json

# Auto-recompile and run on changes (cargo watch)
just watch

# Format code using treefmt
just fmt
```

### Build & Run
```bash
# Standard cargo commands
cargo run
cargo build --release
cargo check

# Export AST to JSON for debugging
cargo run -- --export-ast document.json

# Enable front page generation
cargo run -- --front-page

# Specify output format
cargo run -- -f markdown
cargo run -- -f epub

# Convert AST JSON to EPUB or Markdown
cargo run --bin ast-converter -- -i document.json -o output.epub
cargo run --bin ast-converter -- -i document.json -o output.md -f markdown
```

## Configuration

The application supports two configuration formats:

### New Sources Format (Recommended)
```json
{
  "sources": [
    {
      "name": "Example RSS Feed",
      "type": "rss",
      "url": "https://example.com/feed.rss",
      "description": "Example RSS feed"
    },
    {
      "name": "Ars Technica",
      "type": "ars_technica",
      "api_token": "optional-api-token"
    }
  ],
  "output": {
    "filename": "daily-feed.epub",
    "title": "Daily Feed Digest",
    "author": "RSS Aggregator",
    "format": "epub"
  },
  "front_page": {
    "enabled": true,
    "provider": {
      "type": "anthropic",
      "api_key": "your-api-key",
      "model": "claude-sonnet-4-20250514"
    }
  }
}
```

### Legacy Feeds Format (Still Supported)
```json
{
  "feeds": [
    {
      "type": "rss",
      "name": "Feed Name",
      "url": "https://example.com/feed.rss",
      "description": "Description"
    },
    {
      "type": "ars_technica",
      "api_token": "optional-token"
    }
  ],
  "output": {
    "filename": "output.epub",
    "title": "EPUB Title",
    "author": "Author Name"
  }
}
```

### Source Types

**RSS Sources**: Standard RSS/Atom feeds
- `type`: "rss"
- `url`: Feed URL
- `description`: Human-readable description

**Ars Technica Sources**: Specialized Ars Technica integration with comments
- `type`: "ars_technica"
- `api_token`: Optional API token for authenticated access

### Front Page Configuration

**AI Providers**: 
- `anthropic`: Uses Claude models via Anthropic API
- `ollama`: Uses local Ollama installation

## Development Environment

This project uses Nix for reproducible builds and development environments. The `flake.nix` provides all necessary dependencies including OpenSSL, libiconv, and pkg-config. Use `nix develop` to enter the development shell.

## Key Dependencies

- **clap**: CLI argument parsing with derive macros
- **rss**: RSS feed parsing into structured data
- **reqwest**: HTTP client for async content fetching with timeout configuration
- **tokio**: Async runtime with full feature set
- **futures**: Additional async utilities for concurrent operations
- **epub-builder**: EPUB generation and assembly
- **serde/serde_json**: JSON configuration and AST serialization
- **regex**: HTML content sanitization and text processing (with `OnceLock` optimization)
- **scraper**: HTML parsing for content extraction and comments with error-safe CSS selectors
- **chrono**: Date/time handling with serde support
- **base64**: Encoding utilities
- **html-escape**: HTML entity handling
- **async-trait**: Async trait support for source abstractions
- **insta**: Snapshot testing framework for deterministic test assertions

## HTTP Utilities

The `src/http_utils.rs` module provides centralized HTTP client management with safety features:

### Client Functions
- `create_http_client()`: Standard HTTP client with 30s timeout
- `create_ai_http_client()`: AI operations client with 2-minute timeout
- `create_http_client_with_timeout(duration)`: Custom timeout client

### Safety Features
- **Automatic timeouts**: All clients have connection (10s) and request timeouts
- **Consistent User-Agent**: Standardized "daily-feed/0.1.0" across all requests
- **Error handling**: Proper `Result` types for client creation failures
- **Reusable clients**: Shared across modules to reduce resource overhead

### Usage Patterns
```rust
use crate::http_utils::create_http_client;

async fn fetch_data(url: &str) -> Result<String, Box<dyn Error>> {
    let client = create_http_client()?;
    let response = client.get(url).send().await?;
    // Handle response...
}
```

## Testing

The project uses **snapshot testing** via the `insta` crate for all test assertions. This testing paradigm provides deterministic, maintainable tests that capture expected behavior through literal value snapshots.

### Snapshot Testing Approach

All tests follow these principles:
- **Single assertion per test**: Each test has exactly one `insta::assert_snapshot!()` or `insta::assert_json_snapshot!()` call
- **Deterministic snapshots**: Dynamic data (timestamps, file sizes, temp paths) is normalized to ensure reproducible results
- **Literal value snapshots**: Snapshots contain only concrete, expected values without variables

### Test Structure

The project has comprehensive tests across multiple categories:

- **Unit tests**: Embedded in source files (`#[cfg(test)]` modules)
  - `src/ast.rs`: AST data structure tests with JSON snapshots
  - `src/parser.rs`: HTML parsing tests with JSON snapshots
  - `src/epub_outputter.rs`: EPUB HTML generation tests with string snapshots
  - `src/markdown_outputter.rs`: Markdown generation tests with string snapshots
  - `src/ars_comments.rs`: Comment parsing tests with JSON snapshots
  - `src/ai_client.rs`: AI client functionality tests with JSON snapshots
  - `src/front_page.rs`: Front page generation tests (implied by snapshots)

- **Integration tests**: Multiple test files in `tests/`
  - `tests/integration_tests.rs`: Full workflow tests with range-based file size validation
  - `tests/fetch_tests.rs`: Network operations and RSS feed processing
  - `tests/config_tests.rs`: Configuration parsing and validation with JSON snapshots
  - `tests/ars_comments_tests.rs`: Ars Technica comment extraction with structured snapshots
  - `tests/front_page_tests.rs`: AI front page generation tests
  - `tests/ast_tests.rs`: AST manipulation tests
  - `tests/epub_outputter_tests.rs`: EPUB generation tests
  - `tests/markdown_outputter_tests.rs`: Markdown generation tests
  - `tests/parser_tests.rs`: Content parsing tests

- **Golden file tests**: `tests/golden_tests.rs`
  - End-to-end pipeline tests from RSS to EPUB/Markdown
  - AST roundtrip serialization tests
  - Tests use normalized timestamps and size ranges

- **Cram tests**: `tests/cram_tests.rs`
  - CLI behavior simulation with snapshot assertions
  - Error handling and edge case validation

- **Test fixtures**: Sample RSS feeds in `tests/fixtures/`
- **Golden outputs**: Reference outputs in `tests/golden_output/`

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test <test_name>

# Review and accept snapshot changes
cargo insta review

# Auto-accept all snapshot changes (use carefully)
cargo insta accept
```

### Snapshot Management

- Snapshots are stored in `src/snapshots/` (unit tests) and `tests/snapshots/` (integration tests)
- When test behavior changes, run `cargo insta review` to inspect differences
- Accept valid changes with `cargo insta accept` or reject with `cargo insta reject`
- Never commit `.snap.new` files - these are pending snapshot updates

### Deterministic Test Strategies

To ensure reproducible snapshots, the tests employ several normalization techniques:

- **Timestamp normalization**: Replace dynamic timestamps with fixed values
- **File size ranges**: Use `size > min && size < max` instead of exact sizes
- **Path abstraction**: Extract relevant path components, ignore temp directories
- **Content summarization**: Focus on structural properties rather than exact values

This approach makes tests resilient to environmental differences while maintaining strict behavioral validation.

## Features

### Multi-Source Content Aggregation
- RSS/Atom feed support with concurrent fetching for improved performance
- Specialized Ars Technica integration with comment extraction
- Extensible source system via trait abstractions
- HTML content sanitization for output compatibility
- Robust error handling with proper timeout configuration

### AI-Powered Front Pages
- LLM-generated front page summaries and themes
- Support for Anthropic Claude and Ollama providers
- Structured front page generation with source summaries
- Retry logic with exponential backoff for API calls

### Multiple Output Formats
- EPUB generation with CSS styling and table of contents
- Markdown output with proper formatting
- AST JSON export for debugging and intermediate processing
- Cross-format content preservation

### Ars Technica Comment Integration
- Fetches top comments from Ars Technica articles
- Parses XenForo forum structure to extract comment data
- Returns structured Comment objects with author, content, score, and timestamp
- Main functions in `ars_comments.rs`:
  - `fetch_top_comments(article_url, limit)`: Fetch top N comments
  - `fetch_top_5_comments(article_url)`: Convenience wrapper for top 5

### CLI Features
- Flexible configuration file support (default: `config.json`)
- AST export and import capabilities
- Output format selection (`--format epub|markdown`)
- Front page generation toggle (`--front-page`)
- Verbose mode for debugging (`-v`)

## Notes

- Tests include both unit tests and integration tests with real article data
- Comment fetching handles network failures gracefully with proper timeout handling
- HTML parsing uses CSS selectors for robust comment extraction with error handling
- AI features are optional and gracefully degrade when unavailable
- The compiler-style architecture makes the codebase highly modular and testable
- All network operations use shared HTTP clients with appropriate timeouts
- Error handling follows functional programming principles - no panics in production paths

## Performance Considerations

### Concurrent Operations
- Source fetching happens concurrently using `futures::future::join_all()`
- HTTP clients are reused across operations to reduce overhead
- Regex compilation is moved out of hot paths using `OnceLock`

### Memory Management
- Avoid excessive string cloning - use string slices where possible
- Pre-allocate vectors with known capacity when feasible
- Use streaming approaches for large documents when necessary

### Network Efficiency
- HTTP clients configured with appropriate timeouts:
  - Standard requests: 30 seconds
  - Connection timeout: 10 seconds  
  - AI operations: 2 minutes
- Proper User-Agent headers for all requests
- Retry logic with exponential backoff for AI operations