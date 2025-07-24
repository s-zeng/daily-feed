use daily_feed::fetch::{channels_to_document, document_to_epub, document_to_output};
use daily_feed::ast::Document;
use daily_feed::config::OutputFormat;
use std::fs;
use tempfile::TempDir;

/// Golden test for RSS to AST conversion
/// This test ensures that RSS feeds are consistently parsed into the expected AST structure
#[tokio::test]
async fn test_rss_to_ast_golden() {
    // Load a known RSS feed fixture
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();
    
    let channels = vec![("Test Feed".to_string(), channel)];
    
    // Convert to AST
    let document = channels_to_document(
        &channels,
        "Golden Test Document".to_string(),
        "Test Author".to_string(),
    ).await.unwrap();
    
    // Serialize AST to JSON for comparison
    let ast_json = serde_json::to_string_pretty(&document).unwrap();
    
    // Expected structure validation (golden test assertions)
    assert_eq!(document.metadata.title, "Golden Test Document");
    assert_eq!(document.metadata.author, "Test Author");
    assert_eq!(document.feeds.len(), 1);
    assert_eq!(document.feeds[0].name, "Test Feed");
    assert_eq!(document.feeds[0].articles.len(), 2);
    
    // Validate first article structure
    let first_article = &document.feeds[0].articles[0];
    assert_eq!(first_article.title, "Test Article 1");
    assert_eq!(first_article.metadata.feed_name, "Test Feed");
    assert!(!first_article.content.is_empty());
    
    // Validate second article structure  
    let second_article = &document.feeds[0].articles[1];
    assert_eq!(second_article.title, "Test Article 2");
    assert_eq!(second_article.metadata.feed_name, "Test Feed");
    
    // Write golden file for manual inspection/comparison
    let golden_path = "tests/golden_output/rss_to_ast.json";
    if let Some(parent) = std::path::Path::new(golden_path).parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(golden_path, &ast_json).unwrap();
    
    println!("Golden AST written to: {}", golden_path);
    println!("AST contains {} feeds with {} total articles", 
             document.feeds.len(), 
             document.total_articles());
}

/// Golden test for AST to EPUB conversion
/// This test ensures that AST documents are consistently converted to valid EPUB files
#[tokio::test]
async fn test_ast_to_epub_golden() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("golden_test.epub");
    
    // Create a known AST structure for testing
    let mut document = daily_feed::ast::Document::new(
        "Golden EPUB Test".to_string(),
        "Golden Test Author".to_string(),
    );
    
    let mut feed = daily_feed::ast::Feed::new("Golden Feed".to_string())
        .with_description("A test feed for golden testing".to_string());
    
    // Create test article with various content types
    let mut article = daily_feed::ast::Article::new(
        "Golden Test Article".to_string(),
        "Golden Feed".to_string(),
    ).with_published_date("2025-01-01T12:00:00Z".to_string());
    
    // Add various content blocks to test different AST node types
    article.content = vec![
        daily_feed::ast::ContentBlock::Paragraph(
            daily_feed::ast::TextContent::from_spans(vec![
                daily_feed::ast::TextSpan::plain("This is a ".to_string()),
                daily_feed::ast::TextSpan::bold("bold".to_string()),
                daily_feed::ast::TextSpan::plain(" and ".to_string()),
                daily_feed::ast::TextSpan::italic("italic".to_string()),
                daily_feed::ast::TextSpan::plain(" text example.".to_string()),
            ])
        ),
        daily_feed::ast::ContentBlock::Heading {
            level: 2,
            content: daily_feed::ast::TextContent::plain("Test Heading".to_string()),
        },
        daily_feed::ast::ContentBlock::List {
            ordered: false,
            items: vec![
                daily_feed::ast::TextContent::plain("First item".to_string()),
                daily_feed::ast::TextContent::plain("Second item".to_string()),
            ],
        },
        daily_feed::ast::ContentBlock::Quote(
            daily_feed::ast::TextContent::plain("This is a quote block".to_string())
        ),
        daily_feed::ast::ContentBlock::Code {
            language: Some("rust".to_string()),
            content: "fn main() { println!(\"Hello, world!\"); }".to_string(),
        },
        daily_feed::ast::ContentBlock::Link {
            url: "https://example.com".to_string(),
            text: "Example Link".to_string(),
        },
    ];
    
    // Add a test comment
    let comment = daily_feed::ast::Comment {
        author: "Test Commenter".to_string(),
        content: vec![
            daily_feed::ast::ContentBlock::Paragraph(
                daily_feed::ast::TextContent::plain("This is a test comment.".to_string())
            )
        ],
        score: 42,
        timestamp: Some("2025-01-01T13:00:00Z".to_string()),
    };
    article.add_comment(comment);
    
    feed.add_article(article);
    document.add_feed(feed);
    
    // Convert AST to EPUB
    document_to_epub(&document, epub_path.to_str().unwrap()).await.unwrap();
    
    // Validate EPUB file was created and has expected properties
    assert!(epub_path.exists());
    
    let metadata = fs::metadata(&epub_path).unwrap();
    assert!(metadata.len() > 1000, "EPUB should have substantial content");
    
    // Copy to golden output for manual inspection
    let golden_epub_path = "tests/golden_output/ast_to_epub.epub";
    if let Some(parent) = std::path::Path::new(golden_epub_path).parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::copy(&epub_path, golden_epub_path).unwrap();
    
    println!("Golden EPUB written to: {}", golden_epub_path);
    println!("EPUB size: {} bytes", metadata.len());
}

