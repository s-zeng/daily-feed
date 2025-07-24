use clap::Parser;
use daily_feed::ast::Document;
use daily_feed::epub_outputter::EpubOutputter;
use std::error::Error;

#[derive(Parser, Debug)]
#[clap(author = "Simon Zeng", version, about = "Convert AST JSON to EPUB")]
struct Args {
    /// Path to AST JSON file
    #[arg(short = 'i', long = "input")]
    input: String,

    /// Output EPUB filename
    #[arg(short = 'o', long = "output")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Load AST from JSON
    let json_content = std::fs::read_to_string(&args.input)?;
    let document: Document = serde_json::from_str(&json_content)?;

    println!("Loaded document: {} with {} feeds", 
             document.metadata.title, 
             document.feeds.len());

    // Generate EPUB from AST
    let mut outputter = EpubOutputter::new()?;
    outputter.generate_epub(&document, &args.output)?;

    println!("EPUB generated: {}", args.output);

    Ok(())
}