# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust CLI application called `daily-feed` that fetches RSS feeds. It's built using the rust-nix-template and uses Nix for development environment setup.

It is currently extremely bare-bones, with only a starter template. Eventually 
the application will render EPUB files from the RSS feeds, and push them to 
kindle/boox APIs.

## Development Commands

### Build and Run
- `cargo run` - Build and run the application
- `cargo run -- [ARGS]` - Run with arguments
- `just run [ARGS]` - Alternative way to run via justfile

### Development Workflow
- `just watch [ARGS]` - Run with auto-recompilation using cargo-watch
- `cargo build` - Build the project
- `cargo test` - Run tests
- `cargo check` - Quick compile check

### Formatting
- `just fmt` - Auto-format source tree using treefmt
- `cargo fmt` - Format Rust code specifically

### Nix Environment
- `nix develop` - Enter development shell with all dependencies
- `nix build` - Build the project using Nix

## Architecture

### Main Components
- `src/main.rs` - Entry point with CLI argument parsing using clap
- `src/fetch.rs` - RSS feed fetching functionality using reqwest and rss crates

### Key Dependencies
- `clap` - Command line argument parsing
- `rss` - RSS feed parsing
- `reqwest` - HTTP client for fetching feeds
- `tokio` - Async runtime

### Code Structure
The application is a simple CLI that at the moment:
1. Parses command line arguments (currently only placeholder flags)
2. Fetches an RSS feed from a currently hardcoded URL (Ars Technica)
3. Parses and displays the feed content

## Build System

The project uses both Cargo and Nix:
- Standard Rust toolchain via Cargo
- Nix flake for reproducible development environment
- justfile for common tasks
- treefmt for code formatting

## Development Environment

This project requires:
- OpenSSL development libraries
- libiconv (for macOS)
- pkg-config

These dependencies are automatically provided when using `nix develop`.
