use crate::ast::*;
use regex::Regex;
use scraper::{ElementRef, Html, Node, Selector};
use std::error::Error;

pub async fn parse_feeds_to_document(
    channels: &[(String, rss::Channel)],
    title: String,
    author: String,
) -> Result<Document, Box<dyn Error>> {
    let mut document = Document::new(title, author);
    document.metadata.description = Some("Aggregated RSS feeds".to_string());

    for (feed_name, channel) in channels {
        let mut feed =
            Feed::new(feed_name.clone()).with_description(channel.description().to_string());

        if let Some(link) = channel.link().to_string().into() {
            feed = feed.with_url(link);
        }

        for item in channel.items() {
            let article = parse_rss_item_to_article(item, feed_name, channel).await?;
            feed.add_article(article);
        }

        document.add_feed(feed);
    }

    Ok(document)
}

async fn parse_rss_item_to_article(
    item: &rss::Item,
    feed_name: &str,
    _channel: &rss::Channel,
) -> Result<Article, Box<dyn Error>> {
    let title = item.title().unwrap_or("Untitled").to_string();
    let mut article = Article::new(title.clone(), feed_name.to_string());

    if let Some(pub_date) = item.pub_date() {
        article = article.with_published_date(pub_date.to_string());
    }

    if let Some(url) = item.link() {
        article = article.with_url(url.to_string());
    }

    // Parse content
    let content_html = item.content().or_else(|| item.description()).unwrap_or("");
    let content_blocks = parse_html_to_content_blocks(content_html)?;
    article = article.with_content(content_blocks);


    Ok(article)
}

pub fn parse_html_to_content_blocks(
    html: &str,
) -> Result<Vec<ContentBlock>, Box<dyn Error>> {
    if html.trim().is_empty() {
        return Ok(vec![]);
    }

    let document = Html::parse_fragment(html);
    let mut blocks = Vec::new();

    for node in document.root_element().children() {
        if let Some(element) = ElementRef::wrap(node) {
            if let Some(block) = parse_element_to_content_block(element)? {
                blocks.push(block);
            }
        } else if let Node::Text(text_node) = node.value() {
            let text = text_node.trim();
            if !text.is_empty() {
                blocks.push(ContentBlock::Paragraph(TextContent::plain(
                    text.to_string(),
                )));
            }
        }
    }

    // If no blocks were parsed, treat the entire HTML as a raw paragraph
    if blocks.is_empty() && !html.trim().is_empty() {
        let clean_text = strip_html_tags(html);
        if !clean_text.trim().is_empty() {
            blocks.push(ContentBlock::Paragraph(TextContent::plain(clean_text)));
        }
    }

    Ok(blocks)
}