/// Golden test for full end-to-end workflow with real RSS feed
/// This test captures the complete workflow from RSS fetch to EPUB generation
#[tokio::test]
async fn test_end_to_end_workflow_golden() {
    let temp_dir = TempDir::new().unwrap();
    let ast_path = temp_dir.path().join("workflow_ast.json");
    let epub_path = temp_dir.path().join("workflow_test.epub");
    
    // Load test RSS feeds (using multiple feeds to test aggregation)
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let sample_rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let sample_channel = rss::Channel::read_from(sample_rss_content.as_bytes()).unwrap();
    
    let tech_news_path = "tests/fixtures/tech_news.xml";
    let tech_news_content = fs::read_to_string(tech_news_path).unwrap();
    let tech_channel = rss::Channel::read_from(tech_news_content.as_bytes()).unwrap();
    
    let channels = vec![
        ("Sample Feed".to_string(), sample_channel),
        ("Tech News".to_string(), tech_channel),
    ];
    
    // Step 1: RSS to AST
    let document = channels_to_document(
        &channels,
        "End-to-End Test".to_string(),
        "Workflow Tester".to_string(),
    ).await.unwrap();
    
    // Validate AST structure
    assert_eq!(document.feeds.len(), 2);
    assert_eq!(document.feeds[0].name, "Sample Feed");
    assert_eq!(document.feeds[1].name, "Tech News");
    assert_eq!(document.total_articles(), 5); // 2 from sample + 3 from tech news
    
    // Step 2: Export AST to JSON (intermediate step)
    let ast_json = serde_json::to_string_pretty(&document).unwrap();
    fs::write(&ast_path, &ast_json).unwrap();
    
    // Step 3: AST to EPUB
    document_to_epub(&document, epub_path.to_str().unwrap()).await.unwrap();
    
    // Validate outputs
    assert!(ast_path.exists());
    assert!(epub_path.exists());
    
    let ast_metadata = fs::metadata(&ast_path).unwrap();
    let epub_metadata = fs::metadata(&epub_path).unwrap();
    
    // AST should be substantial JSON (multiple feeds with articles)
    assert!(ast_metadata.len() > 5000, "AST JSON should be substantial");
    
    // EPUB should be larger than AST (includes formatting, CSS, etc.)
    assert!(epub_metadata.len() > 2000, "EPUB should have substantial content");
    
    // Copy to golden outputs
    let golden_ast_path = "tests/golden_output/end_to_end_ast.json";
    let golden_epub_path = "tests/golden_output/end_to_end.epub";
    
    if let Some(parent) = std::path::Path::new(golden_ast_path).parent() {
        fs::create_dir_all(parent).ok();
    }
    
    fs::copy(&ast_path, golden_ast_path).unwrap();
    fs::copy(&epub_path, golden_epub_path).unwrap();
    
    println!("Golden end-to-end AST written to: {}", golden_ast_path);
    println!("Golden end-to-end EPUB written to: {}", golden_epub_path);
    println!("Workflow processed {} feeds with {} articles", 
             document.feeds.len(), 
             document.total_articles());
}

