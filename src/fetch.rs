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
        
        // Description/content
        if let Some(description) = item.description() {
            // Simple HTML tag removal for basic text extraction
            let clean_description = strip_html_tags(description);
            markdown.push_str(&format!("{}\n\n", clean_description));
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
    let mut chars = html.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => {
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
            _ => {} // ignore characters inside tags
        }
    }
    
    // Clean up excessive whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
