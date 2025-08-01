use daily_feed::config::{Config, Feed, OutputConfig, OutputFormat};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_config_deserialization() {
    let json_content = r#"
    {
        "feeds": [
            {
                "type": "generic",
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
    insta::assert_json_snapshot!(config);
}

#[test]
fn test_config_load_from_file() {
    let json_content = r#"
    {
        "feeds": [
            {
                "type": "generic",
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
    insta::assert_json_snapshot!(config);
}

#[test]
fn test_config_load_from_nonexistent_file() {
    let result = Config::load_from_file("nonexistent.json");
    let error_message = result.unwrap_err().to_string();
    insta::assert_snapshot!(error_message);
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
    let error_message = result.unwrap_err().to_string();
    insta::assert_snapshot!(error_message);
}

#[test]
fn test_config_default() {
    let config = Config::default();
    insta::assert_json_snapshot!(config);
}

#[test]
fn test_config_serialization() {
    let config = Config {
        feeds: vec![Feed::Generic {
            name: "Serialize Test".to_string(),
            url: "https://serialize.example.com/feed.xml".to_string(),
            description: "Test serialization".to_string(),
        }],
        output: OutputConfig {
            filename: "serialize_test.epub".to_string(),
            title: "Serialize Test Title".to_string(),
            author: "Serialize Test Author".to_string(),
            format: OutputFormat::Epub,
        },
        front_page: None,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();
    insta::assert_json_snapshot!(deserialized);
}

#[test]
fn test_config_multiple_feeds() {
    let json_content = r#"
    {
        "feeds": [
            {
                "type": "generic",
                "name": "Feed 1",
                "url": "https://feed1.example.com/feed.xml",
                "description": "First feed"
            },
            {
                "type": "generic",
                "name": "Feed 2",
                "url": "https://feed2.example.com/feed.xml",
                "description": "Second feed"
            },
            {
                "type": "ars_technica"
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
    insta::assert_json_snapshot!(config);
}
