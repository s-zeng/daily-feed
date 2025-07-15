use clap::Parser;
mod fetch;

#[derive(Parser, Debug)]
#[clap(author = "Simon Zeng", version, about)]
/// Application configuration
struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    verbose: bool,

    /// an optional name to green
    #[arg()]
    name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }
    let channel = fetch::feed_from_url("https://feeds.arstechnica.com/arstechnica/index").await?;
    let output_path = "feed.epub";
    fetch::channel_to_epub(&channel, output_path)?;
    println!("EPUB generated: {}", output_path);
    Ok(())
}
