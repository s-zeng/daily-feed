use crate::ast::*;
use std::error::Error;
use std::fs;
use std::path::Path;

pub struct MarkdownOutputter;

impl MarkdownOutputter {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_markdown(&self, document: &Document, output_filename: &str) -> Result<(), Box<dyn Error>> {
        let markdown_content = self.render_document_to_markdown(document)?;
        
        // Ensure the output directory exists
        if let Some(parent) = Path::new(output_filename).parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(output_filename, markdown_content)?;
        Ok(())
    }

    fn render_document_to_markdown(&self, document: &Document) -> Result<String, Box<dyn Error>> {
        let mut markdown = String::new();
        
        // Document header
        markdown.push_str(&format!("# {}\n\n", document.metadata.title));
        
        if let Some(description) = &document.metadata.description {
            markdown.push_str(&format!("{}\n\n", description));
        }
        
        markdown.push_str(&format!("**Author:** {}\n", document.metadata.author));
        markdown.push_str(&format!("**Generated:** {}\n", document.metadata.generated_at));
        markdown.push_str(&format!("**Total Articles:** {}\n\n", document.total_articles()));
        
        // Table of contents
        markdown.push_str("## Table of Contents\n\n");
        for feed in &document.feeds {
            markdown.push_str(&format!("- [{}](#{})\n", feed.name, self.to_anchor(&feed.name)));
            for article in &feed.articles {
                markdown.push_str(&format!("  - [{}](#{})\n", 
                    article.title, 
                    self.to_anchor(&article.title)
                ));
            }
        }
        markdown.push_str("\n---\n\n");
        
        // Feed sections
        for feed in &document.feeds {
            markdown.push_str(&self.render_feed_to_markdown(feed)?);
        }
        
        Ok(markdown)
    }

    fn render_feed_to_markdown(&self, feed: &Feed) -> Result<String, Box<dyn Error>> {
        let mut markdown = String::new();
        
        markdown.push_str(&format!("## {}\n\n", feed.name));
        
        if let Some(description) = &feed.description {
            markdown.push_str(&format!("{}\n\n", description));
        }
        
        markdown.push_str(&format!("**Total Articles:** {}\n\n", feed.articles.len()));
        
        for article in &feed.articles {
            markdown.push_str(&self.render_article_to_markdown(article)?);
            markdown.push_str("\n---\n\n");
        }
        
        Ok(markdown)
    }

    fn render_article_to_markdown(&self, article: &Article) -> Result<String, Box<dyn Error>> {
        let mut markdown = String::new();
        
        // Article header
        markdown.push_str(&format!("### {}\n\n", article.title));
        
        // Metadata
        if let Some(date) = &article.metadata.published_date {
            markdown.push_str(&format!("**Published:** {}\n", date));
        }
        if let Some(author) = &article.metadata.author {
            markdown.push_str(&format!("**Author:** {}\n", author));
        }
        markdown.push_str(&format!("**Source:** {}\n", article.metadata.feed_name));
        if let Some(url) = &article.metadata.url {
            markdown.push_str(&format!("**Link:** [Read original article]({})\n", url));
        }
        markdown.push_str("\n");
        
        // Content
        for block in &article.content {
            markdown.push_str(&self.render_content_block_to_markdown(block)?);
        }
        
        // Comments
        if !article.comments.is_empty() {
            markdown.push_str("\n#### Top Comments\n\n");
            for comment in &article.comments {
                markdown.push_str(&self.render_comment_to_markdown(comment)?);
            }
        }
        
        Ok(markdown)
    }

    fn render_comment_to_markdown(&self, comment: &Comment) -> Result<String, Box<dyn Error>> {
        let mut markdown = String::new();
        
        markdown.push_str(&format!("> **{}** (Score: {})\n", comment.author, comment.score));
        if let Some(timestamp) = &comment.timestamp {
            markdown.push_str(&format!("> *{}*\n", timestamp));
        }
        markdown.push_str(">\n");
        
        for block in &comment.content {
            let block_markdown = self.render_content_block_to_markdown(block)?;
            // Prefix each line with "> " for blockquote formatting
            for line in block_markdown.lines() {
                if line.trim().is_empty() {
                    markdown.push_str(">\n");
                } else {
                    markdown.push_str(&format!("> {}\n", line));
                }
            }
        }
        markdown.push_str("\n");
        
        Ok(markdown)
    }

