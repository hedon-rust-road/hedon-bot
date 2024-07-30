use quick_xml::de::from_str;
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
    pub link: String,
    pub published: String,
    pub updated: String,
    pub summary: String,
    pub content: String,
}

impl Atom {
    pub async fn try_new(url: &str) -> anyhow::Result<Atom> {
        let data = send_request(url).await?;
        Ok(resolve_xml_data(&data)?)
    }
}

async fn send_request(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
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
        let atom = Atom::try_new(GO_BLOG_ATOM_URL).await?;
        assert_eq!(atom.title, "The Go Blog");
        Ok(())
    }

    #[test]
    fn resolve_xml_data_should_work() -> anyhow::Result<()> {
        let data = include_str!("../fixtures/atom.xml");
        let atom = resolve_xml_data(data)?;
        assert_eq!(atom.title, "The Go Blog");
        assert_eq!(atom.id, "tag:blog.golang.org,2013:blog.golang.org");
        assert_eq!(atom.updated, "2024-05-02T00:00:00+00:00");
        assert_eq!(atom.entry.len(), 2);
        Ok(())
    }
}
