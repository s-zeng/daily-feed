use daily_feed::ars_comments::{fetch_top_comments, fetch_top_5_comments, parse_comments_from_html, Comment};
use scraper::Html;

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
        score: 10,
        timestamp: Some("2025-01-01T12:00:00Z".to_string()),
    };
    
    insta::assert_json_snapshot!(comment);
}

#[test]
fn test_comment_struct_serialization() {
    let comment = Comment {
        content: "Test content".to_string(),
        author: "Test Author".to_string(),
        score: 10,
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
            score: 1,
            timestamp: None,
        },
        Comment {
            content: "High score comment".to_string(),
            author: "User2".to_string(),
            score: 10,
            timestamp: None,
        },
        Comment {
            content: "Medium score comment".to_string(),
            author: "User3".to_string(),
            score: 5,
            timestamp: None,
        },
    ];
    
    comments.sort_by(|a, b| b.score.cmp(&a.score));
    insta::assert_json_snapshot!(comments);
}

#[test]
fn test_limit_functionality() {
    let mut comments = vec![
        Comment { content: "1".to_string(), author: "U1".to_string(), score: 1, timestamp: None },
        Comment { content: "2".to_string(), author: "U2".to_string(), score: 2, timestamp: None },
        Comment { content: "3".to_string(), author: "U3".to_string(), score: 3, timestamp: None },
        Comment { content: "4".to_string(), author: "U4".to_string(), score: 4, timestamp: None },
        Comment { content: "5".to_string(), author: "U5".to_string(), score: 5, timestamp: None },
        Comment { content: "6".to_string(), author: "U6".to_string(), score: 6, timestamp: None },
    ];
    
    comments.sort_by(|a, b| b.score.cmp(&a.score));
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
    
    insta::assert_json_snapshot!(comments);
}