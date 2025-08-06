use daily_feed::parser::*;

#[test]
fn test_parse_simple_html() {
    let html = "<p>Hello <strong>world</strong>!</p>";

    let blocks = parse_html_to_content_blocks(html).unwrap();
    insta::assert_json_snapshot!(blocks);
}

#[test]
fn test_parse_heading() {
    let html = "<h2>Important News</h2>";

    let blocks = parse_html_to_content_blocks(html).unwrap();
    insta::assert_json_snapshot!(blocks);
}

#[test]
fn test_parse_list() {
    let html = "<ul><li>First item</li><li>Second item</li></ul>";

    let blocks = parse_html_to_content_blocks(html).unwrap();
    insta::assert_json_snapshot!(blocks);
}

#[test]
fn test_strip_html_tags() {
    let html = "<p>Hello <strong>world</strong>! <em>This</em> is a <a href=\"#\">test</a>.</p>";
    let result = strip_html_tags(html);
    insta::assert_snapshot!(result);
}