use daily_feed::config::{Config, Feed, OutputConfig};
use daily_feed::fetch::{channels_to_document, document_to_epub};
use std::fs;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_cli_with_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.json");
    let epub_path = temp_dir.path().join("test_output.epub");

    let config = Config {
        feeds: vec![Feed {
            name: "Test Feed".to_string(),
            url: "https://test.example.com/feed.xml".to_string(),
            description: "A test feed".to_string(),
        }],
        output: OutputConfig {
            filename: epub_path.to_str().unwrap().to_string(),
            title: "Integration Test EPUB".to_string(),
            author: "Integration Test".to_string(),
        },
    };

    let config_json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&config_path, config_json).unwrap();

    // Test that the config file can be loaded properly
    let loaded_config = Config::load_from_file(config_path.to_str().unwrap()).unwrap();
    assert_eq!(loaded_config.feeds.len(), 1);
    assert_eq!(loaded_config.feeds[0].name, "Test Feed");
    assert_eq!(loaded_config.output.title, "Integration Test EPUB");
}

#[test]
fn test_cli_with_default_config() {
    let default_config = Config::default();

    // Test that default config has expected values
    assert_eq!(default_config.feeds.len(), 1);
    assert_eq!(default_config.feeds[0].name, "Ars Technica");
    assert_eq!(default_config.output.filename, "daily-feed.epub");
    assert_eq!(default_config.output.title, "Daily Feed Digest");
    assert_eq!(default_config.output.author, "RSS Aggregator");
}

#[test]
fn test_cli_with_invalid_config() {
    let temp_file = NamedTempFile::new().unwrap();
    let invalid_json = r#"{"invalid": "json structure"}"#;
    fs::write(temp_file.path(), invalid_json).unwrap();

    let result = Config::load_from_file(temp_file.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_cli_with_missing_config() {
    let result = Config::load_from_file("nonexistent_config.json");
    assert!(result.is_err());
}

#[test]
fn test_full_workflow_with_local_feeds() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("workflow_test.epub");

    let config = Config {
        feeds: vec![
            Feed {
                name: "Sample Feed".to_string(),
                url: "https://sample.example.com/feed.xml".to_string(),
                description: "Sample RSS feed".to_string(),
            },
            Feed {
                name: "Tech News".to_string(),
                url: "https://technews.example.com/feed.xml".to_string(),
                description: "Technology news".to_string(),
            },
        ],
        output: OutputConfig {
            filename: epub_path.to_str().unwrap().to_string(),
            title: "Workflow Test EPUB".to_string(),
            author: "Workflow Test".to_string(),
        },
    };

    // Load test RSS feeds
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

    // Test the full workflow: load feeds -> generate EPUB
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let document = channels_to_document(&channels, config.output.title.clone(), config.output.author.clone()).await.unwrap();
        document_to_epub(&document, &config.output.filename).await
    });
    assert!(result.is_ok());

    // Verify EPUB was created
    assert!(epub_path.exists());

    // Verify EPUB has reasonable size (contains content)
    let metadata = fs::metadata(&epub_path).unwrap();
    assert!(metadata.len() > 2000); // Should be at least 2KB for multiple feeds
}

#[test]
fn test_workflow_with_empty_feeds() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("empty_workflow_test.epub");

    let config = Config {
        feeds: vec![Feed {
            name: "Empty Feed".to_string(),
            url: "https://empty.example.com/feed.xml".to_string(),
            description: "Empty RSS feed".to_string(),
        }],
        output: OutputConfig {
            filename: epub_path.to_str().unwrap().to_string(),
            title: "Empty Workflow Test EPUB".to_string(),
            author: "Empty Workflow Test".to_string(),
        },
    };

    // Load empty RSS feed
    let empty_feed_path = "tests/fixtures/empty_feed.xml";
    let empty_rss_content = fs::read_to_string(empty_feed_path).unwrap();
    let empty_channel = rss::Channel::read_from(empty_rss_content.as_bytes()).unwrap();

    let channels = vec![("Empty Feed".to_string(), empty_channel)];

    // Test workflow with empty feed
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let document = channels_to_document(&channels, config.output.title.clone(), config.output.author.clone()).await.unwrap();
        document_to_epub(&document, &config.output.filename).await
    });
    assert!(result.is_ok());

    // Verify EPUB was created even with empty feed
    assert!(epub_path.exists());

    // Verify EPUB has some content (at least the structure)
    let metadata = fs::metadata(&epub_path).unwrap();
    assert!(metadata.len() > 0);
}

