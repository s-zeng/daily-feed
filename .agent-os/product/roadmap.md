# Product Roadmap

## Phase 0: Already Completed

The following features have been implemented:

**Goal:** Core content aggregation and EPUB generation
**Status:** ✅ Complete

### Features

- [x] Multi-source RSS aggregation - Concurrent fetching from multiple RSS/Atom feeds `M`
- [x] Specialized Ars Technica integration - Deep integration with comment extraction `M`
- [x] EPUB output format - Generate properly formatted EPUB files for e-readers `L`
- [x] Markdown output format - Alternative output for Markdown-based workflows `M`
- [x] Compiler-style AST architecture - Robust content transformation pipeline `L`
- [x] AI-powered front page generation - LLM-generated summaries using Anthropic/Ollama `M`
- [x] AST export/import - Debug intermediate representations via JSON `S`
- [x] HTTP utilities with timeouts - Safe network operations with proper error handling `S`
- [x] Comprehensive snapshot testing - Full test coverage using insta framework `L`
- [x] Nix development environment - Reproducible builds and dependencies `M`

### Dependencies

- None (foundational phase)

---

## Phase 1: AI Enhancement & Content Intelligence

**Goal:** Enhance content curation with advanced AI features at generation time
**Success Criteria:** Generated EPUBs contain AI-enhanced content that helps readers quickly identify interesting articles

### Features

- [ ] Article summarization - Generate AI-powered summaries for individual articles included in the EPUB `M`
- [ ] Interest-based sorting - AI-driven sorting by "most interesting" to reorder articles in the output `L`
- [ ] Configurable AI prompts - Allow users to customize summarization and sorting behavior via config `S`
- [ ] Multi-model support expansion - Add support for additional LLM providers (OpenAI, local models) `M`
- [ ] Summary caching - Cache AI-generated summaries to reduce API costs on repeated runs `S`

### Dependencies

- Existing AI client infrastructure (`ai_client.rs`, `front_page.rs`)
- LLM provider APIs (Anthropic, Ollama, etc.)

---

## Phase 2: Vector Similarity & Content Mapping

**Goal:** Generate visual content maps and identify relationships at EPUB generation time
**Success Criteria:** EPUBs include static visualizations showing article relationships and clusters

### Features

- [ ] Article embedding generation - Generate vector embeddings for all articles during processing `M`
- [ ] UMAP dimensionality reduction - Project embeddings into 2D space for visualization `L`
- [ ] SVD analysis - Alternative dimensionality reduction approach `M`
- [ ] Automatic clustering - Identify thematic clusters in article embeddings `M`
- [ ] Static cluster visualization - Generate SVG/HTML visualizations embedded in EPUB `L`
- [ ] Cluster-based organization - Reorganize EPUB table of contents by thematic clusters `M`
- [ ] Cross-source topic detection - Tag articles with common themes across sources `M`

### Dependencies

- Vector embedding service (OpenAI embeddings, local embedding models)
- UMAP/dimensionality reduction libraries (Rust crates)
- Clustering algorithms (k-means, DBSCAN, etc.)
- SVG generation for embedding visualizations in EPUB

---

## Phase 3: Configuration-Based Preferences

**Goal:** Allow users to specify preferences in config file to customize output
**Success Criteria:** Users can configure filters, weights, and preferences that affect generated EPUBs

### Features

- [ ] Topic filtering - Config-based topic filters to include/exclude specific keywords or categories `S`
- [ ] Source weighting - Adjust article count/prominence from different RSS sources `S`
- [ ] Custom front page templates - User-defined front page formats in config `M`
- [ ] Reading time estimation - Calculate and display estimated reading time per article `XS`
- [ ] Article limits - Configure max articles per source or per EPUB `XS`
- [ ] Content length filtering - Include only articles within certain length ranges `S`

### Dependencies

- Extended configuration schema in `config.rs`
- Phase 1 AI features (for intelligent filtering)

---

## Phase 4: Advanced Source Integration

**Goal:** Expand source types and improve content extraction
**Success Criteria:** Support for diverse content sources with high-quality extraction

### Features

- [ ] Substack support - Specialized integration for Substack publications `M`
- [ ] Hacker News integration - Include HN discussions with top articles `M`
- [ ] Reddit support - Fetch subreddit posts with top comments `M`
- [ ] Newsletter parsing - Parse HTML newsletters from email exports `L`
- [ ] YouTube transcripts - Include video transcripts from feed entries `L`
- [ ] Podcast show notes - Extract and format podcast episode notes `M`
- [ ] Custom source plugins - Plugin/script system for user-defined sources `XL`

### Dependencies

- Trait-based source abstraction (already exists in `sources.rs`)
- API access to various platforms
- Content extraction libraries for different formats

---

## Phase 5: Automation & Distribution

**Goal:** Automate daily EPUB generation and delivery
**Success Criteria:** Daily feeds delivered automatically to e-readers without manual intervention

### Features

- [ ] Scheduled execution - systemd timer or cron configuration for daily runs `S`
- [ ] Email delivery - Send generated EPUBs via email (e.g., to Kindle email address) `M`
- [ ] Cloud storage sync - Automatically upload to Dropbox/Google Drive `M`
- [ ] Calibre integration - Auto-import to Calibre library `S`
- [ ] Incremental updates - Only fetch new articles since last run `M`
- [ ] Error notifications - Alert on failed runs or empty feeds `S`
- [ ] Multi-output generation - Generate multiple EPUBs with different configs in one run `M`

### Dependencies

- Email service integration (SMTP or email API)
- Cloud storage APIs
- Calibre CLI tools
- Persistent state storage (track last fetch times)

---

## Effort Scale

- **XS:** 1 day
- **S:** 2-3 days
- **M:** 1 week
- **L:** 2 weeks
- **XL:** 3+ weeks

Documentation written by Claude Code.
