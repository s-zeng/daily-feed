use clap::{Parser, ValueEnum};
mod ai_client;
mod ars_comments;
mod ast;
mod config;
mod epub_outputter;
mod fetch;
mod front_page;
mod markdown_outputter;
mod parser;

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormatArg {
    Epub,
    Markdown,
}

impl From<OutputFormatArg> for config::OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Epub => config::OutputFormat::Epub,
            OutputFormatArg::Markdown => config::OutputFormat::Markdown,
        }
    }
}

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

    /// export AST to JSON file instead of generating output
    #[arg(long)]
    export_ast: Option<String>,

    /// output format (epub or markdown)
    #[arg(short = 'f', long = "format", value_enum)]
    format: Option<OutputFormatArg>,

    /// enable front page generation
    #[arg(long)]
    front_page: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }

    // Load configuration
    let mut config = match config::Config::load_from_file(&args.config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config from {}: {}", args.config_path, e);
            eprintln!("Using default configuration");
            config::Config::default()
        }
    };

    // Override output format from CLI if provided
    if let Some(format_arg) = args.format {
        config.output.format = format_arg.into();
    }

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
    let mut document = fetch::channels_to_document(
        &channels,
        config.output.title.clone(),
        config.output.author.clone(),
    )
    .await?;

    if args.verbose {
        println!(
            "Parsed {} feeds with {} total articles",
            document.feeds.len(),
            document.total_articles()
        );
    }

    // Generate front page if enabled (via CLI flag or config)
    let enable_front_page =
        args.front_page || config.front_page.as_ref().map_or(false, |fp| fp.enabled);

    if enable_front_page {
        if let Some(front_page_config) = &config.front_page {
            if args.verbose {
                println!("Generating front page...");
            }

            let provider = front_page_config.provider.clone().into();
            let front_page_generator = front_page::FrontPageGenerator::new(provider)
                .map_err(|e| format!("Failed to create front page generator: {}", e))?;

            match front_page_generator
                .generate_structured_front_page_from_document(&document)
                .await
            {
                Ok(front_page_blocks) => {
                    // Add structured front page to document
                    document.set_front_page(front_page_blocks);
                    if args.verbose {
                        println!("Structured front page generated successfully");
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to generate front page: {}", e);
                }
            }
        } else {
            eprintln!("Warning: Front page requested but no configuration found in config file");
        }
    }

    // Export AST to JSON if requested, otherwise generate output in specified format
    if let Some(ast_file) = args.export_ast {
        let json = serde_json::to_string_pretty(&document)?;
        std::fs::write(&ast_file, json)?;
        println!("AST exported to: {}", ast_file);
    } else {
        // Generate output from AST
        // Adjust filename extension based on format if not explicitly set
        let output_filename = if config.output.filename.ends_with(".epub")
            && matches!(config.output.format, config::OutputFormat::Markdown)
        {
            config.output.filename.replace(".epub", ".md")
        } else if config.output.filename.ends_with(".md")
            && matches!(config.output.format, config::OutputFormat::Epub)
        {
            config.output.filename.replace(".md", ".epub")
        } else {
            config.output.filename.clone()
        };

        fetch::document_to_output(&document, &output_filename, &config.output.format).await?;
        let format_name = match config.output.format {
            config::OutputFormat::Epub => "EPUB",
            config::OutputFormat::Markdown => "Markdown",
        };
        println!("{} generated: {}", format_name, output_filename);
    }

    Ok(())
}
