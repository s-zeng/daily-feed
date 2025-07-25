use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use crate::ai_client::AiProvider;

#[derive(Debug, Deserialize, Serialize)]
pub struct Feed {
    pub name: String,
    pub url: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OutputFormat {
    #[serde(rename = "epub")]
    Epub,
    #[serde(rename = "markdown")]
    Markdown,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Epub
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FrontPageConfig {
    pub enabled: bool,
    pub provider: AiProviderConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum AiProviderConfig {
    #[serde(rename = "ollama")]
    Ollama {
        base_url: String,
        model: String,
    },
    #[serde(rename = "anthropic")]
    Anthropic {
        api_key: String,
        model: String,
    },
}

impl From<AiProviderConfig> for AiProvider {
    fn from(config: AiProviderConfig) -> Self {
        match config {
            AiProviderConfig::Ollama { base_url, model } => AiProvider::Ollama { base_url, model },
            AiProviderConfig::Anthropic { api_key, model } => AiProvider::Anthropic { api_key, model },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputConfig {
    pub filename: String,
    pub title: String,
    pub author: String,
    #[serde(default)]
    pub format: OutputFormat,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub feeds: Vec<Feed>,
    pub output: OutputConfig,
    pub front_page: Option<FrontPageConfig>,
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
                format: OutputFormat::default(),
            },
            front_page: None,
        }
    }
}
