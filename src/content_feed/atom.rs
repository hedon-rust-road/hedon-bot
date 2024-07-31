use quick_xml::de::from_str;
use reqwest::{Client, Proxy};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Atom {
    pub title: String,
    pub id: String,
    pub updated: String,
    pub entry: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub title: String,
    pub id: String,
    pub link: Link,
    pub published: String,
    pub updated: String,
    pub summary: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    #[serde(rename = "@href")]
    pub href: String,
}

impl Atom {
    pub async fn try_new(url: &str, proxy: Option<String>) -> anyhow::Result<Atom> {
        let data = send_request(url, proxy).await?;
        Ok(resolve_xml_data(&data)?)
    }
}

async fn send_request(url: &str, proxy: Option<String>) -> Result<String, reqwest::Error> {
    let client: Client;
    if let Some(proxy) = proxy {
        let proxy = Proxy::https(proxy)?;
        client = Client::builder().proxy(proxy).build()?;
    } else {
        client = reqwest::Client::new();
    }
    let resp = client.get(url).send().await?.text().await?;
    Ok(resp)
}

fn resolve_xml_data(data: &str) -> Result<Atom, quick_xml::DeError> {
    let atom: Atom = from_str(data)?;
    Ok(atom)
}

#[cfg(test)]
mod tests {
    use crate::channels::go_blog::GO_BLOG_ATOM_URL;

    use super::*;

    #[tokio::test]
    async fn try_new_should_work() -> anyhow::Result<()> {
        let atom = Atom::try_new(GO_BLOG_ATOM_URL, None).await?;
        assert_eq!(atom.title, "The Go Blog");
        Ok(())
    }

    #[test]
    fn resolve_xml_data_should_work() -> anyhow::Result<()> {
        let data = include_str!("../../fixtures/atom.xml");
        let atom = resolve_xml_data(data)?;
        assert_eq!(atom.title, "The Go Blog");
        assert_eq!(atom.id, "tag:blog.golang.org,2013:blog.golang.org");
        assert_eq!(atom.updated, "2024-05-02T00:00:00+00:00");
        assert_eq!(atom.entry.len(), 2);
        assert_eq!(atom.entry[0].link.href, "https://go.dev/blog/chacha8rand");
        Ok(())
    }
}