#[test]
fn test_workflow_with_no_feeds() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("no_feeds_test.epub");

    let config = Config {
        feeds: vec![],
        output: OutputConfig {
            filename: epub_path.to_str().unwrap().to_string(),
            title: "No Feeds Test EPUB".to_string(),
            author: "No Feeds Test".to_string(),
        },
    };

    let channels = vec![];

    // Test workflow with no feeds
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let document = channels_to_document(&channels, config.output.title.clone(), config.output.author.clone()).await.unwrap();
        document_to_epub(&document, &config.output.filename).await
    });
    assert!(result.is_ok());

    // Verify EPUB was created
    assert!(epub_path.exists());

    // Verify EPUB has minimal content
    let metadata = fs::metadata(&epub_path).unwrap();
    assert!(metadata.len() > 0);
}

#[test]
fn test_config_serialization_roundtrip() {
    let original_config = Config {
        feeds: vec![
            Feed {
                name: "Feed 1".to_string(),
                url: "https://feed1.example.com/rss.xml".to_string(),
                description: "First feed description".to_string(),
            },
            Feed {
                name: "Feed 2".to_string(),
                url: "https://feed2.example.com/rss.xml".to_string(),
                description: "Second feed description".to_string(),
            },
        ],
        output: OutputConfig {
            filename: "roundtrip_test.epub".to_string(),
            title: "Roundtrip Test".to_string(),
            author: "Test Author".to_string(),
        },
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&original_config).unwrap();

    // Deserialize back
    let deserialized_config: Config = serde_json::from_str(&json).unwrap();

    // Verify they match
    assert_eq!(original_config.feeds.len(), deserialized_config.feeds.len());
    assert_eq!(
        original_config.feeds[0].name,
        deserialized_config.feeds[0].name
    );
    assert_eq!(
        original_config.feeds[0].url,
        deserialized_config.feeds[0].url
    );
    assert_eq!(
        original_config.feeds[0].description,
        deserialized_config.feeds[0].description
    );
    assert_eq!(
        original_config.feeds[1].name,
        deserialized_config.feeds[1].name
    );
    assert_eq!(
        original_config.output.filename,
        deserialized_config.output.filename
    );
    assert_eq!(
        original_config.output.title,
        deserialized_config.output.title
    );
    assert_eq!(
        original_config.output.author,
        deserialized_config.output.author
    );
}

#[test]
fn test_rss_parsing_with_different_encodings() {
    // Test with UTF-8 content
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    assert_eq!(channel.title(), "Test Feed");
    assert_eq!(channel.items().len(), 2);

    // Test that HTML entities are preserved in the RSS content
    let first_item = &channel.items()[0];
    let description = first_item.description().unwrap();
    assert!(description.contains("&amp;"));
}

#[test]
fn test_epub_generation_with_html_content() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("html_content_test.epub");

    let config = Config {
        feeds: vec![Feed {
            name: "HTML Content Feed".to_string(),
            url: "https://html.example.com/feed.xml".to_string(),
            description: "Feed with HTML content".to_string(),
        }],
        output: OutputConfig {
            filename: epub_path.to_str().unwrap().to_string(),
            title: "HTML Content Test".to_string(),
            author: "HTML Test".to_string(),
        },
    };

    // Load tech news feed which has HTML content
    let tech_news_path = "tests/fixtures/tech_news.xml";
    let tech_news_content = fs::read_to_string(tech_news_path).unwrap();
    let tech_channel = rss::Channel::read_from(tech_news_content.as_bytes()).unwrap();

    let channels = vec![("HTML Content Feed".to_string(), tech_channel)];

    // Test EPUB generation with HTML content
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let document = channels_to_document(&channels, config.output.title.clone(), config.output.author.clone()).await.unwrap();
        document_to_epub(&document, &config.output.filename).await
    });
    assert!(result.is_ok());

    // Verify EPUB was created
    assert!(epub_path.exists());

    // Verify EPUB has content (HTML content should be processed)
    let metadata = fs::metadata(&epub_path).unwrap();
    assert!(metadata.len() > 1000);
}
