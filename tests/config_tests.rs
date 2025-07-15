use daily_feed::config::{Config, Feed, OutputConfig};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_config_deserialization() {
    let json_content = r#"
    {
        "feeds": [
            {
                "name": "Test Feed",
                "url": "https://test.example.com/feed.xml",
                "description": "A test feed"
            }
        ],
        "output": {
            "filename": "test_output.epub",
            "title": "Test Title",
            "author": "Test Author"
        }
    }
    "#;

    let config: Config = serde_json::from_str(json_content).unwrap();

    assert_eq!(config.feeds.len(), 1);
    assert_eq!(config.feeds[0].name, "Test Feed");
    assert_eq!(config.feeds[0].url, "https://test.example.com/feed.xml");
    assert_eq!(config.feeds[0].description, "A test feed");
    assert_eq!(config.output.filename, "test_output.epub");
    assert_eq!(config.output.title, "Test Title");
    assert_eq!(config.output.author, "Test Author");
}

#[test]
fn test_config_load_from_file() {
    let json_content = r#"
    {
        "feeds": [
            {
                "name": "File Test Feed",
                "url": "https://file.example.com/feed.xml",
                "description": "A test feed from file"
            }
        ],
        "output": {
            "filename": "file_test.epub",
            "title": "File Test Title",
            "author": "File Test Author"
        }
    }
    "#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), json_content).unwrap();

    let config = Config::load_from_file(temp_file.path().to_str().unwrap()).unwrap();

    assert_eq!(config.feeds.len(), 1);
    assert_eq!(config.feeds[0].name, "File Test Feed");
    assert_eq!(config.output.filename, "file_test.epub");
}

#[test]
fn test_config_load_from_nonexistent_file() {
    let result = Config::load_from_file("nonexistent.json");
    assert!(result.is_err());
}

#[test]
fn test_config_load_invalid_json() {
    let invalid_json = r#"
    {
        "feeds": [
            {
                "name": "Test Feed",
                "url": "https://test.example.com/feed.xml"
                // Missing description and closing brace
    "#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), invalid_json).unwrap();

    let result = Config::load_from_file(temp_file.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_config_default() {
    let config = Config::default();

    assert_eq!(config.feeds.len(), 1);
    assert_eq!(config.feeds[0].name, "Ars Technica");
    assert!(config.feeds[0].url.contains("arstechnica.com"));
    assert_eq!(config.output.filename, "daily-feed.epub");
    assert_eq!(config.output.title, "Daily Feed Digest");
    assert_eq!(config.output.author, "RSS Aggregator");
}

#[test]
fn test_config_serialization() {
    let config = Config {
        feeds: vec![Feed {
            name: "Serialize Test".to_string(),
            url: "https://serialize.example.com/feed.xml".to_string(),
            description: "Test serialization".to_string(),
        }],
        output: OutputConfig {
            filename: "serialize_test.epub".to_string(),
            title: "Serialize Test Title".to_string(),
            author: "Serialize Test Author".to_string(),
        },
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.feeds.len(), 1);
    assert_eq!(deserialized.feeds[0].name, "Serialize Test");
    assert_eq!(deserialized.output.filename, "serialize_test.epub");
}

#[test]
fn test_config_multiple_feeds() {
    let json_content = r#"
    {
        "feeds": [
            {
                "name": "Feed 1",
                "url": "https://feed1.example.com/feed.xml",
                "description": "First feed"
            },
            {
                "name": "Feed 2",
                "url": "https://feed2.example.com/feed.xml",
                "description": "Second feed"
            },
            {
                "name": "Feed 3",
                "url": "https://feed3.example.com/feed.xml",
                "description": "Third feed"
            }
        ],
        "output": {
            "filename": "multi_feed_test.epub",
            "title": "Multi Feed Test",
            "author": "Multi Feed Author"
        }
    }
    "#;

    let config: Config = serde_json::from_str(json_content).unwrap();

    assert_eq!(config.feeds.len(), 3);
    assert_eq!(config.feeds[0].name, "Feed 1");
    assert_eq!(config.feeds[1].name, "Feed 2");
    assert_eq!(config.feeds[2].name, "Feed 3");
}
