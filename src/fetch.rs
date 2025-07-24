use crate::config::Config;
use crate::ast::Document;
use crate::parser::DocumentParser;
use crate::epub_outputter::EpubOutputter;
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

pub async fn channels_to_document(
    channels: &[(String, rss::Channel)],
    title: String,
    author: String,
) -> Result<Document, Box<dyn Error>> {
    let parser = DocumentParser::new();
    parser.parse_feeds_to_document(channels, title, author).await
}

pub async fn document_to_epub(
    document: &Document,
    output_filename: &str,
) -> Result<(), Box<dyn Error>> {
    let mut outputter = EpubOutputter::new()?;
    outputter.generate_epub(document, output_filename)?;
    Ok(())
}

