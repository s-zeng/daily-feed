use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Feed {
    pub name: String,
    pub url: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputConfig {
    pub filename: String,
    pub title: String,
    pub author: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub feeds: Vec<Feed>,
    pub output: OutputConfig,
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn default() -> Self {
        Config {
            feeds: vec![Feed {
                name: "Ars Technica".to_string(),
                url: "https://arstechnica.com/feed/?t=bd10cf5f81b5d0a5edbb2faa32e6d55c7a1efdb3"
                    .to_string(),
                description: "Technology news and insights".to_string(),
            }],
            output: OutputConfig {
                filename: "daily-feed.epub".to_string(),
                title: "Daily Feed Digest".to_string(),
                author: "RSS Aggregator".to_string(),
            },
        }
    }
}