fn parse_element_to_content_block(
    element: ElementRef,
) -> Result<Option<ContentBlock>, Box<dyn Error>> {
    let tag_name = element.value().name();

    match tag_name {
        "p" => {
            let text_content = parse_element_to_text_content(element)?;
            if !text_content.is_empty() {
                Ok(Some(ContentBlock::Paragraph(text_content)))
            } else {
                Ok(None)
            }
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag_name.chars().nth(1)
                .and_then(|c| c.to_digit(10))
                .map(|d| d as u8)
                .ok_or_else(|| format!("Invalid heading tag format: {}", tag_name))?;
            let text_content = parse_element_to_text_content(element)?;
            if !text_content.is_empty() {
                Ok(Some(ContentBlock::Heading {
                    level,
                    content: text_content,
                }))
            } else {
                Ok(None)
            }
        }
        "ul" | "ol" => {
            let ordered = tag_name == "ol";
            let li_selector = Selector::parse("li").unwrap();
            let mut items = Vec::new();

            for li in element.select(&li_selector) {
                let item_content = parse_element_to_text_content(li)?;
                if !item_content.is_empty() {
                    items.push(item_content);
                }
            }

            if !items.is_empty() {
                Ok(Some(ContentBlock::List { ordered, items }))
            } else {
                Ok(None)
            }
        }
        "blockquote" => {
            let text_content = parse_element_to_text_content(element)?;
            if !text_content.is_empty() {
                Ok(Some(ContentBlock::Quote(text_content)))
            } else {
                Ok(None)
            }
        }
        "pre" | "code" => {
            let code_text = element.text().collect::<String>();
            if !code_text.trim().is_empty() {
                let language = element.value().attr("class").and_then(|classes| {
                    classes
                        .split_whitespace()
                        .find(|class| class.starts_with("language-"))
                        .map(|class| class.strip_prefix("language-").unwrap().to_string())
                });
                Ok(Some(ContentBlock::Code {
                    language,
                    content: code_text,
                }))
            } else {
                Ok(None)
            }
        }
        "a" => {
            if let Some(href) = element.value().attr("href") {
                let link_text = element.text().collect::<String>();
                if !link_text.trim().is_empty() {
                    Ok(Some(ContentBlock::Link {
                        url: href.to_string(),
                        text: link_text,
                    }))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        "img" => {
            if let Some(src) = element.value().attr("src") {
                let alt = element.value().attr("alt").map(|s| s.to_string());
                Ok(Some(ContentBlock::Image {
                    url: src.to_string(),
                    alt,
                }))
            } else {
                Ok(None)
            }
        }
        "div" | "span" | "section" | "article" => {
            // For container elements, parse children and convert to paragraph if needed
            let text_content = parse_element_to_text_content(element)?;
            if !text_content.is_empty() {
                Ok(Some(ContentBlock::Paragraph(text_content)))
            } else {
                Ok(None)
            }
        }
        _ => {
            // For unknown elements, try to extract text content
            let text_content = parse_element_to_text_content(element)?;
            if !text_content.is_empty() {
                Ok(Some(ContentBlock::Paragraph(text_content)))
            } else {
                Ok(None)
            }
        }
    }
}

fn parse_element_to_text_content(
    element: ElementRef,
) -> Result<TextContent, Box<dyn Error>> {
    let mut spans = Vec::new();

    for node in element.children() {
        match node.value() {
            Node::Text(text_node) => {
                let text = text_node.to_string();
                if !text.trim().is_empty() {
                    spans.push(TextSpan::plain(text));
                }
            }
            Node::Element(_element_node) => {
                if let Some(child_element) = ElementRef::wrap(node) {
                    let child_spans = parse_inline_element_to_spans(child_element)?;
                    spans.extend(child_spans);
                }
            }
            _ => {}
        }
    }

    // If no spans were found, get all text content
    if spans.is_empty() {
        let all_text = element.text().collect::<String>();
        if !all_text.trim().is_empty() {
            spans.push(TextSpan::plain(all_text));
        }
    }

    Ok(TextContent::from_spans(spans))
}

fn parse_inline_element_to_spans(
    element: ElementRef,
) -> Result<Vec<TextSpan>, Box<dyn Error>> {
    let tag_name = element.value().name();
    let text = element.text().collect::<String>();

    if text.trim().is_empty() {
        return Ok(vec![]);
    }

    let span = match tag_name {
        "strong" | "b" => TextSpan::bold(text),
        "em" | "i" => TextSpan::italic(text),
        "code" => TextSpan::code(text),
        "a" => {
            if let Some(href) = element.value().attr("href") {
                TextSpan::link(text, href.to_string())
            } else {
                TextSpan::plain(text)
            }
        }
        _ => TextSpan::plain(text),
    };

    Ok(vec![span])
}

pub fn strip_html_tags(html: &str) -> String {
    use std::sync::OnceLock;
    
    static TAG_REGEX: OnceLock<Regex> = OnceLock::new();
    static ENTITY_REGEX: OnceLock<Regex> = OnceLock::new();
    static WHITESPACE_REGEX: OnceLock<Regex> = OnceLock::new();
    
    let tag_regex = TAG_REGEX.get_or_init(|| Regex::new(r"<[^>]*>").expect("Invalid tag regex"));
    let entity_regex = ENTITY_REGEX.get_or_init(|| {
        Regex::new(r"&[a-zA-Z][a-zA-Z0-9]*;|&#[0-9]+;|&#x[0-9a-fA-F]+;")
            .expect("Invalid entity regex")
    });
    let whitespace_regex = WHITESPACE_REGEX.get_or_init(|| {
        Regex::new(r"\s+").expect("Invalid whitespace regex")
    });

    let without_tags = tag_regex.replace_all(html, " ");
    let without_entities = entity_regex.replace_all(&without_tags, " ");

    // Clean up whitespace
    whitespace_regex
        .replace_all(&without_entities, " ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}