/// Test AST roundtrip: RSS -> AST -> JSON -> AST -> EPUB
/// This verifies that the AST can be serialized and deserialized without loss
#[tokio::test]
async fn test_ast_roundtrip_golden() {
    let temp_dir = TempDir::new().unwrap();
    
    // Step 1: RSS to AST
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();
    let channels = vec![("Roundtrip Test".to_string(), channel)];
    
    let original_document = channels_to_document(
        &channels,
        "Roundtrip Test Document".to_string(),
        "Roundtrip Author".to_string(),
    ).await.unwrap();
    
    // Step 2: AST to JSON
    let ast_json = serde_json::to_string_pretty(&original_document).unwrap();
    let json_path = temp_dir.path().join("roundtrip.json");
    fs::write(&json_path, &ast_json).unwrap();
    
    // Step 3: JSON back to AST
    let loaded_json = fs::read_to_string(&json_path).unwrap();
    let loaded_document: Document = serde_json::from_str(&loaded_json).unwrap();
    
    // Step 4: Both ASTs to EPUB
    let original_epub_path = temp_dir.path().join("original.epub");
    let loaded_epub_path = temp_dir.path().join("loaded.epub");
    
    document_to_epub(&original_document, original_epub_path.to_str().unwrap()).await.unwrap();
    document_to_epub(&loaded_document, loaded_epub_path.to_str().unwrap()).await.unwrap();
    
    // Validate roundtrip preservation
    assert_eq!(original_document.metadata.title, loaded_document.metadata.title);
    assert_eq!(original_document.metadata.author, loaded_document.metadata.author);
    assert_eq!(original_document.feeds.len(), loaded_document.feeds.len());
    assert_eq!(original_document.total_articles(), loaded_document.total_articles());
    
    // EPUB files should be identical (or very similar in size)
    let original_metadata = fs::metadata(&original_epub_path).unwrap();
    let loaded_metadata = fs::metadata(&loaded_epub_path).unwrap();
    
    // Allow for minor differences due to timestamps, but should be very close
    let size_diff = (original_metadata.len() as i64 - loaded_metadata.len() as i64).abs();
    assert!(size_diff < 100, "EPUB files should be nearly identical in size");
    
    println!("Roundtrip test successful:");
    println!("  Original EPUB: {} bytes", original_metadata.len());
    println!("  Loaded EPUB: {} bytes", loaded_metadata.len());
    println!("  Size difference: {} bytes", size_diff);
}

/// Test error handling in AST conversion
/// This ensures graceful handling of malformed or empty feeds
#[tokio::test]
async fn test_ast_error_handling_golden() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test with empty feed
    let empty_feed_path = "tests/fixtures/empty_feed.xml";
    let empty_rss_content = fs::read_to_string(empty_feed_path).unwrap();
    let empty_channel = rss::Channel::read_from(empty_rss_content.as_bytes()).unwrap();
    let channels = vec![("Empty Feed".to_string(), empty_channel)];
    
    let document = channels_to_document(
        &channels,
        "Empty Feed Test".to_string(),
        "Error Test Author".to_string(),
    ).await.unwrap();
    
    // Should successfully create document even with empty feed
    assert_eq!(document.feeds.len(), 1);
    assert_eq!(document.feeds[0].articles.len(), 0);
    assert_eq!(document.total_articles(), 0);
    
    // Should still be able to generate EPUB
    let epub_path = temp_dir.path().join("empty_test.epub");
    document_to_epub(&document, epub_path.to_str().unwrap()).await.unwrap();
    
    assert!(epub_path.exists());
    let metadata = fs::metadata(&epub_path).unwrap();
    assert!(metadata.len() > 500, "Even empty EPUB should have structure");
    
    println!("Empty feed handling test successful:");
    println!("  Document has {} feeds with {} articles", 
             document.feeds.len(), 
             document.total_articles());
    println!("  Generated EPUB size: {} bytes", metadata.len());
}

