use daily_feed::ast::Document;
use daily_feed::config::OutputFormat;
use daily_feed::fetch::{document_to_epub, document_to_output};
use daily_feed::parser::parse_feeds_to_document;
use std::fs;
use tempfile::TempDir;

/// Golden test for RSS to AST conversion
/// This test ensures that RSS feeds are consistently parsed into the expected AST structure
#[tokio::test]
async fn test_rss_to_ast_golden() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let channels = vec![("Test Feed".to_string(), channel)];

    let mut document = parse_feeds_to_document(
        &channels,
        "Golden Test Document".to_string(),
        "Test Author".to_string(),
    )
    .await
    .unwrap();

    // Filter out the timestamp for reproducible snapshots
    document.metadata.generated_at = "2025-01-01T00:00:00.000000Z".to_string();

    insta::assert_json_snapshot!(document);
}

/// Golden test for AST to EPUB conversion
/// This test ensures that AST documents are consistently converted to valid EPUB files
#[tokio::test]
async fn test_ast_to_epub_golden() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("golden_test.epub");

    let mut document = daily_feed::ast::Document::new(
        "Golden EPUB Test".to_string(),
        "Golden Test Author".to_string(),
    );

    let mut feed = daily_feed::ast::Feed::new("Golden Feed".to_string())
        .with_description("A test feed for golden testing".to_string());

    let mut article =
        daily_feed::ast::Article::new("Golden Test Article".to_string(), "Golden Feed".to_string())
            .with_published_date("2025-01-01T12:00:00Z".to_string());

    article.content = vec![
        daily_feed::ast::ContentBlock::Paragraph(daily_feed::ast::TextContent::from_spans(vec![
            daily_feed::ast::TextSpan::plain("This is a ".to_string()),
            daily_feed::ast::TextSpan::bold("bold".to_string()),
            daily_feed::ast::TextSpan::plain(" and ".to_string()),
            daily_feed::ast::TextSpan::italic("italic".to_string()),
            daily_feed::ast::TextSpan::plain(" text example.".to_string()),
        ])),
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
        daily_feed::ast::ContentBlock::Quote(daily_feed::ast::TextContent::plain(
            "This is a quote block".to_string(),
        )),
        daily_feed::ast::ContentBlock::Code {
            language: Some("rust".to_string()),
            content: "fn main() { println!(\"Hello, world!\"); }".to_string(),
        },
        daily_feed::ast::ContentBlock::Link {
            url: "https://example.com".to_string(),
            text: "Example Link".to_string(),
        },
    ];

    let comment = daily_feed::ast::Comment {
        author: "Test Commenter".to_string(),
        content: vec![daily_feed::ast::ContentBlock::Paragraph(
            daily_feed::ast::TextContent::plain("This is a test comment.".to_string()),
        )],
        upvotes: 50,
        downvotes: 8,
        timestamp: Some("2025-01-01T13:00:00Z".to_string()),
    };
    article.add_comment(comment);

    feed.add_article(article);
    document.add_feed(feed);

    document_to_epub(&document, epub_path.to_str().unwrap())
        .await
        .unwrap();

    let file_size_valid = if epub_path.exists() {
        let size = fs::metadata(&epub_path).unwrap().len();
        size > 1000 && size < 10000
    } else {
        false
    };

    insta::assert_snapshot!(format!(
        "file_exists: {}, file_size_valid: {}",
        epub_path.exists(),
        file_size_valid
    ));
}

/// Golden test for full end-to-end workflow with real RSS feed
/// This test captures the complete workflow from RSS fetch to EPUB generation
#[tokio::test]
async fn test_end_to_end_workflow_golden() {
    let temp_dir = TempDir::new().unwrap();
    let ast_path = temp_dir.path().join("workflow_ast.json");
    let epub_path = temp_dir.path().join("workflow_test.epub");

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

    let document = parse_feeds_to_document(
        &channels,
        "End-to-End Test".to_string(),
        "Workflow Tester".to_string(),
    )
    .await
    .unwrap();

    let ast_json = serde_json::to_string_pretty(&document).unwrap();
    fs::write(&ast_path, &ast_json).unwrap();

    document_to_epub(&document, epub_path.to_str().unwrap())
        .await
        .unwrap();

    let workflow_result = format!("feeds: {}, articles: {}, ast_exists: {}, epub_exists: {}, ast_size_valid: {}, epub_size_valid: {}",
        document.feeds.len(),
        document.total_articles(),
        ast_path.exists(),
        epub_path.exists(),
        if ast_path.exists() {
            let size = fs::metadata(&ast_path).unwrap().len();
            size > 3000 && size < 10000
        } else { false },
        if epub_path.exists() {
            let size = fs::metadata(&epub_path).unwrap().len();
            size > 3000 && size < 10000
        } else { false }
    );

    insta::assert_snapshot!(workflow_result);
}

