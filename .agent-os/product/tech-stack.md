# Technical Stack

## Language & Runtime

- **Language:** Rust (Edition 2021)
- **Async Runtime:** Tokio 1.32.0 (full features)
- **Version:** 0.1.0

## Core Dependencies

### CLI & Application Framework
- **CLI Framework:** Clap 4.3.14 (with derive macros)
- **Async Utilities:** futures 0.3.28, async-trait 0.1.88

### Network & Content Fetching
- **HTTP Client:** Reqwest 0.11.20 (with JSON and stream support)
- **RSS Parser:** rss 2.0.6
- **HTML Parser:** scraper 0.17 (CSS selector-based extraction)

### Output Generation
- **EPUB Builder:** epub-builder 0.7.4
- **Markdown:** Custom implementation via `markdown_outputter.rs`

### Data Processing
- **Serialization:** serde 1.0, serde_json 1.0 (with derive support)
- **Date/Time:** chrono 0.4 (with serde features)
- **Text Processing:** regex 1.10.2, html-escape 0.2
- **Encoding:** base64 0.21

## Development & Testing

### Build System
- **Build Tool:** Cargo
- **Package Manager:** Nix flakes for reproducible builds
- **Development Environment:** Nix development shell with OpenSSL, libiconv, pkg-config

### Testing Framework
- **Test Framework:** Rust built-in test framework
- **Snapshot Testing:** insta 1.34 (with JSON snapshot support)
- **Async Testing:** tokio-test 0.4
- **Fixtures:** tempfile 3.8 for temporary file creation

### Code Quality
- **Formatter:** treefmt (via `just fmt`)
- **Watch Mode:** cargo-watch (via `just watch`)
- **Testing Strategy:** Comprehensive snapshot testing with offline resilience

## Architecture Patterns

### Design Philosophy
- **Paradigm:** Functional programming ("OCaml with manual garbage collection")
- **Type System:** Algebraic data types, heavy use of traits
- **Error Handling:** Result/Option types without `unwrap()` in production code
- **Data Flow:** Compiler-style pipeline with Abstract Syntax Tree (AST)

### Module Organization
- **Source Abstraction:** Trait-based source system (`sources.rs`)
- **Content Pipeline:** Lexing → Parsing → AST → Transformation → Code Generation
- **Specialized Parsers:** Domain-specific parsers (e.g., `ars_comments.rs`)
- **Output Backends:** Pluggable outputters (EPUB, Markdown)

## External Services

### AI Providers (Optional)
- **Anthropic:** Claude API for front page generation and summaries
- **Ollama:** Local LLM provider for offline AI features
- **Provider Abstraction:** Generic AI client with retry logic

### Content Sources
- **RSS/Atom Feeds:** Standard feed protocol support
- **Ars Technica API:** Specialized integration with comment extraction
- **Extensible:** Trait-based source system for additional providers

## Deployment

- **Distribution:** Standalone CLI binary
- **Platforms:** macOS (Darwin), Linux (via Nix)
- **Dependencies:** OpenSSL, libiconv (provided by Nix)
- **Execution:** Command-line interface with configuration file

## Repository

- **Code Repository:** Local Git repository
- **Version Control:** Git
- **CI/CD:** Not yet configured
- **Documentation:** CLAUDE.md for AI assistant guidance

## Future Infrastructure

### Planned Features
- **Vector Database:** (Planned) For article embedding storage and similarity search
- **UMAP/SVD Processing:** (Planned) Dimensionality reduction for visualization
- **Clustering Engine:** (Planned) Automatic article grouping by topic

Documentation written by Claude Code.
