use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub metadata: DocumentMetadata,
    pub feeds: Vec<Feed>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentMetadata {
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Feed {
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub articles: Vec<Article>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Article {
    pub title: String,
    pub content: Vec<ContentBlock>,
    pub metadata: ArticleMetadata,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArticleMetadata {
    pub published_date: Option<String>,
    pub author: Option<String>,
    pub url: Option<String>,
    pub feed_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Comment {
    pub author: String,
    pub content: Vec<ContentBlock>,
    pub score: i32,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentBlock {
    Paragraph(TextContent),
    Heading { level: u8, content: TextContent },
    List { ordered: bool, items: Vec<TextContent> },
    Quote(TextContent),
    Code { language: Option<String>, content: String },
    Link { url: String, text: String },
    Image { url: String, alt: Option<String> },
    Raw(String), // For complex HTML that we want to preserve as-is
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextContent {
    pub spans: Vec<TextSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextSpan {
    pub text: String,
    pub formatting: TextFormatting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TextFormatting {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub link: Option<String>,
}

impl Document {
    pub fn new(title: String, author: String) -> Self {
        Self {
            metadata: DocumentMetadata {
                title,
                author,
                description: None,
                generated_at: chrono::Utc::now().to_rfc3339(),
            },
            feeds: Vec::new(),
        }
    }

    pub fn add_feed(&mut self, feed: Feed) {
        self.feeds.push(feed);
    }

    pub fn total_articles(&self) -> usize {
        self.feeds.iter().map(|f| f.articles.len()).sum()
    }
}

impl Feed {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            url: None,
            articles: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn add_article(&mut self, article: Article) {
        self.articles.push(article);
    }
}

impl Article {
    pub fn new(title: String, feed_name: String) -> Self {
        Self {
            title,
            content: Vec::new(),
            metadata: ArticleMetadata {
                published_date: None,
                author: None,
                url: None,
                feed_name,
            },
            comments: Vec::new(),
        }
    }

    pub fn with_content(mut self, content: Vec<ContentBlock>) -> Self {
        self.content = content;
        self
    }

    pub fn with_published_date(mut self, date: String) -> Self {
        self.metadata.published_date = Some(date);
        self
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.metadata.url = Some(url);
        self
    }

    pub fn add_comment(&mut self, comment: Comment) {
        self.comments.push(comment);
    }
}

impl TextContent {
    pub fn plain(text: String) -> Self {
        Self {
            spans: vec![TextSpan {
                text,
                formatting: TextFormatting::default(),
            }],
        }
    }

    pub fn from_spans(spans: Vec<TextSpan>) -> Self {
        Self { spans }
    }

    pub fn is_empty(&self) -> bool {
        self.spans.iter().all(|span| span.text.trim().is_empty())
    }

    pub fn to_plain_text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }
}

impl TextSpan {
    pub fn plain(text: String) -> Self {
        Self {
            text,
            formatting: TextFormatting::default(),
        }
    }

    pub fn bold(text: String) -> Self {
        Self {
            text,
            formatting: TextFormatting {
                bold: true,
                ..Default::default()
            },
        }
    }

    pub fn italic(text: String) -> Self {
        Self {
            text,
            formatting: TextFormatting {
                italic: true,
                ..Default::default()
            },
        }
    }

    pub fn code(text: String) -> Self {
        Self {
            text,
            formatting: TextFormatting {
                code: true,
                ..Default::default()
            },
        }
    }

    pub fn link(text: String, url: String) -> Self {
        Self {
            text,
            formatting: TextFormatting {
                link: Some(url),
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let mut doc = Document::new("Test Document".to_string(), "Test Author".to_string());
        
        let mut feed = Feed::new("Test Feed".to_string())
            .with_description("A test feed".to_string());
        
        let article = Article::new("Test Article".to_string(), "Test Feed".to_string())
            .with_content(vec![
                ContentBlock::Paragraph(TextContent::plain("Test content".to_string()))
            ]);
        
        feed.add_article(article);
        doc.add_feed(feed);
        
        assert_eq!(doc.feeds.len(), 1);
        assert_eq!(doc.total_articles(), 1);
        assert_eq!(doc.feeds[0].articles[0].title, "Test Article");
    }

    #[test]
    fn test_text_content() {
        let content = TextContent::from_spans(vec![
            TextSpan::plain("Hello ".to_string()),
            TextSpan::bold("world".to_string()),
            TextSpan::plain("!".to_string()),
        ]);
        
        assert_eq!(content.to_plain_text(), "Hello world!");
        assert!(!content.is_empty());
    }
}