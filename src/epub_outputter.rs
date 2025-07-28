use crate::ast::*;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, TocElement, ZipLibrary};
use std::error::Error;
use std::fs::File;

pub struct EpubOutputter {
    builder: EpubBuilder<ZipLibrary>,
}

impl EpubOutputter {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let builder = EpubBuilder::new(ZipLibrary::new()?)?;
        Ok(Self { builder })
    }

    pub fn generate_epub(&mut self, document: &Document, output_filename: &str) -> Result<(), Box<dyn Error>> {
        self.set_metadata(document)?;
        self.add_stylesheet()?;
        self.add_title_page(document)?;
        self.add_front_page(document)?;
        self.add_table_of_contents(document)?;
        self.add_content(document)?;
        self.write_to_file(output_filename)?;
        Ok(())
    }

    fn set_metadata(&mut self, document: &Document) -> Result<(), Box<dyn Error>> {
        self.builder.metadata("author", &document.metadata.author)?;
        self.builder.metadata("title", &document.metadata.title)?;
        
        if let Some(description) = &document.metadata.description {
            self.builder.metadata("description", description)?;
        }
        
        Ok(())
    }

    fn add_stylesheet(&mut self) -> Result<(), Box<dyn Error>> {
        let css = r#"
        body { font-family: serif; margin: 2em; line-height: 1.6; }
        h1 { color: #333; border-bottom: 2px solid #333; }
        h2 { color: #555; margin-top: 2em; }
        h3, h4, h5, h6 { color: #666; margin-top: 1.5em; }
        .pub-date { color: #666; font-style: italic; margin-bottom: 1em; }
        .content { margin-bottom: 2em; }
        .link { margin-top: 1em; }
        hr { margin: 2em 0; border: 1px solid #ccc; }
        
        p { margin: 1em 0; }
        blockquote { 
            margin: 1em 2em; 
            padding-left: 1em; 
            border-left: 3px solid #ccc; 
            font-style: italic;
        }
        ul, ol { margin: 1em 0; padding-left: 2em; }
        li { margin: 0.5em 0; }
        code { 
            background-color: #f4f4f4; 
            padding: 0.2em 0.4em; 
            font-family: monospace; 
            border-radius: 3px;
        }
        pre { 
            background-color: #f4f4f4; 
            padding: 1em; 
            overflow-x: auto; 
            border-radius: 3px;
            font-family: monospace;
        }
        strong, b { font-weight: bold; }
        em, i { font-style: italic; }
        a { color: #0066cc; text-decoration: underline; }
        img { max-width: 100%; height: auto; margin: 1em 0; }
        
        .toc { margin: 2em 0; }
        .toc h2 { color: #333; margin-bottom: 1em; }
        .toc ul { list-style-type: none; padding-left: 0; }
        .toc li { margin: 0.5em 0; }
        .toc a { color: #0066cc; text-decoration: none; }
        .toc a:hover { text-decoration: underline; }
        .toc .feed-section { font-weight: bold; margin-top: 1em; }
        .toc .article-item { margin-left: 2em; font-weight: normal; }
        
        .comments-section { 
            margin-top: 3em; 
            border-top: 2px solid #ccc; 
            padding-top: 2em; 
        }
        .comments-section h2 { color: #333; margin-bottom: 1em; }
        .comment { 
            margin: 1.5em 0; 
            padding: 1em; 
            background-color: #f9f9f9; 
            border-left: 3px solid #0066cc;
            border-radius: 3px;
        }
        .comment-author { 
            font-weight: bold; 
            color: #333; 
            margin-bottom: 0.5em; 
        }
        .comment-score { 
            color: #666; 
            font-size: 0.9em; 
            margin-left: 1em; 
        }
        .comment-content { 
            margin-top: 0.5em; 
            line-height: 1.5; 
        }
        "#;
        
        self.builder.stylesheet(css.as_bytes())?;
        Ok(())
    }

    fn add_title_page(&mut self, document: &Document) -> Result<(), Box<dyn Error>> {
        let feed_list = document.feeds.iter()
            .map(|feed| format!(
                "<li><strong>{}:</strong> {} ({} articles)</li>",
                feed.name,
                feed.description.as_deref().unwrap_or("No description"),
                feed.articles.len()
            ))
            .collect::<Vec<_>>()
            .join("\n        ");

        let title_page = format!(
            r#"<html>
            <head><title>{}</title></head>
            <body>
            <h1>{}</h1>
            <p>{}</p>
            <p><strong>Generated:</strong> {}</p>
            <p><strong>Total Articles:</strong> {}</p>
            <h2>Feeds</h2>
            <ul>
            {}
            </ul>
            </body>
            </html>"#,
            document.metadata.title,
            document.metadata.title,
            document.metadata.description.as_deref().unwrap_or(""),
            document.metadata.generated_at,
            document.total_articles(),
            feed_list
        );

        self.builder.add_content(
            EpubContent::new("title.xhtml", title_page.as_bytes())
                .title("Title Page")
                .reftype(ReferenceType::TitlePage),
        )?;

        Ok(())
    }

    fn add_front_page(&mut self, document: &Document) -> Result<(), Box<dyn Error>> {
        if let Some(front_page_content) = &document.front_page {
            let front_page_html = format!(
                r#"<html>
                <head><title>Front Page Summary</title></head>
                <body>
                <h1>Front Page Summary</h1>
                <div class="content">{}</div>
                </body>
                </html>"#,
                front_page_content
            );

            self.builder.add_content(
                EpubContent::new("front_page.xhtml", front_page_html.as_bytes())
                    .title("Front Page Summary")
                    .reftype(ReferenceType::Text),
            )?;
        }
        Ok(())
    }

    fn add_table_of_contents(&mut self, document: &Document) -> Result<(), Box<dyn Error>> {
        let mut toc_content = format!(
            r#"<html>
            <head><title>Table of Contents</title></head>
            <body>
            <div class="toc">
            <h1>Table of Contents</h1>
            <ul>
            "#
        );

        // Add front page to TOC if it exists
        if document.front_page.is_some() {
            toc_content.push_str(
                r#"            <li class="feed-section"><a href="front_page.xhtml">Front Page Summary</a></li>
            "#
            );
        }

        let mut chapter_index = 0;
        for feed in &document.feeds {
            chapter_index += 1;
            
            toc_content.push_str(&format!(
                r#"            <li class="feed-section"><a href="feed_{}.xhtml">{}</a>
                <ul>
                "#,
                chapter_index, feed.name
            ));

            for article in &feed.articles {
                chapter_index += 1;
                toc_content.push_str(&format!(
                    r#"                    <li class="article-item"><a href="article_{}.xhtml">{}</a></li>
                    "#,
                    chapter_index, article.title
                ));
            }

            toc_content.push_str("                </ul>\n            </li>\n");
        }

        toc_content.push_str(
            r#"        </ul>
            </div>
            </body>
            </html>"#,
        );

        self.builder.add_content(
            EpubContent::new("toc.xhtml", toc_content.as_bytes())
                .title("Table of Contents")
                .reftype(ReferenceType::Text),
        )?;

        Ok(())
    }

    fn add_content(&mut self, document: &Document) -> Result<(), Box<dyn Error>> {
        let mut chapter_index = 0;
        
        for feed in &document.feeds {
            chapter_index += 1;
            
            // Add feed section page
            let feed_section_html = format!(
                r#"<html>
                <head><title>{} - Feed</title></head>
                <body>
                <h1>{}</h1>
                <p><strong>Description:</strong> {}</p>
                <p><strong>Total Articles:</strong> {}</p>
                <hr/>
                </body>
                </html>"#,
                feed.name,
                feed.name,
                feed.description.as_deref().unwrap_or("No description"),
                feed.articles.len()
            );

            let mut feed_content = EpubContent::new(
                format!("feed_{}.xhtml", chapter_index),
                feed_section_html.as_bytes(),
            )
            .title(&format!("{} - Feed", feed.name))
            .reftype(ReferenceType::Text);

            // Add articles
            for article in &feed.articles {
                chapter_index += 1;
                let article_filename = format!("article_{}.xhtml", chapter_index);
                
                let article_html = self.render_article_to_html(article)?;
                
                feed_content = feed_content.child(TocElement::new(&article_filename, &article.title));
                
                self.builder.add_content(
                    EpubContent::new(article_filename, article_html.as_bytes())
                        .title(&article.title)
                        .reftype(ReferenceType::Text),
                )?;
            }

            self.builder.add_content(feed_content)?;
        }

        Ok(())
    }

    fn render_article_to_html(&self, article: &Article) -> Result<String, Box<dyn Error>> {
        let mut content_html = String::new();
        
        for block in &article.content {
            content_html.push_str(&self.render_content_block_to_html(block)?);
        }

        let comments_html = if !article.comments.is_empty() {
            let mut comments_section = String::from(
                r#"<div class="comments-section">
                <h2>Top Comments</h2>"#
            );
            
            for comment in &article.comments {
                let mut comment_content = String::new();
                for block in &comment.content {
                    comment_content.push_str(&self.render_content_block_to_html(block)?);
                }
                
                comments_section.push_str(&format!(
                    r#"<div class="comment">
                        <div class="comment-author">{}<span class="comment-score">Score: {}</span></div>
                        <div class="comment-content">{}</div>
                    </div>"#,
                    comment.author,
                    comment.score,
                    comment_content
                ));
            }
            
            comments_section.push_str("</div>");
            comments_section
        } else {
            String::new()
        };

        let article_html = format!(
            r#"<html>
            <head><title>{}</title></head>
            <body>
            <h1>{}</h1>
            <div class="pub-date">{} - <strong>Source:</strong> {}</div>
            <div class="content">{}</div>
            {}
            {}
            </body>
            </html>"#,
            article.title,
            article.title,
            article.metadata.published_date.as_deref().unwrap_or(""),
            article.metadata.feed_name,
            content_html,
            if let Some(url) = &article.metadata.url {
                format!("<div class=\"link\"><a href=\"{}\">Read original article</a></div>", url)
            } else {
                String::new()
            },
            comments_html
        );

        Ok(article_html)
    }

    pub fn render_content_block_to_html(&self, block: &ContentBlock) -> Result<String, Box<dyn Error>> {
        match block {
            ContentBlock::Paragraph(content) => {
                Ok(format!("<p>{}</p>", self.render_text_content_to_html(content)?))
            }
            ContentBlock::Heading { level, content } => {
                Ok(format!(
                    "<h{}>{}</h{}>",
                    level,
                    self.render_text_content_to_html(content)?,
                    level
                ))
            }
            ContentBlock::List { ordered, items } => {
                let tag = if *ordered { "ol" } else { "ul" };
                let items_html = items
                    .iter()
                    .map(|item| format!("<li>{}</li>", self.render_text_content_to_html(item).unwrap_or_default()))
                    .collect::<Vec<_>>()
                    .join("");
                Ok(format!("<{}>{}</{}>", tag, items_html, tag))
            }
            ContentBlock::Quote(content) => {
                Ok(format!("<blockquote>{}</blockquote>", self.render_text_content_to_html(content)?))
            }
            ContentBlock::Code { language: _, content } => {
                Ok(format!("<pre><code>{}</code></pre>", html_escape::encode_text(content)))
            }
            ContentBlock::Link { url, text } => {
                Ok(format!("<a href=\"{}\">{}</a>", url, html_escape::encode_text(text)))
            }
            ContentBlock::Image { url, alt } => {
                let alt_attr = alt.as_ref()
                    .map(|a| format!(" alt=\"{}\"", html_escape::encode_double_quoted_attribute(a)))
                    .unwrap_or_default();
                Ok(format!("<img src=\"{}\"{} />", url, alt_attr))
            }
            ContentBlock::Raw(html) => Ok(html.clone()),
        }
    }

    pub fn render_text_content_to_html(&self, content: &TextContent) -> Result<String, Box<dyn Error>> {
        let mut html = String::new();
        
        for span in &content.spans {
            let text = html_escape::encode_text(&span.text);
            let mut span_html = text.to_string();
            
            if span.formatting.bold {
                span_html = format!("<strong>{}</strong>", span_html);
            }
            if span.formatting.italic {
                span_html = format!("<em>{}</em>", span_html);
            }
            if span.formatting.code {
                span_html = format!("<code>{}</code>", span_html);
            }
            if let Some(url) = &span.formatting.link {
                span_html = format!("<a href=\"{}\">{}</a>", url, span_html);
            }
            
            html.push_str(&span_html);
        }
        
        Ok(html)
    }

    fn write_to_file(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let mut output_file = File::create(filename)?;
        self.builder.generate(&mut output_file)?;
        Ok(())
    }
}

