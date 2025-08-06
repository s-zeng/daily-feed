use crate::ast::{Document, DocumentMetadata};
use crate::config::{Config, OutputFormat};
use crate::epub_outputter::EpubOutputter;
use crate::markdown_outputter::MarkdownOutputter;
use crate::sources::Source;
use futures;
use std::error::Error;


pub async fn document_to_epub(
    document: &Document,
    output_filename: &str,
) -> Result<(), Box<dyn Error>> {
    let mut outputter = EpubOutputter::new()?;
    outputter.generate_epub(document, output_filename)?;
    Ok(())
}

pub async fn fetch_all_sources(
    config: &Config,
) -> Result<Document, Box<dyn Error>> {
    let sources = config.get_all_sources();
    let mut feeds = Vec::new();
    
    // Create tasks for concurrent fetching
    let mut tasks = Vec::new();
    for source_entry in sources {
        let source: Box<dyn Source> = source_entry.config.clone().into();
        let name = source_entry.name().to_string();
        let title = config.output.title.clone();
        let author = config.output.author.clone();
        
        let task = async move {
            match source.fetch_document(name.clone(), title, author).await {
                Ok(document) => {
                    println!("Successfully fetched: {}", name);
                    Ok(document.feeds)
                }
                Err(e) => {
                    eprintln!("Failed to fetch {}: {}", name, e);
                    Err(e)
                }
            }
        };
        tasks.push(task);
    }
    
    // Execute all tasks concurrently
    let results = futures::future::join_all(tasks).await;
    
    // Collect successful results
    for result in results {
        if let Ok(source_feeds) = result {
            feeds.extend(source_feeds);
        }
    }

    Ok(Document {
        metadata: DocumentMetadata {
            title: config.output.title.clone(),
            author: config.output.author.clone(),
            description: None,
            generated_at: chrono::Utc::now().to_rfc3339(),
        },
        feeds,
        front_page: None,
    })
}

pub async fn document_to_output(
    document: &Document,
    output_filename: &str,
    format: &OutputFormat,
) -> Result<(), Box<dyn Error>> {
    match format {
        OutputFormat::Epub => document_to_epub(document, output_filename).await,
        OutputFormat::Markdown => {
            let outputter = MarkdownOutputter::new();
            outputter.generate_markdown(document, output_filename)?;
            Ok(())
        }
    }
}