/// Test AST roundtrip: RSS -> AST -> JSON -> AST -> EPUB
/// This verifies that the AST can be serialized and deserialized without loss
#[tokio::test]
async fn test_ast_roundtrip_golden() {
    let temp_dir = TempDir::new().unwrap();

    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();
    let channels = vec![("Roundtrip Test".to_string(), channel)];

    let original_document = parse_feeds_to_document(
        &channels,
        "Roundtrip Test Document".to_string(),
        "Roundtrip Author".to_string(),
    )
    .await
    .unwrap();

    let ast_json = serde_json::to_string_pretty(&original_document).unwrap();
    let json_path = temp_dir.path().join("roundtrip.json");
    fs::write(&json_path, &ast_json).unwrap();

    let loaded_json = fs::read_to_string(&json_path).unwrap();
    let loaded_document: Document = serde_json::from_str(&loaded_json).unwrap();

    let original_epub_path = temp_dir.path().join("original.epub");
    let loaded_epub_path = temp_dir.path().join("loaded.epub");

    document_to_epub(&original_document, original_epub_path.to_str().unwrap())
        .await
        .unwrap();
    document_to_epub(&loaded_document, loaded_epub_path.to_str().unwrap())
        .await
        .unwrap();

    let original_metadata = fs::metadata(&original_epub_path).unwrap();
    let loaded_metadata = fs::metadata(&loaded_epub_path).unwrap();
    let size_diff = (original_metadata.len() as i64 - loaded_metadata.len() as i64).abs();

    let roundtrip_result = format!("title_match: {}, author_match: {}, feeds_match: {}, articles_match: {}, size_diff_acceptable: {}",
        original_document.metadata.title == loaded_document.metadata.title,
        original_document.metadata.author == loaded_document.metadata.author,
        original_document.feeds.len() == loaded_document.feeds.len(),
        original_document.total_articles() == loaded_document.total_articles(),
        size_diff < 100
    );

    insta::assert_snapshot!(roundtrip_result);
}

/// Test error handling in AST conversion
/// This ensures graceful handling of malformed or empty feeds
#[tokio::test]
async fn test_ast_error_handling_golden() {
    let temp_dir = TempDir::new().unwrap();

    let empty_feed_path = "tests/fixtures/empty_feed.xml";
    let empty_rss_content = fs::read_to_string(empty_feed_path).unwrap();
    let empty_channel = rss::Channel::read_from(empty_rss_content.as_bytes()).unwrap();
    let channels = vec![("Empty Feed".to_string(), empty_channel)];

    let document = parse_feeds_to_document(
        &channels,
        "Empty Feed Test".to_string(),
        "Error Test Author".to_string(),
    )
    .await
    .unwrap();

    let epub_path = temp_dir.path().join("empty_test.epub");
    document_to_epub(&document, epub_path.to_str().unwrap())
        .await
        .unwrap();

    let file_size = if epub_path.exists() {
        fs::metadata(&epub_path).unwrap().len()
    } else {
        0
    };

    let error_handling_result = format!(
        "feeds: {}, articles: {}, file_exists: {}, file_size_valid: {}",
        document.feeds.len(),
        document.total_articles(),
        epub_path.exists(),
        file_size > 1000 && file_size < 10000
    );

    insta::assert_snapshot!(error_handling_result);
}

