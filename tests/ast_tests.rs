use daily_feed::ast::*;
use nonempty::NonEmpty;

#[test]
fn test_document_creation() {
    let mut doc = Document::new("Test Document".to_string(), "Test Author".to_string());

    let mut feed = Feed::new("Test Feed".to_string()).with_description("A test feed".to_string());

    let mut article = Article::new("Test Article".to_string(), "Test Feed".to_string());
    article.set_content(
        NonEmpty::new(ContentBlock::Paragraph(TextContent::plain("Test content".to_string()))),
        0,
    );

    feed.set_content(NonEmpty::new(article), 0);
    doc.set_content(NonEmpty::new(feed), 0);

    // Filter out the timestamp for reproducible snapshots
    doc.metadata.generated_at = "2025-01-01T00:00:00.000000Z".to_string();

    insta::assert_json_snapshot!(doc);
}

#[test]
fn test_text_content_plain_text() {
    let content = TextContent::from_spans(vec![
        TextSpan::plain("Hello ".to_string()),
        TextSpan::bold("world".to_string()),
        TextSpan::plain("!".to_string()),
    ]);

    let plain_text = content.to_plain_text();
    insta::assert_snapshot!(plain_text);
}

#[test]
fn test_text_content_structure() {
    let content = TextContent::from_spans(vec![
        TextSpan::plain("Hello ".to_string()),
        TextSpan::bold("world".to_string()),
        TextSpan::plain("!".to_string()),
    ]);

    insta::assert_json_snapshot!(content);
}

#[test]
fn test_article_with_reading_time() {
    let article = Article {
        title: "Test Article".to_string(),
        metadata: ArticleMetadata {
            published_date: Some("2025-01-01".to_string()),
            author: Some("Test Author".to_string()),
            url: Some("https://example.com/article".to_string()),
            feed_name: "Test Feed".to_string(),
        },
        comments: vec![],
        content: Some(ArticleContent {
            blocks: NonEmpty::new(ContentBlock::Paragraph(TextContent::plain(
                "Test content".to_string(),
            ))),
            reading_time_minutes: 5,
        }),
    };

    insta::assert_json_snapshot!(article);
}

#[test]
fn test_article_without_reading_time() {
    let article = Article {
        title: "Test Article".to_string(),
        metadata: ArticleMetadata {
            published_date: Some("2025-01-01".to_string()),
            author: Some("Test Author".to_string()),
            url: Some("https://example.com/article".to_string()),
            feed_name: "Test Feed".to_string(),
        },
        comments: vec![],
        content: None,
    };

    insta::assert_json_snapshot!(article);
}

#[test]
fn test_feed_with_total_reading_time() {
    let feed = Feed {
        name: "Test Feed".to_string(),
        description: Some("Test description".to_string()),
        url: Some("https://example.com/feed".to_string()),
        content: Some(FeedContent {
            articles: NonEmpty::new(Article {
                title: "Article 1".to_string(),
                metadata: ArticleMetadata {
                    published_date: None,
                    author: None,
                    url: None,
                    feed_name: "Test Feed".to_string(),
                },
                comments: vec![],
                content: None,
            }),
            total_reading_time_minutes: 45,
        }),
    };

    insta::assert_json_snapshot!(feed);
}

#[test]
fn test_document_with_total_reading_time() {
    let doc = Document {
        metadata: DocumentMetadata {
            title: "Test Document".to_string(),
            author: "Test Author".to_string(),
            description: Some("Test description".to_string()),
            generated_at: "2025-01-01T00:00:00.000000Z".to_string(),
        },
        front_page: None,
        content: Some(DocumentContent {
            feeds: NonEmpty::new(Feed {
                name: "Test Feed".to_string(),
                description: None,
                url: None,
                content: Some(FeedContent {
                    articles: NonEmpty::new(Article {
                        title: "Test Article".to_string(),
                        metadata: ArticleMetadata {
                            published_date: None,
                            author: None,
                            url: None,
                            feed_name: "Test Feed".to_string(),
                        },
                        comments: vec![],
                        content: None,
                    }),
                    total_reading_time_minutes: 60,
                }),
            }),
            total_reading_time_minutes: 120,
        }),
    };

    insta::assert_json_snapshot!(doc);
}
