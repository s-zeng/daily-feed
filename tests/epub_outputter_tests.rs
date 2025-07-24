use daily_feed::epub_outputter::*;
use daily_feed::ast::*;

#[test]
fn test_render_text_content() {
    let outputter = EpubOutputter::new().unwrap();
    
    let content = TextContent::from_spans(vec![
        TextSpan::plain("Hello ".to_string()),
        TextSpan::bold("world".to_string()),
        TextSpan::plain("!".to_string()),
    ]);
    
    let html = outputter.render_text_content_to_html(&content).unwrap();
    insta::assert_snapshot!(html);
}

#[test]
fn test_render_paragraph() {
    let outputter = EpubOutputter::new().unwrap();
    
    let block = ContentBlock::Paragraph(TextContent::plain("Test paragraph".to_string()));
    let html = outputter.render_content_block_to_html(&block).unwrap();
    insta::assert_snapshot!(html);
}

#[test]
fn test_render_heading() {
    let outputter = EpubOutputter::new().unwrap();
    
    let block = ContentBlock::Heading {
        level: 2,
        content: TextContent::plain("Test Heading".to_string()),
    };
    let html = outputter.render_content_block_to_html(&block).unwrap();
    insta::assert_snapshot!(html);
}