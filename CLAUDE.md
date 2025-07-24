# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with 
code in this repository.

## Project Overview

This is a Rust CLI application called `daily-feed` that aggregates RSS feeds and 
generates EPUB files for offline reading. The application fetches RSS feeds 
asynchronously, processes the content, and outputs a structured EPUB with 
styling and table of contents.

## Style

Try to keep the style as functional as possible ("Ocaml with manual garbage 
collection", as opposed to "C++ with borrow checker"). Use features like 
Algebraic Data Types and Traits liberally, with an algebra-oriented design 
mindset

When writing new documentation files, ensure to clarify that "Documentation written 
by Claude Code" somewhere in the file.

This project is in heavy development. Whenever you make a change, make sure to 
check `CLAUDE.md` and update it if necessary to reflect any newly added/changed 
features or structures

## Architecture

The codebase follows a true compiler-style architecture with distinct phases of data transformation:

### Compiler-Style Structure

The application processes data through several transformation phases, similar to a compiler pipeline:

1. **Lexing/Input Phase**: JSON Config + RSS URLs → Raw Network Data
   - Configuration is parsed and validated (`config.rs`)
   - RSS feeds are fetched from network (`fetch.rs::feed_from_url`, `fetch::fetch_all_feeds`)
   - Raw RSS XML and HTML content is retrieved

2. **Parsing Phase**: Raw RSS/HTML → Structured AST (`parser.rs`)
   - `DocumentParser` converts RSS channels into the unified AST structure
   - HTML content is parsed into structured `ContentBlock` enum variants
   - Comments are fetched and integrated for Ars Technica articles
   - Creates strongly-typed `Document` AST with feeds, articles, and metadata

3. **AST Transformation Phase**: Document AST ↔ JSON Serialization
   - AST can be exported to JSON format (`--export-ast` flag)
   - AST can be loaded from JSON (via `ast-to-epub` binary)
   - Enables intermediate representation persistence and debugging

4. **Code Generation Phase**: Document AST → EPUB Output (`epub_outputter.rs`)
   - `EpubOutputter` transforms AST into EPUB-compatible HTML
   - CSS styling and metadata are applied
   - Table of contents is generated from content hierarchy
   - Final EPUB file is assembled using `epub-builder`

### Module Structure

- `src/main.rs`: CLI entry point and pipeline orchestration (compiler driver)
- `src/lib.rs`: Module exports for library usage
- `src/config.rs`: Configuration parsing and validation (config lexer)
- `src/fetch.rs`: Network operations and high-level pipeline functions
- `src/parser.rs`: RSS/HTML → AST transformation (main parser)
- `src/ast.rs`: Core AST data structures with algebraic types
- `src/epub_outputter.rs`: AST → EPUB code generation (backend)
- `src/ars_comments.rs`: Specialized comment extraction (domain-specific parser)
- `src/bin/ast_to_epub.rs`: Standalone AST → EPUB converter

### Data Flow Pipeline

```
JSON Config → RSS Feeds → Raw HTML/XML → Document AST → EPUB/JSON Output
    ↓             ↓            ↓              ↓             ↓
 Config       Network      Parsing        AST         Code Gen
 Parser       Fetch      (parser.rs)   (ast.rs)   (epub_outputter.rs)
```

### AST-Centric Design

The `ast.rs` module defines the core intermediate representation using Rust's algebraic data types:

- `Document`: Root AST node containing metadata and feeds
- `Feed`: Collection of articles from a single RSS source
- `Article`: Individual content items with metadata and comments
- `ContentBlock`: Enum representing different content types (paragraphs, headings, lists, etc.)
- `TextContent`/`TextSpan`: Rich text with formatting information

This compiler-like approach with a central AST enables:
- Clean separation between parsing, transformation, and output generation
- Easy extension with new input formats or output targets
- Intermediate representation inspection and debugging
- Testable, composable pipeline stages

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

# Convert AST JSON back to EPUB
cargo run --bin ast-to-epub -- -i document.json -o output.epub
```

## Configuration

The application uses a JSON configuration file (`config.json` by default) with this structure:
```json
{
  "feeds": [
    {
      "name": "Feed Name",
      "url": "https://example.com/feed.rss",
      "description": "Description"
    }
  ],
  "output": {
    "filename": "output.epub",
    "title": "EPUB Title",
    "author": "Author Name"
  }
}
```

## Development Environment

This project uses Nix for reproducible builds and development environments. The `flake.nix` provides all necessary dependencies including OpenSSL, libiconv, and pkg-config. Use `nix develop` to enter the development shell.

## Key Dependencies

- **clap**: CLI argument parsing with derive macros
- **rss**: RSS feed parsing into structured data
- **reqwest**: HTTP client for async RSS feed fetching
- **tokio**: Async runtime with full feature set
- **futures**: Additional async utilities
- **epub-builder**: EPUB generation and assembly
- **serde/serde_json**: JSON configuration and AST serialization
- **regex**: HTML content sanitization and text processing
- **scraper**: HTML parsing for content extraction and comments
- **chrono**: Date/time handling with serde support
- **base64**: Encoding utilities
- **html-escape**: HTML entity handling

## Testing

The project has comprehensive tests in the `tests/` directory:

- **Unit tests**: Embedded in source files (`#[cfg(test)]` modules)
- **Integration tests**: `tests/integration_tests.rs`, `tests/fetch_tests.rs`  
- **Golden file tests**: `tests/golden_tests.rs` with expected outputs in `tests/golden_output/`
- **Module-specific tests**: `tests/config_tests.rs`, `tests/ars_comments_tests.rs`
- **Cram tests**: `tests/cram_tests.rs` for CLI behavior
- **Test fixtures**: Sample RSS feeds in `tests/fixtures/`

Run with `cargo test` or use `cargo test --test <test_name>` for specific test files.

## Features

### RSS Feed Aggregation
- Concurrent RSS feed fetching for performance
- HTML content sanitization for EPUB compatibility  
- Proper User-Agent headers for RSS requests

### Ars Technica Comment Integration
- Fetches top comments from Ars Technica articles
- Parses XenForo forum structure to extract comment data
- Returns structured Comment objects with author, content, score, and timestamp
- Main functions:
  - `fetch_top_comments(article_url, limit)`: Fetch top N comments
  - `fetch_top_5_comments(article_url)`: Convenience wrapper for top 5

## Notes

- Tests include both unit tests and integration tests with real article data
- Comment fetching handles network failures gracefully
- HTML parsing uses CSS selectors for robust comment extraction
