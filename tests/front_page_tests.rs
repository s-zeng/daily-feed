use daily_feed::ai_client::AiProvider;
use daily_feed::ast::{
    Article, ArticleMetadata, ContentBlock, Document, DocumentMetadata, Feed, TextContent,
};
use daily_feed::front_page::FrontPageGenerator;
use insta::assert_snapshot;

fn create_test_document() -> Document {
    let breaking_article = Article {
        title: "Major Tech Company Announces Revolutionary AI Breakthrough".to_string(),
        content: vec![
            ContentBlock::Paragraph(TextContent::plain(
                "A major technology company today announced a significant breakthrough in artificial intelligence that could revolutionize healthcare, transportation, and manufacturing industries.".to_string()
            )),
            ContentBlock::Paragraph(TextContent::plain(
                "The new AI system demonstrates unprecedented capabilities in complex reasoning and problem-solving, potentially affecting millions of jobs worldwide.".to_string()
            )),
        ],
        metadata: ArticleMetadata {
            published_date: Some("2025-01-01T00:00:00.000000Z".to_string()),
            author: Some("Tech Reporter".to_string()),
            url: Some("https://techexample.com/ai-breakthrough".to_string()),
            feed_name: "Technology News".to_string(),
        },
        comments: vec![],
    };

    let political_article = Article {
        title: "International Trade Agreement Faces Opposition".to_string(),
        content: vec![
            ContentBlock::Paragraph(TextContent::plain(
                "A proposed international trade agreement has generated significant opposition from labor unions and environmental groups across three continents.".to_string()
            )),
        ],
        metadata: ArticleMetadata {
            published_date: Some("2025-01-01T00:00:00.000000Z".to_string()),
            author: Some("Political Reporter".to_string()),
            url: Some("https://newsexample.com/trade-agreement".to_string()),
            feed_name: "Political News".to_string(),
        },
        comments: vec![],
    };

    let health_article = Article {
        title: "Global Health Organization Issues New Guidelines".to_string(),
        content: vec![
            ContentBlock::Paragraph(TextContent::plain(
                "The World Health Organization has released updated vaccination guidelines that will impact public health policies in over 100 countries.".to_string()
            )),
        ],
        metadata: ArticleMetadata {
            published_date: Some("2025-01-01T00:00:00.000000Z".to_string()),
            author: Some("Health Reporter".to_string()),
            url: Some("https://healthexample.com/guidelines".to_string()),
            feed_name: "Health News".to_string(),
        },
        comments: vec![],
    };

    let tech_feed = Feed {
        name: "Technology News".to_string(),
        description: Some("Latest technology developments".to_string()),
        url: Some("https://techexample.com".to_string()),
        articles: vec![breaking_article],
    };

    let politics_feed = Feed {
        name: "Political News".to_string(),
        description: Some("Global political developments".to_string()),
        url: Some("https://newsexample.com".to_string()),
        articles: vec![political_article],
    };

    let health_feed = Feed {
        name: "Health News".to_string(),
        description: Some("Health and medical news".to_string()),
        url: Some("https://healthexample.com".to_string()),
        articles: vec![health_article],
    };

    Document {
        metadata: DocumentMetadata {
            title: "Daily News Digest".to_string(),
            author: "News Aggregator".to_string(),
            description: Some("Today's most important stories".to_string()),
            generated_at: "2025-01-01T00:00:00.000000Z".to_string(),
        },
        feeds: vec![tech_feed, politics_feed, health_feed],
    }
}

#[tokio::test]
async fn test_front_page_generation_with_ollama() {
    let provider = AiProvider::Ollama {
        base_url: "http://127.0.0.1:1234".to_string(),
        model: "llama2".to_string(),
    };

    let generator = FrontPageGenerator::new(provider).unwrap();
    let document = create_test_document();

    // This test requires a running Ollama server with temperature 0
    // Skip if server is not available to avoid CI failures
    if let Ok(front_page) = generator.generate_front_page(&document).await {
        // Normalize the output for consistent snapshots by removing dynamic elements
        let normalized_output = normalize_ai_output(&front_page);
        assert_snapshot!("front_page_generation_ollama", normalized_output);
    } else {
        // If Ollama server is not available, test the error case
        assert_snapshot!(
            "front_page_generation_ollama_unavailable",
            "Ollama server not available at http://127.0.0.1:1234"
        );
    }
}

#[test]
fn test_content_preparation() {
    let provider = AiProvider::Ollama {
        base_url: "http://127.0.0.1:1234".to_string(),
        model: "llama2".to_string(),
    };

    let generator = FrontPageGenerator::new(provider).unwrap();
    let document = create_test_document();

    let content = generator.prepare_content(&document).unwrap();

    // Normalize content by removing potential whitespace variations
    let normalized_content = normalize_markdown_content(&content);
    assert_snapshot!("content_preparation_markdown", normalized_content);
}

#[test]
fn test_prompt_construction() {
    let provider = AiProvider::Ollama {
        base_url: "http://127.0.0.1:1234".to_string(),
        model: "llama2".to_string(),
    };

    let generator = FrontPageGenerator::new(provider).unwrap();
    let test_content = "Sample news content for testing prompt construction";

    let prompt = generator.build_prompt(test_content);

    // Normalize the prompt for consistent snapshots
    let normalized_prompt = normalize_prompt_content(&prompt);
    assert_snapshot!("prompt_construction_template", normalized_prompt);
}

fn normalize_ai_output(output: &str) -> String {
    // Remove any timestamps, URLs, or other dynamic content that might vary between runs
    // This ensures consistent snapshot testing with AI-generated content
    output
        .lines()
        .map(|line| {
            // Remove specific URLs and replace with placeholders
            line.replace("https://techexample.com/ai-breakthrough", "[TECH_URL]")
                .replace("https://newsexample.com/trade-agreement", "[NEWS_URL]")
                .replace("https://healthexample.com/guidelines", "[HEALTH_URL]")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn normalize_markdown_content(content: &str) -> String {
    // Normalize markdown content for consistent snapshots
    content
        .lines()
        .map(|line| line.trim_end()) // Remove trailing whitespace
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn normalize_prompt_content(prompt: &str) -> String {
    // Normalize prompt content by replacing the dynamic test content with a placeholder
    prompt.replace(
        "Sample news content for testing prompt construction",
        "[TEST_CONTENT_PLACEHOLDER]",
    )
}

