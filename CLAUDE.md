# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with 
code in this repository.

## Project Overview

This is a Rust CLI application called `daily-feed` that aggregates RSS feeds and 
generates EPUB files for offline reading. The application fetches RSS feeds 
asynchronously, processes the content, and outputs a structured EPUB with 
styling and table of contents.

## Architecture

The codebase follows a modular structure:
- `src/main.rs`: CLI entry point with argument parsing using `clap`
- `src/config.rs`: Configuration management for feeds and output settings
- `src/fetch.rs`: RSS fetching and EPUB generation logic

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

# Build with Nix
nix build

# Enter development shell
nix develop
```

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

## Testing

No formal testing infrastructure is currently in place. The application handles errors gracefully and provides verbose output with the `-v` flag.

## Notes

- HTML content is sanitized for EPUB compatibility
- Concurrent RSS fetching for better performance
- Proper User-Agent headers for RSS requests
