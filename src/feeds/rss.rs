use quick_xml::de::from_str;
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct Feed {
    pub channel: Channel,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub title: String,
    pub description: String,
    #[serde(rename = "item")]
    pub items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub title: String,
    pub link: String,
    pub description: String,
    pub guid: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    #[serde(rename = "encoded", alias = "content:encoded", default)]
    pub content: String,
    #[serde(rename = "creator", alias = "dc:creator", default)]
    pub creator: String,
}

impl Feed {
    pub async fn try_new(url: &str) -> anyhow::Result<Self> {
        info!("start fetching rss, url: {}", url);
        let resp = send_request(url).await?;
        Ok(resolve_xml_data(&resp)?)
    }
}

async fn send_request(url: &str) -> Result<String, reqwest::Error> {
    info!("start sending rss request, url: {}", url);
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?.text().await?;
    info!(
        "get rss response, url: {}, resp length: {}",
        url,
        resp.len()
    );
    Ok(resp)
}

fn resolve_xml_data(data: &str) -> Result<Feed, quick_xml::DeError> {
    info!("start resolving xml data");
    let rss: Feed = from_str(data)?;
    info!("resolve xml data success");
    Ok(rss)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolve_xml_data_should_work() -> anyhow::Result<()> {
        let data = include_str!("../../fixtures/redis_feed.xml");
        let feed = resolve_xml_data(data)?;
        assert_eq!(feed.channel.title, "Redis");
        assert_eq!(feed.channel.items.len(), 12);
        Ok(())
    }
}
