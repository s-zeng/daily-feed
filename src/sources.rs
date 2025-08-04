use crate::ars_comments;
use crate::ast::{Comment, Document};
use crate::parser::DocumentParser;
use async_trait::async_trait;
use std::error::Error;

#[async_trait(?Send)]
pub trait Source {
    async fn fetch_document(
        &self,
        name: String,
        title: String,
        author: String,
    ) -> Result<Document, Box<dyn Error>>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum SourceConfig {
    #[serde(rename = "rss")]
    Rss { url: String, description: String },
    #[serde(rename = "ars_technica")]
    ArsTechnica { 
        #[serde(skip_serializing_if = "Option::is_none")]
        api_token: Option<String> 
    },
}

impl SourceConfig {
    pub fn name(&self) -> &str {
        match self {
            SourceConfig::Rss { .. } => "RSS Feed",
            SourceConfig::ArsTechnica { .. } => "Ars Technica",
        }
    }
}

#[derive(Debug)]
pub struct RssSource {
    url: String,
    description: String,
}

impl RssSource {
    pub fn new(url: String, description: String) -> Self {
        Self { url, description }
    }

    async fn fetch_rss_channel(&self) -> Result<rss::Channel, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let response = client
            .get(&self.url)
            .header("User-Agent", "daily-feed/0.1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let content = response.bytes().await?;
        let channel = rss::Channel::read_from(&content[..])?;
        Ok(channel)
    }
}

#[async_trait(?Send)]
impl Source for RssSource {
    async fn fetch_document(
        &self,
        name: String,
        title: String,
        author: String,
    ) -> Result<Document, Box<dyn Error>> {
        let channel = self.fetch_rss_channel().await?;
        let channels = vec![(name, channel)];
        
        let parser = DocumentParser::new();
        parser.parse_feeds_to_document(&channels, title, author).await
    }
}

#[derive(Debug)]
pub struct ArsTechnicaSource {
    rss_source: RssSource,
}

impl ArsTechnicaSource {
    pub fn new(api_token: Option<String>) -> Self {
        let url = if let Some(token) = &api_token {
            format!("https://arstechnica.com/feed/?t={}", token)
        } else {
            "https://arstechnica.com/feed/".to_string()
        };
        
        Self {
            rss_source: RssSource::new(url, "Technology news and insights".to_string()),
        }
    }
}

#[async_trait(?Send)]
impl Source for ArsTechnicaSource {
    async fn fetch_document(
        &self,
        name: String,
        title: String,
        author: String,
    ) -> Result<Document, Box<dyn Error>> {
        // First get the base RSS document
        let mut document = self.rss_source.fetch_document(name, title, author).await?;
        
        // Then enhance each article with Ars Technica comments
        for feed in &mut document.feeds {
            for article in &mut feed.articles {
                if let Some(article_url) = &article.metadata.url {
                    match ars_comments::fetch_top_5_comments(article_url).await {
                        Ok(raw_comments) => {
                            let parser = DocumentParser::new();
                            for raw_comment in raw_comments {
                                let comment_content = parser
                                    .parse_html_to_content_blocks(&raw_comment.content)?;
                                let comment = Comment {
                                    author: raw_comment.author,
                                    content: comment_content,
                                    upvotes: raw_comment.upvotes,
                                    downvotes: raw_comment.downvotes,
                                    timestamp: raw_comment.timestamp,
                                };
                                article.comments.push(comment);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to fetch comments for {}: {}", article.title, e);
                        }
                    }
                }
            }
        }
        
        Ok(document)
    }
}

impl From<SourceConfig> for Box<dyn Source> {
    fn from(config: SourceConfig) -> Self {
        match config {
            SourceConfig::Rss { url, description } => {
                Box::new(RssSource::new(url, description))
            }
            SourceConfig::ArsTechnica { api_token } => {
                Box::new(ArsTechnicaSource::new(api_token))
            }
        }
    }
}