use quick_xml::de::from_str;
use reqwest::{Client, Proxy};
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct Atom {
    pub title: String,
    pub id: String,
    pub updated: String,
    pub entry: Vec<Entry>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Entry {
    pub title: String,
    pub id: String,
    pub link: Link,
    pub published: String,
    pub updated: String,
    #[serde(default)]
    pub summary: String,
    pub content: String,
}

#[derive(Debug, Default, Deserialize)]
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
    info!(
        "sending request to get atom data from {}, use proxy: {}",
        url,
        proxy.is_some()
    );
    let client: Client;
    if let Some(proxy) = proxy {
        let proxy = Proxy::https(proxy)?;
        client = Client::builder().proxy(proxy).build()?;
    } else {
        client = reqwest::Client::new();
    }
    let resp = client.get(url).send().await?.text().await?;
    info!("get atom data from {} success", url);
    Ok(resp)
}

fn resolve_xml_data(data: &str) -> Result<Atom, quick_xml::DeError> {
    info!("resolving xml data");
    let atom: Atom = from_str(data)?;
    info!("resolving xml data success");
    Ok(atom)
}

#[cfg(test)]
mod tests {
    use crate::channels::{
        go_blog::GO_BLOG_ATOM_URL, rust_blog::RUST_BLOG_ATOM_URL,
        rust_inside_blog::RUST_INSIDE_BLOG_ATOM_URL,
    };

    use super::*;

    #[tokio::test]
    async fn try_new_from_go_blog_should_work() -> anyhow::Result<()> {
        let atom = Atom::try_new(GO_BLOG_ATOM_URL, None).await?;
        assert_eq!(atom.title, "The Go Blog");
        Ok(())
    }

    #[test]
    fn resolve_xml_data_from_go_blog_should_work() -> anyhow::Result<()> {
        let data = include_str!("../../fixtures/atom.xml");
        let atom = resolve_xml_data(data)?;
        assert_eq!(atom.title, "The Go Blog");
        assert_eq!(atom.id, "tag:blog.golang.org,2013:blog.golang.org");
        assert_eq!(atom.updated, "2024-05-02T00:00:00+00:00");
        assert_eq!(atom.entry.len(), 2);
        assert_eq!(atom.entry[0].link.href, "https://go.dev/blog/chacha8rand");
        Ok(())
    }

    #[tokio::test]
    async fn try_new_from_rust_blog_should_work() -> anyhow::Result<()> {
        let atom = Atom::try_new(RUST_BLOG_ATOM_URL, None).await?;
        assert_eq!(atom.title, "Rust Blog");
        Ok(())
    }

    #[test]
    fn resolve_xml_data_from_rust_blog_should_work() -> anyhow::Result<()> {
        let data = include_str!("../../fixtures/rust_blog.xml");
        let atom = resolve_xml_data(data)?;
        assert_eq!(atom.title, "Rust Blog");
        assert_eq!(atom.id, "https://blog.rust-lang.org/");
        assert_eq!(atom.updated, "2024-07-29T15:38:27+00:00");
        assert_eq!(atom.entry.len(), 10);
        assert_eq!(
            atom.entry[0].link.href,
            "https://blog.rust-lang.org/2024/07/29/crates-io-development-update.html"
        );
        Ok(())
    }

    #[tokio::test]
    async fn try_new_from_rust_inside_blog_should_work() -> anyhow::Result<()> {
        let atom = Atom::try_new(RUST_INSIDE_BLOG_ATOM_URL, None).await?;
        assert_eq!(atom.title, "Inside Rust Blog");
        Ok(())
    }

    #[test]
    fn resolve_xml_data_from_rust_inside_blog_should_work() -> anyhow::Result<()> {
        let data = include_str!("../../fixtures/rust_inside_blog.xml");
        let atom = resolve_xml_data(data)?;
        assert_eq!(atom.title, "Inside Rust Blog");
        assert_eq!(atom.id, "https://blog.rust-lang.org/inside-rust/");
        assert_eq!(atom.updated, "2024-07-31T23:58:49+00:00");
        assert_eq!(atom.entry.len(), 1);
        assert_eq!(
            atom.entry[0].link.href,
            "https://blog.rust-lang.org/inside-rust/2024/08/01/welcome-tc-to-the-lang-team.html"
        );
        Ok(())
    }
}