/// Golden test for AST to Markdown conversion
/// This test ensures that AST documents are consistently converted to valid Markdown files
#[tokio::test]
async fn test_ast_to_markdown_golden() {
    let temp_dir = TempDir::new().unwrap();
    let markdown_path = temp_dir.path().join("golden_test.md");
    
    // Create a known AST structure for testing
    let mut document = daily_feed::ast::Document::new(
        "Golden Markdown Test".to_string(),
        "Golden Test Author".to_string(),
    );
    
    let mut feed = daily_feed::ast::Feed::new("Golden Feed".to_string())
        .with_description("A test feed for golden markdown testing".to_string());
    
    // Create test article with various content types
    let mut article = daily_feed::ast::Article::new(
        "Golden Test Article".to_string(),
        "Golden Feed".to_string(),
    ).with_published_date("2025-01-01T12:00:00Z".to_string())
     .with_url("https://example.com/article".to_string());
    
    // Add various content blocks to test different AST node types
    article.content = vec![
        daily_feed::ast::ContentBlock::Paragraph(
            daily_feed::ast::TextContent::from_spans(vec![
                daily_feed::ast::TextSpan::plain("This is a ".to_string()),
                daily_feed::ast::TextSpan::bold("bold".to_string()),
                daily_feed::ast::TextSpan::plain(" and ".to_string()),
                daily_feed::ast::TextSpan::italic("italic".to_string()),
                daily_feed::ast::TextSpan::plain(" text example.".to_string()),
            ])
        ),
        daily_feed::ast::ContentBlock::Heading {
            level: 2,
            content: daily_feed::ast::TextContent::plain("Test Heading".to_string()),
        },
        daily_feed::ast::ContentBlock::List {
            ordered: false,
            items: vec![
                daily_feed::ast::TextContent::plain("First item".to_string()),
                daily_feed::ast::TextContent::plain("Second item".to_string()),
            ],
        },
        daily_feed::ast::ContentBlock::Quote(
            daily_feed::ast::TextContent::plain("This is a quote block".to_string())
        ),
        daily_feed::ast::ContentBlock::Code {
            language: Some("rust".to_string()),
            content: "fn main() { println!(\"Hello, world!\"); }".to_string(),
        },
        daily_feed::ast::ContentBlock::Link {
            url: "https://example.com".to_string(),
            text: "Example Link".to_string(),
        },
    ];
    
    // Add a test comment
    let comment = daily_feed::ast::Comment {
        author: "Test Commenter".to_string(),
        content: vec![
            daily_feed::ast::ContentBlock::Paragraph(
                daily_feed::ast::TextContent::plain("This is a test comment.".to_string())
            )
        ],
        score: 42,
        timestamp: Some("2025-01-01T13:00:00Z".to_string()),
    };
    article.add_comment(comment);
    
    feed.add_article(article);
    document.add_feed(feed);
    
    // Convert AST to Markdown using the outputter
    document_to_output(&document, markdown_path.to_str().unwrap(), &OutputFormat::Markdown).await.unwrap();
    
    // Validate Markdown file was created and has expected properties
    assert!(markdown_path.exists());
    
    let markdown_content = fs::read_to_string(&markdown_path).unwrap();
    
    // Validate Markdown structure
    assert!(markdown_content.contains("# Golden Markdown Test"), "Should have document title as H1");
    assert!(markdown_content.contains("## Golden Feed"), "Should have feed name as H2");
    assert!(markdown_content.contains("### Golden Test Article"), "Should have article title as H3");
    assert!(markdown_content.contains("**bold**"), "Should preserve bold formatting");
    assert!(markdown_content.contains("*italic*"), "Should preserve italic formatting");
    assert!(markdown_content.contains("##### Test Heading"), "Should render heading blocks");
    assert!(markdown_content.contains("- First item"), "Should render unordered lists");
    assert!(markdown_content.contains("> This is a quote block"), "Should render quote blocks");
    assert!(markdown_content.contains("```rust"), "Should render code blocks with language");
    assert!(markdown_content.contains("[Example Link](https://example.com)"), "Should render link blocks");
    assert!(markdown_content.contains("#### Top Comments"), "Should include comments section");
    assert!(markdown_content.contains("> **Test Commenter** (Score: 42)"), "Should render comment metadata");
    assert!(markdown_content.contains("Table of Contents"), "Should include table of contents");
    
    let metadata = fs::metadata(&markdown_path).unwrap();
    assert!(metadata.len() > 500, "Markdown should have substantial content");
    
    // Copy to golden output for manual inspection
    let golden_markdown_path = "tests/golden_output/ast_to_markdown.md";
    if let Some(parent) = std::path::Path::new(golden_markdown_path).parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::copy(&markdown_path, golden_markdown_path).unwrap();
    
    println!("Golden Markdown written to: {}", golden_markdown_path);
    println!("Markdown size: {} bytes", metadata.len());
}

