use clap::Parser;
mod ast;
mod config;
mod fetch;
mod ars_comments;
mod parser;
mod epub_outputter;

#[derive(Parser, Debug)]
#[clap(author = "Simon Zeng", version, about)]
/// Application configuration
struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    verbose: bool,

    /// path to config file
    #[arg(short = 'c', long = "config", default_value = "config.json")]
    config_path: String,

    /// export AST to JSON file instead of generating EPUB
    #[arg(long)]
    export_ast: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }

    // Load configuration
    let config = match config::Config::load_from_file(&args.config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config from {}: {}", args.config_path, e);
            eprintln!("Using default configuration");
            config::Config::default()
        }
    };

    if args.verbose {
        println!("Loaded {} feeds", config.feeds.len());
    }

    // Fetch all RSS feeds
    let channels = fetch::fetch_all_feeds(&config).await?;

    if channels.is_empty() {
        eprintln!("No feeds were successfully fetched");
        return Ok(());
    }

    // Parse feeds into AST document
    let document = fetch::channels_to_document(
        &channels,
        config.output.title.clone(),
        config.output.author.clone(),
    ).await?;

    if args.verbose {
        println!("Parsed {} feeds with {} total articles", 
                 document.feeds.len(), 
                 document.total_articles());
    }

    // Export AST to JSON if requested, otherwise generate EPUB
    if let Some(ast_file) = args.export_ast {
        let json = serde_json::to_string_pretty(&document)?;
        std::fs::write(&ast_file, json)?;
        println!("AST exported to: {}", ast_file);
    } else {
        // Generate EPUB from AST
        fetch::document_to_epub(&document, &config.output.filename).await?;
        println!("EPUB generated: {}", config.output.filename);
    }

    Ok(())
}
