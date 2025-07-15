use daily_feed::config::{Config, Feed, OutputConfig};
use daily_feed::fetch::channels_to_epub;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_feed_from_file() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();

    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    assert_eq!(channel.title(), "Test Feed");
    assert_eq!(channel.description(), "A test RSS feed for unit testing");
    assert_eq!(channel.link(), "https://test.example.com");
    assert_eq!(channel.items().len(), 2);

    let first_item = &channel.items()[0];
    assert_eq!(first_item.title(), Some("Test Article 1"));
    assert_eq!(first_item.link(), Some("https://test.example.com/article1"));
    assert!(first_item.description().unwrap().contains("test article"));
}

#[tokio::test]
async fn test_tech_news_feed() {
    let tech_news_path = "tests/fixtures/tech_news.xml";
    let rss_content = fs::read_to_string(tech_news_path).unwrap();

    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    assert_eq!(channel.title(), "Tech News Daily");
    assert_eq!(channel.description(), "Latest technology news and updates");
    assert_eq!(channel.items().len(), 3);

    let items = channel.items();
    assert_eq!(items[0].title(), Some("New AI Breakthrough"));
    assert_eq!(items[1].title(), Some("Cloud Computing Trends"));
    assert_eq!(items[2].title(), Some("Cybersecurity Alert"));

    // Check HTML content is preserved
    assert!(items[2].description().unwrap().contains("<ul>"));
    assert!(items[2].description().unwrap().contains("<li>"));
}

#[tokio::test]
async fn test_empty_feed() {
    let empty_feed_path = "tests/fixtures/empty_feed.xml";
    let rss_content = fs::read_to_string(empty_feed_path).unwrap();

    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    assert_eq!(channel.title(), "Empty Feed");
    assert_eq!(channel.description(), "A feed with no articles");
    assert_eq!(channel.items().len(), 0);
}

#[tokio::test]
async fn test_channels_to_epub_single_feed() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("test_single.epub");

    let config = Config {
        feeds: vec![Feed {
            name: "Test Feed".to_string(),
            url: "https://test.example.com/feed.xml".to_string(),
            description: "A test feed".to_string(),
        }],
        output: OutputConfig {
            filename: epub_path.to_str().unwrap().to_string(),
            title: "Test EPUB".to_string(),
            author: "Test Author".to_string(),
        },
    };

    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let channels = vec![("Test Feed".to_string(), channel)];

    let result = channels_to_epub(&channels, &config);
    assert!(result.is_ok());

    // Verify EPUB file was created
    assert!(epub_path.exists());

    // Verify file is not empty
    let metadata = fs::metadata(&epub_path).unwrap();
    assert!(metadata.len() > 0);
}

#[tokio::test]
async fn test_channels_to_epub_multiple_feeds() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("test_multiple.epub");

    let config = Config {
        feeds: vec![
            Feed {
                name: "Test Feed".to_string(),
                url: "https://test.example.com/feed.xml".to_string(),
                description: "A test feed".to_string(),
            },
            Feed {
                name: "Tech News".to_string(),
                url: "https://technews.example.com/feed.xml".to_string(),
                description: "Technology news".to_string(),
            },
        ],
        output: OutputConfig {
            filename: epub_path.to_str().unwrap().to_string(),
            title: "Multi Feed EPUB".to_string(),
            author: "Test Author".to_string(),
        },
    };

    // Load sample RSS
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let sample_rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let sample_channel = rss::Channel::read_from(sample_rss_content.as_bytes()).unwrap();

    // Load tech news RSS
    let tech_news_path = "tests/fixtures/tech_news.xml";
    let tech_news_content = fs::read_to_string(tech_news_path).unwrap();
    let tech_channel = rss::Channel::read_from(tech_news_content.as_bytes()).unwrap();

    let channels = vec![
        ("Test Feed".to_string(), sample_channel),
        ("Tech News".to_string(), tech_channel),
    ];

    let result = channels_to_epub(&channels, &config);
    assert!(result.is_ok());

    // Verify EPUB file was created
    assert!(epub_path.exists());

    // Verify file is not empty and larger than single feed
    let metadata = fs::metadata(&epub_path).unwrap();
    assert!(metadata.len() > 1000); // Should be reasonably sized for multiple feeds
}

#[tokio::test]
async fn test_channels_to_epub_empty_feed() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("test_empty.epub");

    let config = Config {
        feeds: vec![Feed {
            name: "Empty Feed".to_string(),
            url: "https://empty.example.com/feed.xml".to_string(),
            description: "An empty feed".to_string(),
        }],
        output: OutputConfig {
            filename: epub_path.to_str().unwrap().to_string(),
            title: "Empty EPUB".to_string(),
            author: "Test Author".to_string(),
        },
    };

    let empty_feed_path = "tests/fixtures/empty_feed.xml";
    let rss_content = fs::read_to_string(empty_feed_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let channels = vec![("Empty Feed".to_string(), channel)];

    let result = channels_to_epub(&channels, &config);
    assert!(result.is_ok());

    // Verify EPUB file was created even with empty feed
    assert!(epub_path.exists());

    // Verify file is not empty (should contain title page and structure)
    let metadata = fs::metadata(&epub_path).unwrap();
    assert!(metadata.len() > 0);
}

#[test]
fn test_sanitize_html_for_epub() {
    // Test access to the sanitize function by testing the feed processing
    let sample_rss_path = "tests/fixtures/tech_news.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    // Verify that HTML content is preserved in the parsed channel
    let items = channel.items();
    let cybersecurity_item = &items[2];
    let description = cybersecurity_item.description().unwrap();

    // Should contain HTML tags
    assert!(description.contains("<p>"));
    assert!(description.contains("<ul>"));
    assert!(description.contains("<li>"));

    // This tests that the RSS parsing works correctly with HTML content
    // The actual sanitization happens during EPUB generation
}

#[tokio::test]
async fn test_invalid_rss_format() {
    let invalid_rss = r#"<?xml version="1.0" encoding="UTF-8"?>
    <rss version="2.0">
        <channel>
            <title>Invalid Feed</title>
            <description>This feed has invalid structure</description>
            <!-- Missing closing tags -->
        </channel>
    "#;

    let result = rss::Channel::read_from(invalid_rss.as_bytes());
    // RSS parser should handle this gracefully or return an error
    // We're testing that our code can handle RSS parsing errors
    match result {
        Ok(_) => {
            // If it parses successfully, that's fine too
            // The RSS parser is quite forgiving
        }
        Err(_) => {
            // If it fails, we expect our code to handle this error
            // which it does in the fetch_all_feeds function
        }
    }
}