/// Golden test for AST to Markdown conversion
/// This test ensures that AST documents are consistently converted to valid Markdown files
#[tokio::test]
async fn test_ast_to_markdown_golden() {
    let temp_dir = TempDir::new().unwrap();
    let markdown_path = temp_dir.path().join("golden_test.md");

    let mut document = daily_feed::ast::Document::new(
        "Golden Markdown Test".to_string(),
        "Golden Test Author".to_string(),
    );

    let mut feed = daily_feed::ast::Feed::new("Golden Feed".to_string())
        .with_description("A test feed for golden markdown testing".to_string());

    let mut article =
        daily_feed::ast::Article::new("Golden Test Article".to_string(), "Golden Feed".to_string())
            .with_published_date("2025-01-01T12:00:00Z".to_string())
            .with_url("https://example.com/article".to_string());

    article.content = vec![
        daily_feed::ast::ContentBlock::Paragraph(daily_feed::ast::TextContent::from_spans(vec![
            daily_feed::ast::TextSpan::plain("This is a ".to_string()),
            daily_feed::ast::TextSpan::bold("bold".to_string()),
            daily_feed::ast::TextSpan::plain(" and ".to_string()),
            daily_feed::ast::TextSpan::italic("italic".to_string()),
            daily_feed::ast::TextSpan::plain(" text example.".to_string()),
        ])),
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
    ];

    let comment = daily_feed::ast::Comment {
        author: "Test Commenter".to_string(),
        content: vec![daily_feed::ast::ContentBlock::Paragraph(
            daily_feed::ast::TextContent::plain("This is a test comment.".to_string()),
        )],
        upvotes: 50,
        downvotes: 8,
        timestamp: Some("2025-01-01T13:00:00Z".to_string()),
    };
    article.add_comment(comment);

    feed.add_article(article);
    document.add_feed(feed);

    document_to_output(
        &document,
        markdown_path.to_str().unwrap(),
        &OutputFormat::Markdown,
    )
    .await
    .unwrap();

    let markdown_content = fs::read_to_string(&markdown_path).unwrap();
    let file_size = fs::metadata(&markdown_path).unwrap().len();

    let markdown_features = format!(
        "has_h1: {}, has_h2: {}, has_bold: {}, has_toc: {}, size_range: {}",
        markdown_content.contains("# Golden Markdown Test"),
        markdown_content.contains("## Golden Feed"),
        markdown_content.contains("**bold**"),
        markdown_content.contains("Table of Contents"),
        if file_size >= 650 && file_size <= 700 { "650-700_bytes" } else { "unexpected_size" }
    );

    insta::assert_snapshot!(markdown_features);
}

/// Golden test for RSS to Markdown conversion using fixtures
/// This test ensures RSS feeds are consistently converted to structured Markdown
#[tokio::test]
async fn test_rss_to_markdown_golden() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let channels = vec![("Test Feed".to_string(), channel)];

    let document = parse_feeds_to_document(
        &channels,
        "Golden RSS to Markdown Test".to_string(),
        "Test Author".to_string(),
    )
    .await
    .unwrap();

    let temp_dir = TempDir::new().unwrap();
    let markdown_path = temp_dir.path().join("rss_to_markdown.md");

    document_to_output(
        &document,
        markdown_path.to_str().unwrap(),
        &OutputFormat::Markdown,
    )
    .await
    .unwrap();

    let markdown_content = fs::read_to_string(&markdown_path).unwrap();

    let rss_markdown_features = format!(
        "has_title: {}, has_feed: {}, has_article: {}, has_toc: {}, feeds: {}, articles: {}",
        markdown_content.contains("# Golden RSS to Markdown Test"),
        markdown_content.contains("## Test Feed"),
        markdown_content.contains("### Test Article"),
        markdown_content.contains("Table of Contents"),
        document.feeds.len(),
        document.total_articles()
    );

    insta::assert_snapshot!(rss_markdown_features);
}

/// Golden test for multi-feed markdown output
/// This test ensures complex documents with multiple feeds render correctly
#[tokio::test]
async fn test_multi_feed_markdown_golden() {
    let temp_dir = TempDir::new().unwrap();
    let markdown_path = temp_dir.path().join("multi_feed_test.md");

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

    let document = parse_feeds_to_document(
        &channels,
        "Multi-Feed Markdown Test".to_string(),
        "Multi-Feed Author".to_string(),
    )
    .await
    .unwrap();

    document_to_output(
        &document,
        markdown_path.to_str().unwrap(),
        &OutputFormat::Markdown,
    )
    .await
    .unwrap();

    let markdown_content = fs::read_to_string(&markdown_path).unwrap();

    let multi_feed_result = format!(
        "has_title: {}, has_sample_feed: {}, has_tech_news: {}, has_toc: {}, total_articles: {}",
        markdown_content.contains("# Multi-Feed Markdown Test"),
        markdown_content.contains("## Sample Feed"),
        markdown_content.contains("## Tech News"),
        markdown_content.contains("Table of Contents"),
        document.total_articles()
    );

    insta::assert_snapshot!(multi_feed_result);
}
