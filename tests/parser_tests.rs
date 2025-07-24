use daily_feed::parser::*;

#[test]
fn test_parse_simple_html() {
    let parser = DocumentParser::new();
    let html = "<p>Hello <strong>world</strong>!</p>";
    
    let blocks = parser.parse_html_to_content_blocks(html).unwrap();
    insta::assert_json_snapshot!(blocks);
}

#[test]
fn test_parse_heading() {
    let parser = DocumentParser::new();
    let html = "<h2>Test Heading</h2>";
    
    let blocks = parser.parse_html_to_content_blocks(html).unwrap();
    insta::assert_json_snapshot!(blocks);
}

#[test]
fn test_parse_list() {
    let parser = DocumentParser::new();
    let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
    
    let blocks = parser.parse_html_to_content_blocks(html).unwrap();
    insta::assert_json_snapshot!(blocks);
}

#[test]
fn test_strip_html_tags() {
    let parser = DocumentParser::new();
    let html = "<p>Hello <strong>world</strong> &amp; everyone!</p>";
    let result = parser.strip_html_tags(html);
    insta::assert_snapshot!(result);
}