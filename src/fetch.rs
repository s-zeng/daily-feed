use crate::config::Config;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use std::error::Error;
use std::fs::File;

pub async fn feed_from_url(url: &str) -> Result<rss::Channel, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "daily-feed/0.1.0")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let content = response.bytes().await?;
    let channel = rss::Channel::read_from(&content[..])?;
    Ok(channel)
}

pub async fn fetch_all_feeds(
    config: &Config,
) -> Result<Vec<(String, rss::Channel)>, Box<dyn Error>> {
    let mut results = Vec::new();

    for feed in &config.feeds {
        match feed_from_url(&feed.url).await {
            Ok(channel) => {
                println!("Successfully fetched: {}", feed.name);
                results.push((feed.name.clone(), channel));
            }
            Err(e) => {
                eprintln!("Failed to fetch {}: {}", feed.name, e);
            }
        }
    }

    Ok(results)
}

pub fn channels_to_epub(
    channels: &[(String, rss::Channel)],
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    // Set metadata
    builder.metadata("author", &config.output.author)?;
    builder.metadata("title", &config.output.title)?;
    builder.metadata("description", "Aggregated RSS feeds")?;

    // Add comprehensive CSS for HTML content
    let css = r#"
        body { font-family: serif; margin: 2em; line-height: 1.6; }
        h1 { color: #333; border-bottom: 2px solid #333; }
        h2 { color: #555; margin-top: 2em; }
        h3, h4, h5, h6 { color: #666; margin-top: 1.5em; }
        .pub-date { color: #666; font-style: italic; margin-bottom: 1em; }
        .content { margin-bottom: 2em; }
        .link { margin-top: 1em; }
        hr { margin: 2em 0; border: 1px solid #ccc; }
        
        /* Preserve HTML formatting */
        p { margin: 1em 0; }
        blockquote { margin: 1em 2em; padding-left: 1em; border-left: 3px solid #ccc; }
        ul, ol { margin: 1em 0; padding-left: 2em; }
        li { margin: 0.5em 0; }
        code { background-color: #f4f4f4; padding: 0.2em 0.4em; font-family: monospace; }
        pre { background-color: #f4f4f4; padding: 1em; overflow-x: auto; }
        strong, b { font-weight: bold; }
        em, i { font-style: italic; }
        a { color: #0066cc; text-decoration: underline; }
        img { max-width: 100%; height: auto; margin: 1em 0; }
        table { border-collapse: collapse; width: 100%; margin: 1em 0; }
        th, td { border: 1px solid #ccc; padding: 0.5em; text-align: left; }
        th { background-color: #f4f4f4; font-weight: bold; }
    "#;
    builder.stylesheet(css.as_bytes())?;

    // Create title page
    let title_page = format!(
        r#"<html>
        <head><title>{}</title></head>
        <body>
        <h1>{}</h1>
        <p>Aggregated RSS feeds</p>
        <ul>
        {}
        </ul>
        </body>
        </html>"#,
        config.output.title,
        config.output.title,
        channels
            .iter()
            .map(|(name, channel)| format!(
                "<li><strong>{}:</strong> {}</li>",
                name,
                channel.description()
            ))
            .collect::<Vec<_>>()
            .join("\n        ")
    );

    builder.add_content(
        EpubContent::new("title.xhtml", title_page.as_bytes())
            .title("Title Page")
            .reftype(ReferenceType::TitlePage),
    )?;

    // Add each RSS item as a chapter from all feeds
    let mut chapter_index = 0;
    for (feed_name, channel) in channels {
        for item in channel.items() {
            chapter_index += 1;
            let chapter_title = item.title().unwrap_or("Untitled");
            let pub_date = item.pub_date().unwrap_or("");

            let content = item.content().or_else(|| item.description()).unwrap_or("");

            let clean_content = sanitize_html_for_epub(content);

            let chapter_html = format!(
                r#"<html>
                <head><title>{}</title></head>
                <body>
                <h1>{}</h1>
                <div class="pub-date">{} - <strong>Source:</strong> {}</div>
                <div class="content">{}</div>
                {}
                </body>
                </html>"#,
                chapter_title,
                chapter_title,
                pub_date,
                feed_name,
                clean_content,
                if let Some(link) = item.link() {
                    format!(
                        "<div class=\"link\"><a href=\"{}\">Read original article</a></div>",
                        link
                    )
                } else {
                    String::new()
                }
            );

            builder.add_content(
                EpubContent::new(
                    format!("chapter_{}.xhtml", chapter_index),
                    chapter_html.as_bytes(),
                )
                .title(chapter_title)
                .reftype(ReferenceType::Text),
            )?;
        }
    }

    // Generate the EPUB
    let mut output_file = File::create(&config.output.filename)?;
    builder.generate(&mut output_file)?;

    Ok(())
}

fn sanitize_html_for_epub(html: &str) -> String {
    use regex::Regex;

    // EPUB supports most HTML tags, so we'll preserve them but clean up problematic ones
    let mut result = html.to_string();

    // Create regex patterns for cleaning
    let script_regex = Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap();
    let style_regex = Regex::new(r"(?i)<style[^>]*>.*?</style>").unwrap();
    let style_attr_regex = Regex::new(r#"\s+style="[^"]*""#).unwrap();
    let onclick_regex = Regex::new(r#"\s+onclick="[^"]*""#).unwrap();
    let onload_regex = Regex::new(r#"\s+onload="[^"]*""#).unwrap();
    let class_regex = Regex::new(r#"\s+class="[^"]*""#).unwrap();
    let whitespace_regex = Regex::new(r"\s+").unwrap();

    // Remove or replace problematic tags and attributes for EPUB compatibility
    result = script_regex.replace_all(&result, "").to_string();
    result = style_regex.replace_all(&result, "").to_string();
    result = style_attr_regex.replace_all(&result, "").to_string();
    result = onclick_regex.replace_all(&result, "").to_string();
    result = onload_regex.replace_all(&result, "").to_string();
    result = class_regex.replace_all(&result, "").to_string();

    // Convert some common but potentially problematic tags to safer alternatives
    result = result
        .replace("<font ", "<span ")
        .replace("</font>", "</span>");

    // Clean up excessive whitespace while preserving structure
    result = whitespace_regex.replace_all(&result, " ").to_string();

    // Decode HTML entities (including numeric character references)
    result = result
        .replace("&#039;", "'") // Decode apostrophe
        .replace("&#8217;", "'") // Decode right single quotation mark
        .replace("&#8216;", "'") // Decode left single quotation mark
        .replace("&#8220;", "\"") // Decode left double quotation mark
        .replace("&#8221;", "\"") // Decode right double quotation mark
        .replace("&#8211;", "-") // Decode en dash
        .replace("&#8212;", "-") // Decode em dash
        .replace("&#8230;", "...") // Decode ellipsis
        .replace("&#160;", " ") // Decode non-breaking space
        .replace("&amp;", "&") // Normalize first
        .replace("&", "&amp;") // Then re-encode
        .replace("&amp;lt;", "&lt;")
        .replace("&amp;gt;", "&gt;")
        .replace("&amp;quot;", "&quot;")
        .replace("&amp;apos;", "&apos;")
        .replace("&amp;nbsp;", "&nbsp;");

    result.trim().to_string()
}
