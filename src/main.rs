use clap::Parser;
mod config;
mod fetch;

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

    // Generate EPUB
    fetch::channels_to_epub(&channels, &config)?;
    println!("EPUB generated: {}", config.output.filename);

    Ok(())
}
