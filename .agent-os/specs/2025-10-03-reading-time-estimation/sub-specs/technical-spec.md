# Technical Specification

This is the technical specification for the spec detailed in @.agent-os/specs/2025-10-03-reading-time-estimation/spec.md

## Technical Requirements

### AST Extensions

- Add `reading_time_minutes: Option<u32>` field to the `Article` struct in `src/ast.rs`
- Add `total_reading_time_minutes: Option<u32>` field to the `Feed` struct in `src/ast.rs`
- Add `total_reading_time_minutes: Option<u32>` field to the `Document` struct in `src/ast.rs`
- Ensure these fields are serialized/deserialized properly for AST JSON export

### Configuration

- Add optional `reading_speed_wpm: Option<u32>` to the `Config` struct in `src/config.rs`
- Default to 200 words per minute if not specified
- Validate that `reading_speed_wpm` is within reasonable range (50-500 WPM) if provided

### Word Count Calculation

- Implement `calculate_word_count()` method on `Article` in `src/ast.rs`
- Count words from all text content in `ContentBlock` variants:
  - `Paragraph(TextContent)`: Count words in all text spans
  - `Heading(_, TextContent)`: Count words in heading text
  - `ListItem(TextContent)`: Count words in list items
  - `BlockQuote(Vec<ContentBlock>)`: Recursively count words in nested blocks
  - `InlineCode(String)` and `CodeBlock(_, String)`: Count words in code (approximate)
- Skip word counting for: `Image`, `Link` (count only link text, not URL), `RawHtml`
- Use simple whitespace-based word splitting: `text.split_whitespace().count()`

### Reading Time Calculation

- Implement `calculate_reading_time(word_count: usize, wpm: u32) -> u32` utility function
- Formula: `reading_time_minutes = (word_count / wpm).ceil()`
- Return result as `u32` representing minutes

### Time Formatting

- Implement `format_reading_time(minutes: u32) -> String` utility function
- Logic:
  - If `minutes < 60`: return `"{minutes} min"`
  - If `minutes >= 60`:
    - Calculate hours: `hours = minutes / 60`
    - Calculate remaining minutes: `mins = minutes % 60`
    - Return `"{hours}h {mins}min"`

### Parser Integration

- Modify `DocumentParser::parse()` in `src/parser.rs` to:
  - Calculate word count for each article after parsing content blocks
  - Calculate reading time for each article using configured WPM
  - Store reading time in `Article.reading_time_minutes`
  - Aggregate article reading times to calculate `Feed.total_reading_time_minutes`
  - Aggregate feed reading times to calculate `Document.total_reading_time_minutes`

### EPUB Output Integration

- Modify `EpubOutputter` in `src/epub_outputter.rs`:
  - Display article reading time in article metadata section (after date, before content)
  - Format: `<p class="reading-time">⏱ {formatted_time}</p>`
  - Add CSS styling for `.reading-time` class (subtle, grey, small font)
  - Include reading time in table of contents entries: `{article_title} ({formatted_time})`
  - Display feed total reading time in feed headers
  - Display document total reading time in the front matter or title page

### Markdown Output Integration

- Modify `MarkdownOutputter` in `src/markdown_outputter.rs`:
  - Display article reading time in article metadata (after date)
  - Format: `⏱ **Reading time:** {formatted_time}`
  - Include reading time in links/references if generating a TOC
  - Display feed total reading time after feed title
  - Display document total reading time at the beginning of the document

### Testing Requirements

- Add snapshot tests for word count calculation with various `ContentBlock` types
- Add snapshot tests for reading time calculation with different word counts and WPM values
- Add snapshot tests for time formatting at boundary values (59 min, 60 min, 120 min, etc.)
- Add integration tests that verify reading times appear in EPUB HTML output
- Add integration tests that verify reading times appear in Markdown output
- Update existing golden tests to include reading time assertions
- Ensure all tests use deterministic word counts and times for snapshot stability

### Performance Considerations

- Word count calculation should happen once per article during parsing
- Cache calculated values in AST fields rather than recalculating during output
- Aggregation should happen in a single pass during document assembly
- Avoid regex or complex text processing; use simple whitespace splitting

### Error Handling

- Gracefully handle edge cases:
  - Articles with zero words should show "< 1 min"
  - Invalid or out-of-range WPM values should fall back to default (200 WPM)
  - Missing reading time data should not break output generation
- Use `Option<u32>` for reading time fields to handle cases where calculation fails
