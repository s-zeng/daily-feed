use daily_feed::config::{Config, Feed, OutputConfig, OutputFormat};
use daily_feed::fetch::{channels_to_document, document_to_epub};
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_feed_from_file_title() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    insta::assert_snapshot!(channel.title());
}

#[tokio::test]
async fn test_feed_from_file_description() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    insta::assert_snapshot!(channel.description());
}

#[tokio::test]
async fn test_feed_from_file_link() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    insta::assert_snapshot!(channel.link());
}

#[tokio::test]
async fn test_feed_from_file_items_count() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    insta::assert_snapshot!(channel.items().len().to_string());
}

#[tokio::test]
async fn test_feed_from_file_first_item() {
    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let first_item = &channel.items()[0];
    let item_info = format!(
        "title: {:?}, link: {:?}, desc_contains_test: {}",
        first_item.title(),
        first_item.link(),
        first_item
            .description()
            .unwrap_or("")
            .contains("test article")
    );
    insta::assert_snapshot!(item_info);
}

#[tokio::test]
async fn test_tech_news_feed_metadata() {
    let tech_news_path = "tests/fixtures/tech_news.xml";
    let rss_content = fs::read_to_string(tech_news_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let metadata = format!(
        "title: {}, desc: {}, items: {}",
        channel.title(),
        channel.description(),
        channel.items().len()
    );
    insta::assert_snapshot!(metadata);
}

#[tokio::test]
async fn test_tech_news_feed_item_titles() {
    let tech_news_path = "tests/fixtures/tech_news.xml";
    let rss_content = fs::read_to_string(tech_news_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let items = channel.items();
    let titles: Vec<_> = items.iter().map(|i| i.title().unwrap_or("")).collect();
    insta::assert_json_snapshot!(titles);
}

#[tokio::test]
async fn test_tech_news_feed_html_preservation() {
    let tech_news_path = "tests/fixtures/tech_news.xml";
    let rss_content = fs::read_to_string(tech_news_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let items = channel.items();
    let description = items[2].description().unwrap_or("");
    let html_check = format!(
        "contains_ul: {}, contains_li: {}",
        description.contains("<ul>"),
        description.contains("<li>")
    );
    insta::assert_snapshot!(html_check);
}

#[tokio::test]
async fn test_empty_feed() {
    let empty_feed_path = "tests/fixtures/empty_feed.xml";
    let rss_content = fs::read_to_string(empty_feed_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let feed_info = format!(
        "title: {}, desc: {}, items: {}",
        channel.title(),
        channel.description(),
        channel.items().len()
    );
    insta::assert_snapshot!(feed_info);
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
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

    let sample_rss_path = "tests/fixtures/sample_rss.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let channels = vec![("Test Feed".to_string(), channel)];

    let document = channels_to_document(
        &channels,
        config.output.title.clone(),
        config.output.author.clone(),
    )
    .await
    .unwrap();
    let result = document_to_epub(&document, &config.output.filename).await;

    let file_exists = epub_path.exists();
    let file_size_valid = if file_exists {
        let size = fs::metadata(&epub_path).unwrap().len();
        size > 1000 && size < 10000 // Reasonable size range
    } else {
        false
    };

    insta::assert_snapshot!(format!(
        "result_ok: {}, file_exists: {}, file_size_valid: {}",
        result.is_ok(),
        file_exists,
        file_size_valid
    ));
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
        ("Test Feed".to_string(), sample_channel),
        ("Tech News".to_string(), tech_channel),
    ];

    let document = channels_to_document(
        &channels,
        config.output.title.clone(),
        config.output.author.clone(),
    )
    .await
    .unwrap();
    let result = document_to_epub(&document, &config.output.filename).await;

    let file_exists = epub_path.exists();
    let file_size_valid = if file_exists {
        let size = fs::metadata(&epub_path).unwrap().len();
        size > 1000 && size < 10000 // Reasonable size range
    } else {
        false
    };

    insta::assert_snapshot!(format!(
        "result_ok: {}, file_exists: {}, file_size_valid: {}",
        result.is_ok(),
        file_exists,
        file_size_valid
    ));
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
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

    let empty_feed_path = "tests/fixtures/empty_feed.xml";
    let rss_content = fs::read_to_string(empty_feed_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let channels = vec![("Empty Feed".to_string(), channel)];

    let document = channels_to_document(
        &channels,
        config.output.title.clone(),
        config.output.author.clone(),
    )
    .await
    .unwrap();
    let result = document_to_epub(&document, &config.output.filename).await;

    let file_exists = epub_path.exists();
    let file_size_valid = if file_exists {
        let size = fs::metadata(&epub_path).unwrap().len();
        size > 1000 && size < 10000 // Reasonable size range
    } else {
        false
    };

    insta::assert_snapshot!(format!(
        "result_ok: {}, file_exists: {}, file_size_valid: {}",
        result.is_ok(),
        file_exists,
        file_size_valid
    ));
}

#[test]
fn test_sanitize_html_for_epub() {
    let sample_rss_path = "tests/fixtures/tech_news.xml";
    let rss_content = fs::read_to_string(sample_rss_path).unwrap();
    let channel = rss::Channel::read_from(rss_content.as_bytes()).unwrap();

    let items = channel.items();
    let cybersecurity_item = &items[2];
    let description = cybersecurity_item.description().unwrap();

    let html_tags_present = format!(
        "has_p: {}, has_ul: {}, has_li: {}",
        description.contains("<p>"),
        description.contains("<ul>"),
        description.contains("<li>")
    );

    insta::assert_snapshot!(html_tags_present);
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
    let parse_result = match result {
        Ok(_) => "success_parsed_despite_invalid".to_string(),
        Err(e) => format!("error_{}", e.to_string().len()),
    };

    insta::assert_snapshot!(parse_result);
}
