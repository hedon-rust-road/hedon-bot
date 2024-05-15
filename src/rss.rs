use quick_xml::de::from_str;
use serde::Deserialize;

pub const DEFAULT_ONCE_POST_LIMIT: u8 = 5;

#[derive(Debug, Deserialize)]
pub struct Rss {
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

pub async fn send_request(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?.text().await?;
    Ok(resp)
}

pub fn resolve_xml_data(data: &str) -> Result<Rss, quick_xml::DeError> {
    let rss: Rss = from_str(data)?;
    Ok(rss)
}
