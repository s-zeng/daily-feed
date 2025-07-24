use reqwest;
use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub content: String,
    pub author: String,
    pub score: i32,
    pub timestamp: Option<String>,
}


pub async fn fetch_top_comments(article_url: &str, limit: usize) -> Result<Vec<Comment>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    
    // First, fetch the article page to extract the iframe URL
    let response = client
        .get(article_url)
        .header("User-Agent", "daily-feed/0.1.0")
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let html_content = response.text().await?;
    let document = Html::parse_document(&html_content);
    
    // Extract the iframe URL from the data-url attribute
    let data_url_selector = Selector::parse("[data-url]").unwrap();
    let iframe_url = document
        .select(&data_url_selector)
        .next()
        .and_then(|element| element.value().attr("data-url"))
        .ok_or("Could not find iframe URL in article page")?;
    
    // Fetch the forum thread page
    let forum_response = client
        .get(iframe_url)
        .header("User-Agent", "daily-feed/0.1.0")
        .send()
        .await?;
    
    if !forum_response.status().is_success() {
        return Err(format!("HTTP error accessing forum: {}", forum_response.status()).into());
    }
    
    let forum_html = forum_response.text().await?;
    let forum_document = Html::parse_document(&forum_html);
    
    // Parse comments from the forum HTML
    let comments = parse_comments_from_html(&forum_document)?;
    
    // Sort by score (descending) and take top N
    let mut sorted_comments = comments;
    sorted_comments.sort_by(|a, b| b.score.cmp(&a.score));
    sorted_comments.truncate(limit);
    
    Ok(sorted_comments)
}

fn parse_comments_from_html(document: &Html) -> Result<Vec<Comment>, Box<dyn Error>> {
    let mut comments = Vec::new();
    
    // XenForo comment structure selectors
    let comment_selector = Selector::parse(".message").unwrap();
    let author_selector = Selector::parse(".username").unwrap();
    let content_selector = Selector::parse(".message-content .bbWrapper").unwrap();
    let score_selector = Selector::parse(".reactionsBar-link").unwrap();
    let timestamp_selector = Selector::parse("time").unwrap();
    
    for comment_element in document.select(&comment_selector) {
        // Extract author
        let author = comment_element
            .select(&author_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Anonymous".to_string());
        
        // Extract content
        let content = comment_element
            .select(&content_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| String::new());
        
        // Skip empty comments
        if content.is_empty() {
            continue;
        }
        
        // Extract score (reactions/likes)
        let score = comment_element
            .select(&score_selector)
            .next()
            .and_then(|el| el.text().collect::<String>().parse::<i32>().ok())
            .unwrap_or(0);
        
        // Extract timestamp
        let timestamp = comment_element
            .select(&timestamp_selector)
            .next()
            .and_then(|el| el.value().attr("datetime"))
            .map(|s| s.to_string());
        
        comments.push(Comment {
            content,
            author,
            score,
            timestamp,
        });
    }
    
    Ok(comments)
}

pub async fn fetch_top_5_comments(article_url: &str) -> Result<Vec<Comment>, Box<dyn Error>> {
    fetch_top_comments(article_url, 5).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_parse_comments_from_html() {
        let html = r#"
        <div class="message">
            <div class="username">testuser1</div>
            <div class="message-content">
                <div class="bbWrapper">This is a test comment</div>
            </div>
            <div class="reactionsBar-link">5</div>
            <time datetime="2025-01-01T12:00:00Z"></time>
        </div>
        <div class="message">
            <div class="username">testuser2</div>
            <div class="message-content">
                <div class="bbWrapper">Another test comment</div>
            </div>
            <div class="reactionsBar-link">3</div>
            <time datetime="2025-01-01T13:00:00Z"></time>
        </div>
        "#;
        
        let document = Html::parse_document(html);
        let comments = parse_comments_from_html(&document).unwrap();
        
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].author, "testuser1");
        assert_eq!(comments[0].content, "This is a test comment");
        assert_eq!(comments[0].score, 5);
        assert_eq!(comments[1].author, "testuser2");
        assert_eq!(comments[1].score, 3);
    }
}
