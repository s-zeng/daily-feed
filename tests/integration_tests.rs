use daily_feed::config::{Config, Feed, OutputConfig, OutputFormat};
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
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

    let config_json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&config_path, config_json).unwrap();

    let loaded_config = Config::load_from_file(config_path.to_str().unwrap()).unwrap();

    // Create deterministic snapshot by removing temp path
    let config_summary = format!(
        "feeds: {}, title: {}, author: {}, format: {:?}",
        loaded_config.feeds.len(),
        loaded_config.output.title,
        loaded_config.output.author,
        loaded_config.output.format
    );
    insta::assert_snapshot!(config_summary);
}

#[test]
fn test_cli_with_default_config() {
    let default_config = Config::default();
    insta::assert_json_snapshot!(default_config);
}

#[test]
fn test_cli_with_invalid_config() {
    let temp_file = NamedTempFile::new().unwrap();
    let invalid_json = r#"{"invalid": "json structure"}"#;
    fs::write(temp_file.path(), invalid_json).unwrap();

    let result = Config::load_from_file(temp_file.path().to_str().unwrap());
    let error_message = result.unwrap_err().to_string();
    insta::assert_snapshot!(error_message);
}

#[test]
fn test_cli_with_missing_config() {
    let result = Config::load_from_file("nonexistent_config.json");
    let error_message = result.unwrap_err().to_string();
    insta::assert_snapshot!(error_message);
}

#[test]
fn test_full_workflow_with_local_feeds_success() {
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
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

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

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let document = channels_to_document(
            &channels,
            config.output.title.clone(),
            config.output.author.clone(),
        )
        .await
        .unwrap();
        document_to_epub(&document, &config.output.filename).await
    });

    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_full_workflow_creates_epub_file() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("workflow_test.epub");

    let config = Config {
        feeds: vec![Feed {
            name: "Sample Feed".to_string(),
            url: "https://sample.example.com/feed.xml".to_string(),
            description: "Sample RSS feed".to_string(),
        }],
        output: OutputConfig {
            filename: epub_path.to_str().unwrap().to_string(),
            title: "Workflow Test EPUB".to_string(),
            author: "Workflow Test".to_string(),
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let sample_rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let sample_channel = rss::Channel::read_from(sample_rss_content.as_bytes()).unwrap();

    let channels = vec![("Sample Feed".to_string(), sample_channel)];

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let document = channels_to_document(
            &channels,
            config.output.title.clone(),
            config.output.author.clone(),
        )
        .await
        .unwrap();
        document_to_epub(&document, &config.output.filename)
            .await
            .unwrap();
    });

    let file_exists = epub_path.exists();
    insta::assert_snapshot!(file_exists.to_string());
}

#[test]
fn test_full_workflow_epub_size() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("workflow_test.epub");

    let config = Config {
        feeds: vec![Feed {
            name: "Sample Feed".to_string(),
            url: "https://sample.example.com/feed.xml".to_string(),
            description: "Sample RSS feed".to_string(),
        }],
        output: OutputConfig {
            filename: epub_path.to_str().unwrap().to_string(),
            title: "Workflow Test EPUB".to_string(),
            author: "Workflow Test".to_string(),
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let sample_rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let sample_channel = rss::Channel::read_from(sample_rss_content.as_bytes()).unwrap();

    let channels = vec![("Sample Feed".to_string(), sample_channel)];

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let document = channels_to_document(
            &channels,
            config.output.title.clone(),
            config.output.author.clone(),
        )
        .await
        .unwrap();
        document_to_epub(&document, &config.output.filename)
            .await
            .unwrap();
    });

    let file_size_valid = if epub_path.exists() {
        let size = fs::metadata(&epub_path).unwrap().len();
        size > 3000 && size < 10000
    } else {
        false
    };
    insta::assert_snapshot!(format!(
        "file_exists: {}, file_size_valid: {}",
        epub_path.exists(),
        file_size_valid
    ));
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
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

    let empty_feed_path = "tests/fixtures/empty_feed.xml";
    let empty_rss_content = fs::read_to_string(empty_feed_path).unwrap();
    let empty_channel = rss::Channel::read_from(empty_rss_content.as_bytes()).unwrap();

    let channels = vec![("Empty Feed".to_string(), empty_channel)];

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let document = channels_to_document(
            &channels,
            config.output.title.clone(),
            config.output.author.clone(),
        )
        .await
        .unwrap();
        document_to_epub(&document, &config.output.filename).await
    });

    let file_size_valid = if epub_path.exists() {
        let size = fs::metadata(&epub_path).unwrap().len();
        size > 1000 && size < 10000
    } else {
        false
    };

    insta::assert_snapshot!(format!(
        "result_ok: {}, file_exists: {}, file_size_valid: {}",
        result.is_ok(),
        epub_path.exists(),
        file_size_valid
    ));
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
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

    let channels = vec![];

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let document = channels_to_document(
            &channels,
            config.output.title.clone(),
            config.output.author.clone(),
        )
        .await
        .unwrap();
        document_to_epub(&document, &config.output.filename).await
    });

    let file_size_valid = if epub_path.exists() {
        let size = fs::metadata(&epub_path).unwrap().len();
        size > 1000 && size < 10000
    } else {
        false
    };

    insta::assert_snapshot!(format!(
        "result_ok: {}, file_exists: {}, file_size_valid: {}",
        result.is_ok(),
        epub_path.exists(),
        file_size_valid
    ));
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
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

    let json = serde_json::to_string_pretty(&original_config).unwrap();
    let deserialized_config: Config = serde_json::from_str(&json).unwrap();

    insta::assert_json_snapshot!(deserialized_config);
}

#[test]
fn test_rss_parsing_channel_title() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let title = channel.title();
    insta::assert_snapshot!(title);
}

#[test]
fn test_rss_parsing_items_count() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let items_count = channel.items().len();
    insta::assert_snapshot!(items_count.to_string());
}

#[test]
fn test_rss_parsing_html_entities() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let first_item = &channel.items()[0];
    let description = first_item.description().unwrap();
    let contains_amp = description.contains("&amp;");
    insta::assert_snapshot!(contains_amp.to_string());
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
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

    let tech_news_path = "tests/fixtures/tech_news.xml";
    let tech_news_content = fs::read_to_string(tech_news_path).unwrap();
    let tech_channel = rss::Channel::read_from(tech_news_content.as_bytes()).unwrap();

    let channels = vec![("HTML Content Feed".to_string(), tech_channel)];

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let document = channels_to_document(
            &channels,
            config.output.title.clone(),
            config.output.author.clone(),
        )
        .await
        .unwrap();
        document_to_epub(&document, &config.output.filename).await
    });

    let file_size_valid = if epub_path.exists() {
        let size = fs::metadata(&epub_path).unwrap().len();
        size > 3000 && size < 10000
    } else {
        false
    };

    insta::assert_snapshot!(format!(
        "result_ok: {}, file_exists: {}, file_size_valid: {}",
        result.is_ok(),
        epub_path.exists(),
        file_size_valid
    ));
}
