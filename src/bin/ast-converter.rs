use clap::{Parser, ValueEnum};
use daily_feed::ast::Document;
use daily_feed::config::OutputFormat;
use daily_feed::epub_outputter::EpubOutputter;
use daily_feed::markdown_outputter::MarkdownOutputter;
use std::error::Error;

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormatArg {
    Epub,
    Markdown,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Epub => OutputFormat::Epub,
            OutputFormatArg::Markdown => OutputFormat::Markdown,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(
    author = "Simon Zeng",
    version,
    about = "Convert AST JSON to output format"
)]
struct Args {
    /// Path to AST JSON file
    #[arg(short = 'i', long = "input")]
    input: String,

    /// Output filename
    #[arg(short = 'o', long = "output")]
    output: String,

    /// Output format
    #[arg(short = 'f', long = "format", value_enum, default_value = "epub")]
    format: OutputFormatArg,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Load AST from JSON
    let json_content = std::fs::read_to_string(&args.input)?;
    let document: Document = serde_json::from_str(&json_content)?;

    println!(
        "Loaded document: {} with {} feeds",
        document.metadata.title,
        document.feeds.len()
    );

    // Generate output from AST
    let format: OutputFormat = args.format.into();
    match format {
        OutputFormat::Epub => {
            let mut outputter = EpubOutputter::new()?;
            outputter.generate_epub(&document, &args.output)?;
            println!("EPUB generated: {}", args.output);
        }
        OutputFormat::Markdown => {
            let outputter = MarkdownOutputter::new();
            outputter.generate_markdown(&document, &args.output)?;
            println!("Markdown generated: {}", args.output);
        }
    }

    Ok(())
}
