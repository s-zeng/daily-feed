# Daily Feed Tests

Documentation written entirely by Claude Code.

This directory contains comprehensive tests for the daily-feed RSS aggregator, including both traditional unit tests and cram-style integration tests.

## Test Structure

### Unit Tests
- `tests/fetch_tests.rs` - Tests for RSS fetching and EPUB generation
- `tests/integration_tests.rs` - End-to-end workflow tests with real RSS data

### Cram-Style Tests
- `tests/cram_tests.rs` - Command-line behavior validation tests

### Golden Reference Tests
- `tests/golden_tests.rs` - Reference output validation tests
- `tests/golden_output/` - Golden reference files for regression testing

## Cram-Style Testing

The cram-style tests in `cram_tests.rs` validate the expected behavior of the RSS-to-AST and AST-to-EPUB conversion pipeline in a reproducible way. These tests capture:

### RSS to AST Export
- **Input**: RSS feed content with known structure
- **Expected Output**: Structured AST with preserved content and formatting
- **Validation**: JSON serialization, metadata preservation, content block structure

### AST to EPUB Conversion  
- **Input**: Known AST document structure
- **Expected Output**: Valid EPUB file with proper formatting
- **Validation**: File creation, size validation, format verification

### End-to-End Workflow
- **Pipeline**: RSS → AST → JSON → EPUB
- **Validation**: Content preservation, structure integrity, roundtrip consistency

### Error Handling
- **Edge Cases**: Empty feeds, malformed HTML, invalid JSON
- **Expected Behavior**: Graceful error handling, meaningful error messages

### Output Format Validation
- **JSON Export**: Valid structure with expected schema
- **EPUB Export**: Proper file format and content organization

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test cram_tests           # Cram-style tests
cargo test --test golden_tests         # Golden reference tests
cargo test --test fetch_tests          # RSS fetching tests
cargo test --test integration_tests    # Integration tests

# Run with output
cargo test --test cram_tests -- --nocapture
```

## Expected Test Output Examples

### RSS to AST Export
```
✓ RSS to AST export - Expected behavior verified
  Document: 1 feeds, 2 articles
  JSON size: 5635 bytes
```

### AST to EPUB Conversion
```
✓ AST to EPUB conversion - Expected behavior verified
  EPUB size: 4272 bytes
  Archive contains: mimetype, OPF, XHTML files
```

### End-to-End Content Preservation
```
✓ End-to-end content preservation - Expected behavior verified
  Pipeline: RSS -> AST (5340 bytes) -> EPUB (4517 bytes)
  Content blocks: paragraph=true, structured=true
```

## Test Philosophy

The cram-style tests focus on **expected behavior** rather than implementation details:

1. **Input/Output Validation**: Test that given inputs produce expected outputs
2. **Format Compliance**: Ensure generated files meet format specifications
3. **Error Resilience**: Verify graceful handling of edge cases and malformed inputs
4. **Content Preservation**: Validate that the RSS → AST → EPUB pipeline preserves content structure and formatting

This approach provides confidence that the refactored compiler-like architecture maintains the expected functionality while enabling future changes to the implementation without breaking tests.
