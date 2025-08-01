use regex::Regex;
use reqwest;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub content: String,
    pub author: String,
    pub upvotes: u32,
    pub downvotes: u32,
    pub timestamp: Option<String>,
}

pub async fn fetch_top_comments(
    article_url: &str,
    limit: usize,
) -> Result<Vec<Comment>, Box<dyn Error>> {
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

    // Sort by net score (upvotes - downvotes, descending) and take top N
    let mut sorted_comments = comments;
    sorted_comments.sort_by(|a, b| {
        let a_net = a.upvotes as i32 - a.downvotes as i32;
        let b_net = b.upvotes as i32 - b.downvotes as i32;
        b_net.cmp(&a_net)
    });
    sorted_comments.truncate(limit);

    Ok(sorted_comments)
}

pub fn parse_comments_from_html(document: &Html) -> Result<Vec<Comment>, Box<dyn Error>> {
    let mut comments = Vec::new();

    // XenForo comment structure selectors
    let comment_selector = Selector::parse(".message").unwrap();
    let author_selector = Selector::parse(".username").unwrap();
    let content_selector = Selector::parse(".message-content .bbWrapper").unwrap();
    let timestamp_selector =
        Selector::parse(".message-meta time, .message-attribution time, .message-date time")
            .unwrap();

    for comment_element in document.select(&comment_selector) {
        // Extract author
        let author = comment_element
            .select(&author_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Anonymous".to_string());

        // Extract content
        let mut content = comment_element
            .select(&content_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| String::new());

        // Remove "Click to expand..." text from collapsible quotes
        content = content.replace("Click to expand...", "").trim().to_string();

        // Skip empty comments
        if content.is_empty() {
            continue;
        }

        // Extract upvotes and downvotes - try multiple methods
        let upvote_selector = Selector::parse(".contentVote-score--positive").unwrap();
        let downvote_selector = Selector::parse(".contentVote-score--negative").unwrap();
        let combined_selector = Selector::parse(".contentVote-scores").unwrap();

        // Try to parse upvotes from positive score element
        let mut upvotes = comment_element
            .select(&upvote_selector)
            .next()
            .and_then(|el| el.text().collect::<String>().trim().parse::<u32>().ok())
            .unwrap_or(0);

        // Try to parse downvotes from negative score element
        // Note: downvote elements contain negative numbers (e.g., "-2"), so we need to handle the sign
        let mut downvotes = comment_element
            .select(&downvote_selector)
            .next()
            .and_then(|el| {
                let text = el.text().collect::<String>().trim().to_string();
                // Handle negative numbers by removing the minus sign
                if text.starts_with('-') {
                    text[1..].parse::<u32>().ok()
                } else {
                    text.parse::<u32>().ok()
                }
            })
            .unwrap_or(0);

        // Always try parsing from combined format as it may contain more accurate data
        if let Some(combined_element) = comment_element.select(&combined_selector).next() {
            let combined_text = combined_element.text().collect::<String>();
            // Look for pattern like "(number/number)" with flexible whitespace, newlines, and tabs
            // The format can be "(0\n\t\t\t\t\t/\n\t\t\t\t\t0)" or similar variations
            if let Some(captures) = Regex::new(r"\(\s*(\d+)\s*[/\n\t\s]*(\d+)\s*\)")
                .unwrap()
                .captures(&combined_text)
            {
                // Use the parsed values if they're higher than what we found in individual elements
                if let (Ok(combined_upvotes), Ok(combined_downvotes)) =
                    (captures[1].parse::<u32>(), captures[2].parse::<u32>())
                {
                    upvotes = std::cmp::max(upvotes, combined_upvotes);
                    downvotes = std::cmp::max(downvotes, combined_downvotes);
                }
            }
        }

        // Extract timestamp - try specific selectors first, then fallback to any time element
        let timestamp = comment_element
            .select(&timestamp_selector)
            .next()
            .and_then(|el| el.value().attr("datetime"))
            .or_else(|| {
                // Fallback to any time element if specific selectors don't match
                let fallback_selector = Selector::parse("time").unwrap();
                comment_element
                    .select(&fallback_selector)
                    .next()
                    .and_then(|el| el.value().attr("datetime"))
            })
            .map(|s| s.to_string());

        comments.push(Comment {
            content,
            author,
            upvotes,
            downvotes,
            timestamp,
        });
    }

    Ok(comments)
}

pub async fn fetch_top_5_comments(article_url: &str) -> Result<Vec<Comment>, Box<dyn Error>> {
    fetch_top_comments(article_url, 5).await
}
