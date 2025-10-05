use nonempty::NonEmpty;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArticleContent {
    pub blocks: NonEmpty<ContentBlock>,
    pub reading_time_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedContent {
    pub articles: NonEmpty<Article>,
    pub total_reading_time_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentContent {
    pub feeds: NonEmpty<Feed>,
    pub total_reading_time_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Headline {
    pub title: String,
    pub published_date: Option<String>,
    pub source_name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub metadata: DocumentMetadata,
    pub front_page: Option<Vec<ContentBlock>>,
    pub content: Option<DocumentContent>,
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
    pub content: Option<FeedContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Article {
    pub title: String,
    pub metadata: ArticleMetadata,
    pub comments: Vec<Comment>,
    pub content: Option<ArticleContent>,
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
    pub upvotes: u32,
    pub downvotes: u32,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentBlock {
    Paragraph(TextContent),
    Heading {
        level: u8,
        content: TextContent,
    },
    List {
        ordered: bool,
        items: Vec<TextContent>,
    },
    Quote(TextContent),
    Code {
        language: Option<String>,
        content: String,
    },
    Link {
        url: String,
        text: String,
    },
    Image {
        url: String,
        alt: Option<String>,
    },
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
            front_page: None,
            content: None,
        }
    }

    pub fn set_content(&mut self, feeds: NonEmpty<Feed>, total_reading_time_minutes: u32) {
        self.content = Some(DocumentContent {
            feeds,
            total_reading_time_minutes,
        });
    }

    pub fn set_front_page(&mut self, front_page: Vec<ContentBlock>) {
        self.front_page = Some(front_page);
    }

    pub fn total_articles(&self) -> usize {
        self.content
            .as_ref()
            .map(|c| c.feeds.iter().map(|f| f.content.as_ref().map_or(0, |fc| fc.articles.len())).sum())
            .unwrap_or(0)
    }

    pub fn extract_headlines(&self) -> Vec<Headline> {
        self.content
            .as_ref()
            .map(|c| {
                c.feeds
                    .iter()
                    .flat_map(|feed| {
                        feed.content.as_ref().map(|fc| {
                            fc.articles.iter().map(|article| Headline {
                                title: article.title.clone(),
                                published_date: article.metadata.published_date.clone(),
                                source_name: article.metadata.feed_name.clone(),
                                url: article.metadata.url.clone(),
                            }).collect::<Vec<_>>()
                        }).unwrap_or_default()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Feed {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            url: None,
            content: None,
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

    pub fn set_content(&mut self, articles: NonEmpty<Article>, total_reading_time_minutes: u32) {
        self.content = Some(FeedContent {
            articles,
            total_reading_time_minutes,
        });
    }
}

impl Article {
    pub fn new(title: String, feed_name: String) -> Self {
        Self {
            title,
            metadata: ArticleMetadata {
                published_date: None,
                author: None,
                url: None,
                feed_name,
            },
            comments: Vec::new(),
            content: None,
        }
    }

    pub fn set_content(&mut self, blocks: NonEmpty<ContentBlock>, reading_time_minutes: u32) {
        self.content = Some(ArticleContent {
            blocks,
            reading_time_minutes,
        });
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
