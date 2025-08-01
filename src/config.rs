use crate::ai_client::AiProvider;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Feed {
    #[serde(rename = "generic")]
    Generic {
        name: String,
        url: String,
        description: String,
    },
    #[serde(rename = "ars_technica")]
    ArsTechnica {
        #[serde(skip_serializing_if = "Option::is_none")]
        api_token: Option<String>,
    },
}

impl Feed {
    pub fn name(&self) -> &str {
        match self {
            Feed::Generic { name, .. } => name,
            Feed::ArsTechnica { .. } => "Ars Technica",
        }
    }

    pub fn url(&self) -> String {
        match self {
            Feed::Generic { url, .. } => url.clone(),
            Feed::ArsTechnica { api_token } => {
                if let Some(token) = api_token {
                    format!("https://arstechnica.com/feed/?t={}", token)
                } else {
                    "https://arstechnica.com/feed/".to_string()
                }
            }
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Feed::Generic { description, .. } => description,
            Feed::ArsTechnica { .. } => "Technology news and insights",
        }
    }

    pub fn api_token(&self) -> Option<&str> {
        match self {
            Feed::Generic { .. } => None,
            Feed::ArsTechnica { api_token } => api_token.as_deref(),
        }
    }
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
    Ollama { base_url: String, model: String },
    #[serde(rename = "anthropic")]
    Anthropic { api_key: String, model: String },
}

impl From<AiProviderConfig> for AiProvider {
    fn from(config: AiProviderConfig) -> Self {
        match config {
            AiProviderConfig::Ollama { base_url, model } => AiProvider::Ollama { base_url, model },
            AiProviderConfig::Anthropic { api_key, model } => {
                AiProvider::Anthropic { api_key, model }
            }
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
            feeds: vec![Feed::ArsTechnica { api_token: None }],
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
