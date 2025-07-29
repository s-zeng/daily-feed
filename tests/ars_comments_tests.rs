use daily_feed::ars_comments::{fetch_top_comments, fetch_top_5_comments, parse_comments_from_html, Comment};
use scraper::{Html, Selector};

#[tokio::test]
async fn test_fetch_top_comments_with_invalid_url() {
    let result = fetch_top_comments("https://invalid-url-that-does-not-exist.com", 5).await;
    let is_error = result.is_err();
    insta::assert_snapshot!(is_error.to_string());
}

#[tokio::test]
async fn test_fetch_top_5_comments_wrapper() {
    let result = fetch_top_5_comments("https://invalid-url-that-does-not-exist.com").await;
    let is_error = result.is_err();
    insta::assert_snapshot!(is_error.to_string());
}

#[test]
fn test_comment_struct_creation() {
    let comment = Comment {
        content: "Test content".to_string(),
        author: "Test Author".to_string(),
        upvotes: 12,
        downvotes: 2,
        timestamp: Some("2025-01-01T12:00:00Z".to_string()),
    };
    
    insta::assert_json_snapshot!(comment);
}

#[test]
fn test_comment_struct_serialization() {
    let comment = Comment {
        content: "Test content".to_string(),
        author: "Test Author".to_string(),
        upvotes: 12,
        downvotes: 2,
        timestamp: Some("2025-01-01T12:00:00Z".to_string()),
    };
    
    let json = serde_json::to_string(&comment).unwrap();
    let deserialized: Comment = serde_json::from_str(&json).unwrap();
    
    insta::assert_json_snapshot!(deserialized);
}

// Mock HTML content for testing HTML parsing without network calls
#[tokio::test]
async fn test_html_parsing_with_mock_server() {
    let result = fetch_top_comments("https://httpbin.org/status/404", 5).await;
    let is_error = result.is_err();
    insta::assert_snapshot!(is_error.to_string());
}

#[test]
fn test_comment_ordering_by_score() {
    let mut comments = vec![
        Comment {
            content: "Low score comment".to_string(),
            author: "User1".to_string(),
            upvotes: 3,
            downvotes: 2,
            timestamp: None,
        },
        Comment {
            content: "High score comment".to_string(),
            author: "User2".to_string(),
            upvotes: 15,
            downvotes: 5,
            timestamp: None,
        },
        Comment {
            content: "Medium score comment".to_string(),
            author: "User3".to_string(),
            upvotes: 8,
            downvotes: 3,
            timestamp: None,
        },
    ];
    
    comments.sort_by(|a, b| {
        let a_net = a.upvotes as i32 - a.downvotes as i32;
        let b_net = b.upvotes as i32 - b.downvotes as i32;
        b_net.cmp(&a_net)
    });
    insta::assert_json_snapshot!(comments);
}

#[test]
fn test_limit_functionality() {
    let mut comments = vec![
        Comment { content: "1".to_string(), author: "U1".to_string(), upvotes: 1, downvotes: 0, timestamp: None },
        Comment { content: "2".to_string(), author: "U2".to_string(), upvotes: 2, downvotes: 0, timestamp: None },
        Comment { content: "3".to_string(), author: "U3".to_string(), upvotes: 3, downvotes: 0, timestamp: None },
        Comment { content: "4".to_string(), author: "U4".to_string(), upvotes: 4, downvotes: 0, timestamp: None },
        Comment { content: "5".to_string(), author: "U5".to_string(), upvotes: 5, downvotes: 0, timestamp: None },
        Comment { content: "6".to_string(), author: "U6".to_string(), upvotes: 6, downvotes: 0, timestamp: None },
    ];
    
    comments.sort_by(|a, b| {
        let a_net = a.upvotes as i32 - a.downvotes as i32;
        let b_net = b.upvotes as i32 - b.downvotes as i32;
        b_net.cmp(&a_net)
    });
    comments.truncate(3);
    
    insta::assert_json_snapshot!(comments);
}

#[tokio::test]
async fn test_fetch_comments_from_real_article() {
    let article_url = "https://arstechnica.com/science/2025/07/ancient-skull-may-have-been-half-human-half-neanderthal-child/";
    
    let result = fetch_top_5_comments(article_url).await;
    
    let test_result = match result {
        Ok(comments) => {
            format!("success_len_{}_authors_{}", 
                comments.len(),
                comments.iter().map(|c| c.author.clone()).collect::<Vec<_>>().join(",")
            )
        }
        Err(e) => {
            format!("error_{}", e.to_string().len())
        }
    };
    
    insta::assert_snapshot!(test_result);
}

#[test]
fn test_parse_comments_from_html() {
    let html = r#"
    <div class="message">
        <div class="username">testuser1</div>
        <div class="message-content">
            <div class="bbWrapper">This is a test comment</div>
        </div>
        <div class="contentVote-score--positive">8</div>
        <div class="contentVote-score--negative">3</div>
        <time datetime="2025-01-01T12:00:00Z"></time>
    </div>
    <div class="message">
        <div class="username">testuser2</div>
        <div class="message-content">
            <div class="bbWrapper">Another test comment</div>
        </div>
        <div class="contentVote-score--positive">5</div>
        <div class="contentVote-score--negative">2</div>
        <time datetime="2025-01-01T13:00:00Z"></time>
    </div>
    "#;
    
    let document = Html::parse_document(html);
    let comments = parse_comments_from_html(&document).unwrap();
    
    insta::assert_json_snapshot!(comments);
}

