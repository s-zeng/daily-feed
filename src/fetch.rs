use std::error::Error;

pub async fn feed_from_url(url: &str) -> Result<rss::Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = rss::Channel::read_from(&content[..])?;
    Ok(channel)
}
