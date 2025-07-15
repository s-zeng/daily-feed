use std::error::Error;

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

pub fn channel_to_markdown(channel: &rss::Channel) -> String {
    let mut markdown = String::new();
    
    // Feed title as main header
    markdown.push_str(&format!("# {}\n\n", channel.title()));
    
    // Feed description
    markdown.push_str(&format!("{}\n\n", channel.description()));
    
    // Feed link if available
    if let Some(link) = channel.link().strip_prefix("http").or_else(|| channel.link().strip_prefix("https")) {
        markdown.push_str(&format!("🔗 [{}](http{})\n\n", channel.link(), link));
    } else {
        markdown.push_str(&format!("🔗 {}\n\n", channel.link()));
    }
    
    markdown.push_str("---\n\n");
    
    // Items
    for item in channel.items() {
        // Item title as header
        if let Some(title) = item.title() {
            markdown.push_str(&format!("## {}\n\n", title));
        }
        
        // Publication date
        if let Some(pub_date) = item.pub_date() {
            markdown.push_str(&format!("📅 *{}*\n\n", pub_date));
        }
        
        // Description/content - try content first, then description
        let content = item.content()
            .or_else(|| item.description())
            .unwrap_or("");
        
        if !content.is_empty() {
            // Simple HTML tag removal for basic text extraction
            let clean_content = strip_html_tags(content);
            markdown.push_str(&format!("{}\n\n", clean_content));
        }
        
        // Link
        if let Some(link) = item.link() {
            markdown.push_str(&format!("🔗 [Read more]({})\n\n", link));
        }
        
        markdown.push_str("---\n\n");
    }
    
    markdown
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut tag_content = String::new();
    let mut chars = html.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                in_tag = true;
                tag_content.clear();
            }
            '>' => {
                in_tag = false;
                // Convert block-level HTML tags to paragraph breaks
                let tag_lower = tag_content.to_lowercase();
                if tag_lower.starts_with("p") || tag_lower.starts_with("/p") ||
                   tag_lower.starts_with("br") || tag_lower.starts_with("/br") ||
                   tag_lower.starts_with("div") || tag_lower.starts_with("/div") ||
                   tag_lower.starts_with("h1") || tag_lower.starts_with("h2") ||
                   tag_lower.starts_with("h3") || tag_lower.starts_with("h4") ||
                   tag_lower.starts_with("h5") || tag_lower.starts_with("h6") ||
                   tag_lower.starts_with("/h1") || tag_lower.starts_with("/h2") ||
                   tag_lower.starts_with("/h3") || tag_lower.starts_with("/h4") ||
                   tag_lower.starts_with("/h5") || tag_lower.starts_with("/h6") {
                    result.push('\n');
                }
                tag_content.clear();
            }
            _ if in_tag => {
                tag_content.push(ch);
            }
            _ => {
                // Convert common HTML entities
                if ch == '&' {
                    let mut entity = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch == ';' {
                            chars.next(); // consume the ';'
                            break;
                        }
                        entity.push(chars.next().unwrap());
                    }
                    
                    match entity.as_str() {
                        "amp" => result.push('&'),
                        "lt" => result.push('<'),
                        "gt" => result.push('>'),
                        "quot" => result.push('"'),
                        "apos" => result.push('\''),
                        "nbsp" => result.push(' '),
                        _ => {
                            // Unknown entity, keep as is
                            result.push('&');
                            result.push_str(&entity);
                            result.push(';');
                        }
                    }
                } else {
                    result.push(ch);
                }
            }
        }
    }
    
    // Clean up excessive whitespace while preserving paragraph breaks
    result
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}
