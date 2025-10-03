# Product Mission

## Pitch

daily-feed is a Rust CLI application that helps information-conscious readers escape endless scrolling by providing a curated, once-a-day newsfeed delivered directly to their e-reader.

## Users

### Primary Customers

- **Intentional Readers**: Individuals who prefer focused, curated content consumption over endless social media scrolling
- **E-reader Enthusiasts**: People who want to consume online content on dedicated reading devices in a distraction-free format

### User Personas

**The Intentional Reader** (25-45 years old)
- **Role:** Knowledge worker, developer, researcher, or content consumer
- **Context:** Values deep reading and focused information consumption; owns an e-reader
- **Pain Points:** Information overload, endless scrolling, difficulty finding quality content, distraction from algorithmic feeds
- **Goals:** Stay informed without sacrificing focus, consume content on preferred devices, maintain healthy reading habits

## The Problem

### Information Overload and Endless Scrolling

Modern content consumption is dominated by algorithmic feeds designed to maximize engagement through endless scrolling. This leads to decreased focus, information overload, and difficulty distinguishing signal from noise.

**Our Solution:** Aggregate content from curated RSS sources and deliver it once daily in EPUB or Markdown format, optimized for e-readers.

### Poor Reading Experience on E-readers

Most online content isn't formatted for e-readers, making it difficult to consume news and articles on dedicated reading devices without manual conversion or browser-based workarounds.

**Our Solution:** Transform RSS feeds and web content into properly formatted EPUB files with clean typography, embedded comments, and AI-generated summaries.

## Differentiators

### Compiler-Style Architecture

Unlike typical feed aggregators that use simple string processing, daily-feed employs a compiler-style pipeline with an Abstract Syntax Tree (AST). This enables robust content transformation, intermediate representation debugging, and extensible output formats.

### AI-Enhanced Curation

Unlike passive RSS readers, daily-feed uses LLM providers (Anthropic Claude, Ollama) to generate intelligent front pages with summaries and themes, helping readers quickly identify interesting content before diving in.

### Specialized Source Integration

Unlike generic RSS aggregators, daily-feed includes specialized integrations (e.g., Ars Technica with comment extraction) that preserve the full context of content, including community discussions and metadata.

## Key Features

### Core Features

- **Multi-Source RSS Aggregation:** Fetch content from multiple RSS/Atom feeds concurrently with robust error handling
- **Specialized Source Support:** Deep integration with sources like Ars Technica including comment extraction and metadata preservation
- **EPUB Generation:** Transform aggregated content into properly formatted EPUB files optimized for e-readers
- **Markdown Output:** Generate clean Markdown files for alternative reading workflows
- **AST-Based Processing:** Compiler-style pipeline with Abstract Syntax Tree for reliable content transformation

### AI & Intelligence Features

- **AI-Powered Front Pages:** LLM-generated summaries and thematic groupings using Anthropic Claude or Ollama
- **Provider Flexibility:** Support for multiple AI providers (Anthropic, Ollama) with graceful degradation
- **Article Summarization:** (Planned) Individual article summaries for quick scanning
- **Interest-Based Sorting:** (Planned) AI-driven sorting by "most interesting" based on user preferences

### Advanced Features

- **Vector Similarity Analysis:** (Planned) UMAP/SVD visualization of article embeddings with automatic clustering
- **Content Relationship Mapping:** (Planned) Identify thematically related articles across different sources
- **AST Export/Import:** Debug and inspect intermediate representations in JSON format
- **Concurrent Processing:** Parallel source fetching for improved performance

### Development Features

- **Snapshot Testing:** Comprehensive test coverage using insta for deterministic assertions
- **Functional Code Style:** Algebraic data types and functional programming patterns ("OCaml with manual garbage collection")
- **Safe Error Handling:** Production code avoids `unwrap()` in favor of proper Result/Option handling
- **Nix Development Environment:** Reproducible builds and dependencies via Nix flakes

Documentation written by Claude Code.
