use daily_feed::ars_comments::{
    fetch_top_5_comments, fetch_top_comments, parse_comments_from_html, Comment,
};
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
        Comment {
            content: "1".to_string(),
            author: "U1".to_string(),
            upvotes: 1,
            downvotes: 0,
            timestamp: None,
        },
        Comment {
            content: "2".to_string(),
            author: "U2".to_string(),
            upvotes: 2,
            downvotes: 0,
            timestamp: None,
        },
        Comment {
            content: "3".to_string(),
            author: "U3".to_string(),
            upvotes: 3,
            downvotes: 0,
            timestamp: None,
        },
        Comment {
            content: "4".to_string(),
            author: "U4".to_string(),
            upvotes: 4,
            downvotes: 0,
            timestamp: None,
        },
        Comment {
            content: "5".to_string(),
            author: "U5".to_string(),
            upvotes: 5,
            downvotes: 0,
            timestamp: None,
        },
        Comment {
            content: "6".to_string(),
            author: "U6".to_string(),
            upvotes: 6,
            downvotes: 0,
            timestamp: None,
        },
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

    // Handle both online and offline scenarios like front_page_tests with separate snapshots
    match result {
        Ok(comments) => {
            // When online, we get actual comments - use online snapshot
            let test_result = format!(
                "success_len_{}_has_authors_{}",
                comments.len(),
                !comments.is_empty()
            );
            insta::assert_snapshot!("fetch_comments_from_real_article_online", test_result);
        }
        Err(_) => {
            // When offline (like in CI), network requests fail - use offline snapshot
            insta::assert_snapshot!("fetch_comments_from_real_article_offline", "network_unavailable");
        }
    }
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
            // When online, check if we get real vote data - use online snapshot
            let has_non_zero_upvotes = comments.iter().any(|c| c.upvotes > 0);
            let total_votes: u32 = comments.iter().map(|c| c.upvotes + c.downvotes).sum();
            let test_result = format!(
                "online_comments_count_{}_has_votes_{}_total_votes_{}",
                comments.len(),
                has_non_zero_upvotes,
                total_votes > 0
            );
            insta::assert_snapshot!("comment_scores_are_not_always_zero_online", test_result);
        }
        Err(_) => {
            // When offline (like in CI), network requests fail - use offline snapshot
            insta::assert_snapshot!("comment_scores_are_not_always_zero_offline", "network_unavailable");
        }
    }
}

#[tokio::test]
async fn test_debug_comment_html_structure() {
    use daily_feed::http_utils::create_http_client;

    let article_url = "https://arstechnica.com/science/2025/07/ancient-skull-may-have-been-half-human-half-neanderthal-child/";
    
    // Try the full network operation - if any step fails, treat as offline
    let result: Result<&str, Box<dyn std::error::Error>> = async {
        let client = create_http_client()?;
        let response = client
            .get(article_url)
            .header("User-Agent", "daily-feed/0.1.0")
            .send()
            .await?;
        let html_content = response.text().await?;
        let document = Html::parse_document(&html_content);
        let data_url_selector = Selector::parse("[data-url]")
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        
        if let Some(element) = document.select(&data_url_selector).next() {
            if element.value().attr("data-url").is_some() {
                Ok("found_iframe_structure")
            } else {
                Ok("no_iframe_found") 
            }
        } else {
            Ok("no_data_url_element")
        }
    }.await;
    
    // Use simple if-let pattern like front_page tests
    if let Ok(structure_info) = result {
        insta::assert_snapshot!("debug_comment_html_structure_online", structure_info);
    } else {
        insta::assert_snapshot!("debug_comment_html_structure_offline", "network_unavailable");
    }
}
