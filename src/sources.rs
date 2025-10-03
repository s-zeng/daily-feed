use crate::ars_comments;
use crate::ast::{Comment, Document};
use crate::http_utils::create_http_client;
use crate::parser::{parse_feeds_to_document, parse_html_to_content_blocks};
use async_trait::async_trait;
use std::error::Error;
use std::collections::HashMap;

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
    #[serde(rename = "hackernews")]
    HackerNews,
}

impl SourceConfig {
    pub fn name(&self) -> &str {
        match self {
            SourceConfig::Rss { .. } => "RSS Feed",
            SourceConfig::ArsTechnica { .. } => "Ars Technica",
            SourceConfig::HackerNews => "Hacker News",
        }
    }
}

#[derive(Debug)]
pub struct RssSource {
    url: String,
    #[allow(dead_code)]
    description: String,
}

impl RssSource {
    pub fn new(url: String, description: String) -> Self {
        Self { url, description }
    }

    async fn fetch_rss_channel(&self) -> Result<rss::Channel, Box<dyn Error>> {
        let client = create_http_client()?;
        let response = client
            .get(&self.url)
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
        
        parse_feeds_to_document(&channels, title, author).await
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
                            for raw_comment in raw_comments {
                                let comment_content = parse_html_to_content_blocks(&raw_comment.content)?;
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

#[derive(Debug)]
pub struct HackerNewsSource;

#[derive(Debug, serde::Deserialize)]
struct JsonFeedItem {
    #[allow(dead_code)]
    id: String,
    title: String,
    content_html: String,
    url: String,
    date_published: String,
    author: JsonFeedAuthor,
}

#[derive(Debug, serde::Deserialize)]
struct JsonFeedAuthor {
    name: String,
}

#[derive(Debug, serde::Deserialize)]
struct JsonFeed {
    items: Vec<JsonFeedItem>,
}

impl HackerNewsSource {
    pub fn new() -> Self {
        Self
    }

    async fn fetch_json_feed(&self) -> Result<JsonFeed, Box<dyn Error>> {
        let client = create_http_client()?;
        let response = client
            .get("https://hnrss.org/bestcomments.jsonfeed")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let json_feed: JsonFeed = response.json().await?;
        Ok(json_feed)
    }

    fn extract_parent_title(&self, title: &str) -> String {
        // Title format: "New comment by username in \"Article Title\""
        if let Some(start) = title.find(" in \"") {
            if let Some(end) = title.rfind('"') {
                if end > start + 5 {
                    return title[start + 5..end].to_string();
                }
            }
        }
        // Fallback: return the whole title if parsing fails
        title.to_string()
    }
}

#[async_trait(?Send)]
impl Source for HackerNewsSource {
    async fn fetch_document(
        &self,
        name: String,
        title: String,
        author: String,
    ) -> Result<Document, Box<dyn Error>> {
        let json_feed = self.fetch_json_feed().await?;
        
        // Group comments by parent article title
        let mut articles_map: HashMap<String, Vec<JsonFeedItem>> = HashMap::new();
        
        for item in json_feed.items {
            let parent_title = self.extract_parent_title(&item.title);
            articles_map.entry(parent_title).or_insert_with(Vec::new).push(item);
        }
        
        // Convert to articles with comments
        let mut articles = Vec::new();
        for (parent_title, comment_items) in articles_map {
            let mut comments = Vec::new();
            let mut article_url: Option<String> = None;
            
            // Parse each comment and extract the first URL as article URL
            for item in comment_items {
                if article_url.is_none() {
                    // Extract article URL from comment URL (remove the comment ID part)
                    if let Some(base_url) = item.url.split("#").next() {
                        article_url = Some(base_url.to_string());
                    }
                }
                
                let comment_content = parse_html_to_content_blocks(&item.content_html)?;
                let comment = Comment {
                    author: item.author.name,
                    content: comment_content,
                    upvotes: 0, // HN comments don't have scores in this feed
                    downvotes: 0,
                    timestamp: Some(item.date_published),
                };
                comments.push(comment);
            }
            
            // Create article with empty content (just the headline and comments)
            let article = crate::ast::Article {
                title: parent_title.clone(),
                content: vec![], // No article content, just headlines
                metadata: crate::ast::ArticleMetadata {
                    published_date: comments.first().map(|c| c.timestamp.clone()).flatten(),
                    author: None,
                    url: article_url,
                    feed_name: name.clone(),
                },
                comments,
                reading_time_minutes: None,
            };
            articles.push(article);
        }
        
        let feed = crate::ast::Feed {
            name: name.clone(),
            description: Some("Hacker News best comments and parent articles".to_string()),
            url: Some("https://hnrss.org/bestcomments.jsonfeed".to_string()),
            articles,
            total_reading_time_minutes: None,
        };
        
        let document = Document {
            metadata: crate::ast::DocumentMetadata {
                title,
                author,
                description: Some("Hacker News digest with best comments".to_string()),
                generated_at: chrono::Utc::now().to_rfc3339(),
            },
            front_page: None,
            feeds: vec![feed],
            total_reading_time_minutes: None,
        };
        
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
            SourceConfig::HackerNews => {
                Box::new(HackerNewsSource::new())
            }
        }
    }
}