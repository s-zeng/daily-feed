# Spec Tasks

These are the tasks to be completed for the spec detailed in @.agent-os/specs/2025-10-03-reading-time-estimation/spec.md

> Created: 2025-10-03
> Status: Ready for Implementation

## Tasks

- [x] 1. Add reading time fields to AST and configuration
  - [x] 1.1 Write tests for reading time AST fields serialization/deserialization
  - [x] 1.2 Add `reading_time_minutes: Option<u32>` field to `Article` struct in `src/ast.rs`
  - [x] 1.3 Add `total_reading_time_minutes: Option<u32>` field to `Feed` struct in `src/ast.rs`
  - [x] 1.4 Add `total_reading_time_minutes: Option<u32>` field to `Document` struct in `src/ast.rs`
  - [x] 1.5 Add `reading_speed_wpm: Option<u32>` field to `Config` struct in `src/config.rs` with validation (50-500 WPM range)
  - [x] 1.6 Verify all tests pass

- [ ] 2. Implement word count and reading time calculation
  - [ ] 2.1 Write tests for word count calculation with various `ContentBlock` types
  - [ ] 2.2 Implement `calculate_word_count()` method on `Article` in `src/ast.rs`
  - [ ] 2.3 Write tests for reading time calculation with different word counts and WPM values
  - [ ] 2.4 Implement `calculate_reading_time(word_count, wpm)` utility function
  - [ ] 2.5 Write tests for time formatting at boundary values (59 min, 60 min, 120 min)
  - [ ] 2.6 Implement `format_reading_time(minutes)` utility function
  - [ ] 2.7 Write tests for edge case: zero words should show "< 1 min"
  - [ ] 2.8 Verify all tests pass

- [ ] 3. Integrate reading time calculation in parser
  - [ ] 3.1 Write tests for parser integration with reading time calculation
  - [ ] 3.2 Modify `DocumentParser::parse()` to calculate word count for each article
  - [ ] 3.3 Calculate and store reading time in `Article.reading_time_minutes` using configured WPM
  - [ ] 3.4 Aggregate article reading times to calculate `Feed.total_reading_time_minutes`
  - [ ] 3.5 Aggregate feed reading times to calculate `Document.total_reading_time_minutes`
  - [ ] 3.6 Verify all tests pass

- [ ] 4. Add reading time display to EPUB output
  - [ ] 4.1 Write tests for EPUB output with reading time display
  - [ ] 4.2 Add reading time to article metadata section in `EpubOutputter` with format: `<p class="reading-time">⏱ {formatted_time}</p>`
  - [ ] 4.3 Add CSS styling for `.reading-time` class (subtle, grey, small font)
  - [ ] 4.4 Include reading time in table of contents entries with format: `{article_title} ({formatted_time})`
  - [ ] 4.5 Display feed total reading time in feed headers
  - [ ] 4.6 Display document total reading time in front matter
  - [ ] 4.7 Verify all tests pass

- [ ] 5. Add reading time display to Markdown output
  - [ ] 5.1 Write tests for Markdown output with reading time display
  - [ ] 5.2 Add reading time to article metadata in `MarkdownOutputter` with format: `⏱ **Reading time:** {formatted_time}`
  - [ ] 5.3 Include reading time in TOC if present
  - [ ] 5.4 Display feed total reading time after feed title
  - [ ] 5.5 Display document total reading time at document beginning
  - [ ] 5.6 Update golden tests to include reading time assertions
  - [ ] 5.7 Verify all tests pass

- [ ] 6. Final validation and edge case testing
  - [ ] 6.1 Verify reading times are correctly calculated for articles with complex content blocks (nested quotes, lists)
  - [ ] 6.2 Test that invalid WPM values fall back to default (200 WPM)
  - [ ] 6.3 Test that missing reading time data doesn't break output generation
  - [ ] 6.4 Verify AST JSON export includes reading time fields
  - [ ] 6.5 Verify AST JSON import correctly deserializes reading time fields
  - [ ] 6.6 Run full integration tests with both EPUB and Markdown outputs
  - [ ] 6.7 Verify all snapshot tests are deterministic and pass consistently