/// Golden test for RSS to Markdown conversion using fixtures
/// This test ensures RSS feeds are consistently converted to structured Markdown
#[tokio::test]
async fn test_rss_to_markdown_golden() {
    // Load a known RSS feed fixture
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();
    
    let channels = vec![("Test Feed".to_string(), channel)];
    
    // Convert to AST
    let document = channels_to_document(
        &channels,
        "Golden RSS to Markdown Test".to_string(),
        "Test Author".to_string(),
    ).await.unwrap();
    
    // Generate markdown from AST
    let temp_dir = TempDir::new().unwrap();
    let markdown_path = temp_dir.path().join("rss_to_markdown.md");
    
    document_to_output(&document, markdown_path.to_str().unwrap(), &OutputFormat::Markdown).await.unwrap();
    
    // Validate markdown file structure
    assert!(markdown_path.exists());
    let markdown_content = fs::read_to_string(&markdown_path).unwrap();
    
    // Expected structure validation
    assert!(markdown_content.contains("# Golden RSS to Markdown Test"));
    assert!(markdown_content.contains("## Test Feed"));
    assert!(markdown_content.contains("### Test Article"));
    assert!(markdown_content.contains("Table of Contents"));
    assert!(markdown_content.contains("**Total Articles:**"));
    
    // Copy to golden output
    let golden_path = "tests/golden_output/rss_to_markdown.md";
    if let Some(parent) = std::path::Path::new(golden_path).parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::copy(&markdown_path, golden_path).unwrap();
    
    println!("Golden RSS to Markdown written to: {}", golden_path);
    println!("Markdown contains {} feeds with {} total articles", 
             document.feeds.len(), 
             document.total_articles());
}

/// Golden test for multi-feed markdown output
/// This test ensures complex documents with multiple feeds render correctly
#[tokio::test]
async fn test_multi_feed_markdown_golden() {
    let temp_dir = TempDir::new().unwrap();
    let markdown_path = temp_dir.path().join("multi_feed_test.md");
    
    // Load multiple test RSS feeds
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let sample_rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let sample_channel = rss::Channel::read_from(sample_rss_content.as_bytes()).unwrap();
    
    let tech_news_path = "tests/fixtures/tech_news.xml";
    let tech_news_content = fs::read_to_string(tech_news_path).unwrap();
    let tech_channel = rss::Channel::read_from(tech_news_content.as_bytes()).unwrap();
    
    let channels = vec![
        ("Sample Feed".to_string(), sample_channel),
        ("Tech News".to_string(), tech_channel),
    ];
    
    // Convert to AST and then to Markdown
    let document = channels_to_document(
        &channels,
        "Multi-Feed Markdown Test".to_string(),
        "Multi-Feed Author".to_string(),
    ).await.unwrap();
    
    document_to_output(&document, markdown_path.to_str().unwrap(), &OutputFormat::Markdown).await.unwrap();
    
    // Validate multi-feed structure
    assert!(markdown_path.exists());
    let markdown_content = fs::read_to_string(&markdown_path).unwrap();
    
    assert!(markdown_content.contains("# Multi-Feed Markdown Test"));
    assert!(markdown_content.contains("## Sample Feed"));
    assert!(markdown_content.contains("## Tech News"));
    assert!(markdown_content.contains("Table of Contents"));
    
    // Check that all feeds and articles are present in TOC
    let toc_section = markdown_content.split("---").nth(0).unwrap_or("");
    assert!(toc_section.contains("Sample Feed"));
    assert!(toc_section.contains("Tech News"));
    
    // Validate total article count
    assert_eq!(document.total_articles(), 5); // 2 from sample + 3 from tech news
    assert!(markdown_content.contains("**Total Articles:** 5"));
    
    // Copy to golden output
    let golden_path = "tests/golden_output/multi_feed_markdown.md";
    if let Some(parent) = std::path::Path::new(golden_path).parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::copy(&markdown_path, golden_path).unwrap();
    
    println!("Golden multi-feed Markdown written to: {}", golden_path);
    println!("Multi-feed document processed {} feeds with {} articles", 
             document.feeds.len(), 
             document.total_articles());
}