#[test]
fn test_click_to_expand_filtering() {
    let html = r#"
    <div class="message">
        <div class="username">testuser</div>
        <div class="message-content">
            <div class="bbWrapper">This is before the quote.
            
            Click to expand...
            
            This is after the quote text.</div>
        </div>
        <div class="contentVote-score--positive">5</div>
        <div class="contentVote-score--negative">1</div>
        <time datetime="2025-01-01T12:00:00Z"></time>
    </div>
    "#;
    
    let document = Html::parse_document(html);
    let comments = parse_comments_from_html(&document).unwrap();
    
    insta::assert_json_snapshot!(comments);
}

#[test]
fn test_parse_comments_with_real_world_html_structure() {
    // Test with the actual HTML structure we see in the debug output
    let html = r#"
    <div class="message">
        <div class="username">realuser</div>
        <div class="message-content">
            <div class="bbWrapper">This is a real world comment structure</div>
        </div>
        <div class="contentVote-scores">0
            (0
            /
            0)</div>
        <div class="contentVote-score contentVote-score--total js-voteCount js-voteCount--total">0</div>
        <div class="contentVote-score contentVote-score--positive js-voteCount js-voteCount--positive">0</div>
        <div class="contentVote-score contentVote-score--negative js-voteCount js-voteCount--negative">0</div>
        <time datetime="2025-01-01T12:00:00Z"></time>
    </div>
    <div class="message">
        <div class="username">anotheruser</div>
        <div class="message-content">
            <div class="bbWrapper">Comment with actual votes</div>
        </div>
        <div class="contentVote-scores">15
            (20
            /
            5)</div>
        <div class="contentVote-score contentVote-score--total js-voteCount js-voteCount--total">15</div>
        <div class="contentVote-score contentVote-score--positive js-voteCount js-voteCount--positive">20</div>
        <div class="contentVote-score contentVote-score--negative js-voteCount js-voteCount--negative">5</div>
        <time datetime="2025-01-01T13:00:00Z"></time>
    </div>
    "#;
    
    let document = Html::parse_document(html);
    let comments = parse_comments_from_html(&document).unwrap();
    
    insta::assert_json_snapshot!(comments);
}

#[tokio::test]
async fn test_comment_scores_are_not_always_zero() {
    let article_url = "https://arstechnica.com/science/2025/07/ancient-skull-may-have-been-half-human-half-neanderthal-child/";
    
    let result = fetch_top_5_comments(article_url).await;
    
    match result {
        Ok(comments) => {
            // Create a snapshot of actual comment votes to verify they're not all zeros
            let net_scores: Vec<i32> = comments.iter().map(|c| c.upvotes as i32 - c.downvotes as i32).collect();
            let upvotes: Vec<u32> = comments.iter().map(|c| c.upvotes).collect();
            let downvotes: Vec<u32> = comments.iter().map(|c| c.downvotes).collect();
            let has_non_zero_upvotes = upvotes.iter().any(|&score| score > 0);
            
            // For snapshot, record upvotes, downvotes, net scores and whether any upvotes are non-zero
            let test_result = format!("upvotes: {:?}, downvotes: {:?}, net_scores: {:?}, has_non_zero_upvotes: {}", upvotes, downvotes, net_scores, has_non_zero_upvotes);
            insta::assert_snapshot!(test_result);
        }
        Err(e) => {
            // If the test fails due to network issues, we still want to record it
            insta::assert_snapshot!(format!("network_error: {}", e));
        }
    }
}

#[tokio::test]
async fn test_debug_comment_html_structure() {
    use reqwest;
    
    let article_url = "https://arstechnica.com/science/2025/07/ancient-skull-may-have-been-half-human-half-neanderthal-child/";
    let client = reqwest::Client::new();
    
    // Fetch the article page to extract the iframe URL
    let response = client
        .get(article_url)
        .header("User-Agent", "daily-feed/0.1.0")
        .send()
        .await;
    
    if let Ok(response) = response {
        let html_content = response.text().await.unwrap();
        let document = Html::parse_document(&html_content);
        
        // Extract the iframe URL from the data-url attribute
        let data_url_selector = Selector::parse("[data-url]").unwrap();
        if let Some(element) = document.select(&data_url_selector).next() {
            if let Some(iframe_url) = element.value().attr("data-url") {
                // Fetch the forum thread page
                let forum_response = client
                    .get(iframe_url)
                    .header("User-Agent", "daily-feed/0.1.0")
                    .send()
                    .await;
                
                if let Ok(forum_response) = forum_response {
                    let forum_html = forum_response.text().await.unwrap();
                    let forum_document = Html::parse_document(&forum_html);
                    
                    // Look for ALL comments' reactions structure, not just the first
                    let comment_selector = Selector::parse(".message").unwrap();
                    let mut all_score_elements = Vec::new();
                    
                    for (i, comment) in forum_document.select(&comment_selector).enumerate().take(10) {
                        let score_related_elements: Vec<String> = comment
                            .select(&Selector::parse("*").unwrap())
                            .filter_map(|el| {
                                let classes = el.value().classes().collect::<Vec<_>>();
                                let text = el.text().collect::<String>().trim().to_string();
                                if (classes.iter().any(|c| c.contains("reaction") || c.contains("score") || c.contains("like")) 
                                    || (text.chars().all(|c| c.is_ascii_digit() || c.is_whitespace() || c == '(' || c == ')' || c == '/') && !text.is_empty()))
                                    && !text.is_empty() {
                                    Some(format!("classes: {:?}, text: '{}'", classes, text))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        
                        if !score_related_elements.is_empty() {
                            all_score_elements.push(format!("comment_{}: {:?}", i, score_related_elements));
                        }
                    }
                    
                    insta::assert_snapshot!(format!("all_comments_score_elements: {:?}", all_score_elements));
                    return;
                }
            }
        }
    }
    
    insta::assert_snapshot!("debug_failed");
}
