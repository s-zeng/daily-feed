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
    println!("Hello {}!", args.name.unwrap_or("world".to_string()));
    let feed_result = fetch::feed_from_url("https://feeds.arstechnica.com/arstechnica/index").await?;
    println!("{:?}", feed_result);
    Ok(())
}
