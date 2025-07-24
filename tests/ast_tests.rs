use daily_feed::ast::*;

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