    fn render_content_block_to_markdown(&self, block: &ContentBlock) -> Result<String, Box<dyn Error>> {
        match block {
            ContentBlock::Paragraph(content) => {
                Ok(format!("{}\n\n", self.render_text_content_to_markdown(content)?))
            }
            ContentBlock::Heading { level, content } => {
                let heading_prefix = "#".repeat(*level as usize + 3); // +3 because document starts at h1, feeds at h2, articles at h3
                Ok(format!("{} {}\n\n", heading_prefix, self.render_text_content_to_markdown(content)?))
            }
            ContentBlock::List { ordered, items } => {
                let mut list_markdown = String::new();
                for (i, item) in items.iter().enumerate() {
                    let prefix = if *ordered {
                        format!("{}. ", i + 1)
                    } else {
                        "- ".to_string()
                    };
                    list_markdown.push_str(&format!("{}{}\n", prefix, self.render_text_content_to_markdown(item)?));
                }
                list_markdown.push('\n');
                Ok(list_markdown)
            }
            ContentBlock::Quote(content) => {
                let quote_text = self.render_text_content_to_markdown(content)?;
                let mut quoted = String::new();
                for line in quote_text.lines() {
                    quoted.push_str(&format!("> {}\n", line));
                }
                quoted.push('\n');
                Ok(quoted)
            }
            ContentBlock::Code { language, content } => {
                let lang = language.as_deref().unwrap_or("");
                Ok(format!("```{}\n{}\n```\n\n", lang, content))
            }
            ContentBlock::Link { url, text } => {
                Ok(format!("[{}]({})\n\n", text, url))
            }
            ContentBlock::Image { url, alt } => {
                let alt_text = alt.as_deref().unwrap_or("");
                Ok(format!("![{}]({})\n\n", alt_text, url))
            }
            ContentBlock::Raw(html) => {
                // For raw HTML, we could try to convert it, but for now just wrap it
                Ok(format!("```html\n{}\n```\n\n", html))
            }
        }
    }

    fn render_text_content_to_markdown(&self, content: &TextContent) -> Result<String, Box<dyn Error>> {
        let mut markdown = String::new();
        
        for span in &content.spans {
            let mut text = span.text.clone();
            
            // Apply formatting
            if span.formatting.code {
                text = format!("`{}`", text);
            }
            if span.formatting.bold {
                text = format!("**{}**", text);
            }
            if span.formatting.italic {
                text = format!("*{}*", text);
            }
            if let Some(url) = &span.formatting.link {
                text = format!("[{}]({})", text, url);
            }
            
            markdown.push_str(&text);
        }
        
        Ok(markdown)
    }

    fn to_anchor(&self, text: &str) -> String {
        text.to_lowercase()
            .replace(' ', "-")
            .replace(['(', ')', '[', ']', '{', '}', '<', '>', '"', '\'', '/', '\\', '|', '?', '*', '&', '%', '$', '#', '@', '!', '^', '~', '`', '+', '=', ',', '.', ';', ':'], "")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_text_content() {
        let outputter = MarkdownOutputter::new();
        
        let content = TextContent::from_spans(vec![
            TextSpan::plain("Hello ".to_string()),
            TextSpan::bold("world".to_string()),
            TextSpan::plain("!".to_string()),
        ]);
        
        let markdown = outputter.render_text_content_to_markdown(&content).unwrap();
        assert_eq!(markdown, "Hello **world**!");
    }

    #[test]
    fn test_render_paragraph() {
        let outputter = MarkdownOutputter::new();
        
        let block = ContentBlock::Paragraph(TextContent::plain("Test paragraph".to_string()));
        let markdown = outputter.render_content_block_to_markdown(&block).unwrap();
        assert_eq!(markdown, "Test paragraph\n\n");
    }

    #[test]
    fn test_render_heading() {
        let outputter = MarkdownOutputter::new();
        
        let block = ContentBlock::Heading {
            level: 1,
            content: TextContent::plain("Test Heading".to_string()),
        };
        let markdown = outputter.render_content_block_to_markdown(&block).unwrap();
        assert_eq!(markdown, "#### Test Heading\n\n");
    }

    #[test]
    fn test_to_anchor() {
        let outputter = MarkdownOutputter::new();
        
        assert_eq!(outputter.to_anchor("Hello World"), "hello-world");
        assert_eq!(outputter.to_anchor("Test & More"), "test--more");
        assert_eq!(outputter.to_anchor("Complex (Test) [Case]!"), "complex-test-case");
    }

    #[test]
    fn test_render_code_block() {
        let outputter = MarkdownOutputter::new();
        
        let block = ContentBlock::Code {
            language: Some("rust".to_string()),
            content: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
        };
        let markdown = outputter.render_content_block_to_markdown(&block).unwrap();
        assert_eq!(markdown, "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\n");
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
        assert_eq!(markdown, "- Item 1\n- Item 2\n\n");
    }
}