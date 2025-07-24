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

## Architecture

The codebase follows a modular structure:
- `src/main.rs`: CLI entry point with argument parsing using `clap`
- `src/config.rs`: Configuration management for feeds and output settings
- `src/fetch.rs`: RSS fetching and EPUB generation logic
- `src/ars_comments.rs`: Ars Technica comment fetching functionality

Key data flow: Load JSON config → Fetch RSS feeds concurrently → Parse content → Generate styled EPUB

## Common Commands

### Development
```bash
# Run the application
just run

# Run with custom config
just run -c path/to/config.json

# Auto-recompile and run on changes
just watch

# Format code
just fmt


### Build & Run
```bash
# Standard cargo commands
cargo run
cargo build --release
cargo check
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
- **rss**: RSS feed parsing
- **reqwest**: HTTP client for async feed fetching
- **tokio**: Async runtime
- **epub-builder**: EPUB generation
- **serde/serde_json**: JSON configuration handling
- **regex**: HTML content sanitization
- **scraper**: HTML parsing for comment extraction

## Testing

Tests are written in the `tests/` directory. Run with `cargo test`

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
