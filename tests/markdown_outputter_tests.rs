use daily_feed::markdown_outputter::*;
use daily_feed::ast::*;

#[test]
fn test_render_text_content() {
    let outputter = MarkdownOutputter::new();
    
    let content = TextContent::from_spans(vec![
        TextSpan::plain("Hello ".to_string()),
        TextSpan::bold("world".to_string()),
        TextSpan::plain("!".to_string()),
    ]);
    
    let markdown = outputter.render_text_content_to_markdown(&content).unwrap();
    insta::assert_snapshot!(markdown);
}

#[test]
fn test_render_paragraph() {
    let outputter = MarkdownOutputter::new();
    
    let block = ContentBlock::Paragraph(TextContent::plain("Test paragraph".to_string()));
    let markdown = outputter.render_content_block_to_markdown(&block).unwrap();
    insta::assert_snapshot!(markdown);
}

#[test]
fn test_render_heading() {
    let outputter = MarkdownOutputter::new();
    
    let block = ContentBlock::Heading {
        level: 1,
        content: TextContent::plain("Test Heading".to_string()),
    };
    let markdown = outputter.render_content_block_to_markdown(&block).unwrap();
    insta::assert_snapshot!(markdown);
}

#[test]
fn test_to_anchor() {
    let outputter = MarkdownOutputter::new();
    
    let result1 = outputter.to_anchor("Hello World");
    insta::assert_snapshot!(result1, @"hello-world");
}

#[test]
fn test_to_anchor_special_chars() {
    let outputter = MarkdownOutputter::new();
    
    let result = outputter.to_anchor("Test & More");
    insta::assert_snapshot!(result, @"test--more");
}

#[test]
fn test_to_anchor_complex() {
    let outputter = MarkdownOutputter::new();
    
    let result = outputter.to_anchor("Complex (Test) [Case]!");
    insta::assert_snapshot!(result, @"complex-test-case");
}

#[test]
fn test_render_code_block() {
    let outputter = MarkdownOutputter::new();
    
    let block = ContentBlock::Code {
        language: Some("rust".to_string()),
        content: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
    };
    let markdown = outputter.render_content_block_to_markdown(&block).unwrap();
    insta::assert_snapshot!(markdown);
}

#[test]
fn test_render_list() {
    let outputter = MarkdownOutputter::new();
    
    let block = ContentBlock::List {
        ordered: false,
        items: vec![
            TextContent::plain("Item 1".to_string()),
            TextContent::plain("Item 2".to_string()),
        ],
    };
    let markdown = outputter.render_content_block_to_markdown(&block).unwrap();
    insta::assert_snapshot!(markdown);
}

#[test]
fn test_render_document_with_front_page() {
    let outputter = MarkdownOutputter::new();
    
    let article = Article {
        title: "Test Article".to_string(),
        content: vec![ContentBlock::Paragraph(TextContent::plain("Article content".to_string()))],
        metadata: ArticleMetadata {
            published_date: Some("2025-01-01T00:00:00.000000Z".to_string()),
            author: Some("Test Author".to_string()),
            url: Some("https://example.com/article".to_string()),
            feed_name: "Test Feed".to_string(),
        },
        comments: vec![],
    };
    
    let feed = Feed {
        name: "Test Feed".to_string(),
        description: Some("A test feed".to_string()),
        url: Some("https://example.com/feed".to_string()),
        articles: vec![article],
    };
    
    let document = Document {
        metadata: DocumentMetadata {
            title: "Test Document".to_string(),
            author: "Test Author".to_string(),
            description: Some("Test description".to_string()),
            generated_at: "2025-01-01T00:00:00.000000Z".to_string(),
        },
        front_page: Some("This is a front page summary with important news highlights.".to_string()),
        feeds: vec![feed],
    };
    
    let markdown = outputter.render_document_to_markdown(&document).unwrap();
    insta::assert_snapshot!(markdown);
}

#[test]
fn test_render_document_without_front_page() {
    let outputter = MarkdownOutputter::new();
    
    let article = Article {
        title: "Test Article".to_string(),
        content: vec![ContentBlock::Paragraph(TextContent::plain("Article content".to_string()))],
        metadata: ArticleMetadata {
            published_date: Some("2025-01-01T00:00:00.000000Z".to_string()),
            author: Some("Test Author".to_string()),
            url: Some("https://example.com/article".to_string()),
            feed_name: "Test Feed".to_string(),
        },
        comments: vec![],
    };
    
    let feed = Feed {
        name: "Test Feed".to_string(),
        description: Some("A test feed".to_string()),
        url: Some("https://example.com/feed".to_string()),
        articles: vec![article],
    };
    
    let document = Document {
        metadata: DocumentMetadata {
            title: "Test Document".to_string(),
            author: "Test Author".to_string(),
            description: Some("Test description".to_string()),
            generated_at: "2025-01-01T00:00:00.000000Z".to_string(),
        },
        front_page: None,
        feeds: vec![feed],
    };
    
    let markdown = outputter.render_document_to_markdown(&document).unwrap();
    insta::assert_snapshot!(markdown);
}

#[test]
fn test_front_page_multiline_content() {
    let outputter = MarkdownOutputter::new();
    
    let article = Article {
        title: "Test Article".to_string(),
        content: vec![ContentBlock::Paragraph(TextContent::plain("Article content".to_string()))],
        metadata: ArticleMetadata {
            published_date: Some("2025-01-01T00:00:00.000000Z".to_string()),
            author: Some("Test Author".to_string()),
            url: Some("https://example.com/article".to_string()),
            feed_name: "Test Feed".to_string(),
        },
        comments: vec![],
    };
    
    let feed = Feed {
        name: "Test Feed".to_string(),
        description: Some("A test feed".to_string()),
        url: Some("https://example.com/feed".to_string()),
        articles: vec![article],
    };
    
    let document = Document {
        metadata: DocumentMetadata {
            title: "Test Document".to_string(),
            author: "Test Author".to_string(),
            description: Some("Test description".to_string()),
            generated_at: "2025-01-01T00:00:00.000000Z".to_string(),
        },
        front_page: Some("# Breaking News\n\nMultiple important stories today:\n\n- Economic markets show volatility\n- Technology sector announces breakthrough\n- Climate summit reaches agreement".to_string()),
        feeds: vec![feed],
    };
    
    let markdown = outputter.render_document_to_markdown(&document).unwrap();
    insta::assert_snapshot!(markdown);
}