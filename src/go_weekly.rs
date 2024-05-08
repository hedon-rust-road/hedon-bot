use std::vec;

use quick_xml::de::from_str;
use scraper::{Html, Selector};
use serde::Deserialize;

const GO_WEEKLY_RSS_URL: &str = "https://cprss.s3.amazonaws.com/golangweekly.com.xml";

#[derive(Debug, Deserialize)]
pub struct Rss {
    pub channel: Channel,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub title: String,
    pub description: String,
    pub link: String,
    pub item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub title: String,
    pub link: String,
    pub description: String,
    pub guid: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
}

#[derive(Debug, Deserialize)]
pub struct ItemDescription {
    pub table: ItemDescriptionTable,
}

#[derive(Debug, Deserialize)]
pub struct ItemDescriptionTable {
    pub tr: ItemDescriptionTableTr,
}

#[derive(Debug, Deserialize)]
pub struct ItemDescriptionTableTr {
    pub td: ItemDescriptionTableTrTd,
}

#[derive(Debug, Deserialize)]
pub struct ItemDescriptionTableTrTd {
    pub div: ItemDescriptionTableTrTdDiv,
}

#[derive(Debug, Deserialize)]
pub struct ItemDescriptionTableTrTdDiv {
    pub tables: Vec<ItemDescriptionTableTrTdDivTable>,
}

#[derive(Debug, Deserialize)]
pub struct ItemDescriptionTableTrTdDivTable {
    pub tr: ItemDescriptionTableTrTdDivTableTr,
}

#[derive(Debug, Deserialize)]
pub struct ItemDescriptionTableTrTdDivTableTr {
    pub p: Vec<ItemDescriptionTableTrTdDivTableTrP>,
}

#[derive(Debug, Deserialize)]
pub struct ItemDescriptionTableTrTdDivTableTrP {
    pub p: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Article {
    url: String,
    title: String,
    description: String,
    author: String,
}

pub async fn get() -> anyhow::Result<Rss> {
    let data = send_request().await?;
    let rss = resolve_xml_data(&data)?;
    Ok(rss)
}

async fn send_request() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let resp = client.get(GO_WEEKLY_RSS_URL).send().await?.text().await?;
    Ok(resp)
}

fn resolve_xml_data(data: &str) -> Result<Rss, quick_xml::DeError> {
    let rss: Rss = from_str(data)?;
    Ok(rss)
}

#[allow(dead_code)]
fn resolve_item_description(desc: &str) -> Vec<Article> {
    let mut res = vec![];
    let document = Html::parse_document(desc);
    let table_selector = Selector::parse("table").unwrap();

    for element in document.select(&table_selector) {
        let align = element.value().attr("align").unwrap_or("");
        let border = element.value().attr("border").unwrap_or("");
        let cellpadding = element.value().attr("cellpadding").unwrap_or("");
        let cellspacing = element.value().attr("cellspacing").unwrap_or("");

        // Find <table> that matches the conditions.
        if border == "0" && cellpadding == "0" && cellspacing == "0" && align.is_empty() {
            // Filter elements that don't contains artile url.
            let content = element.html();
            if !content.contains("https://golangweekly.com/link/") || !content.contains("1.05em") {
                continue;
            }
            // Parse the article meta info.
            // <table>
            //   <td>
            //     <p>
            //       <span>
            //         <a href="{link}">{title}</a>
            //       </span>
            //       - {description}
            //     </p>
            //     <p>{author}</p>
            //   </td>
            // </table>
            let document = Html::parse_document(&content);
            let td_selector = Selector::parse("td").unwrap();
            let td_element = document.select(&td_selector).next().unwrap();

            // Extracting link and title
            let a_selector = Selector::parse("a").unwrap();
            let a_element = td_element.select(&a_selector).next().unwrap();
            let link = a_element.value().attr("href").unwrap();
            let title = a_element.inner_html();

            let p_selector = Selector::parse("p").unwrap();
            let paragraphs = td_element.select(&p_selector).collect::<Vec<_>>();
            if paragraphs.len() != 2 {
                continue;
            }

            // Extracting description
            let description = paragraphs[0].html();

            // Extracting author
            let author = paragraphs[1]
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();

            let artile = Article {
                url: link.trim().to_string(),
                title: title.trim().to_string(),
                description: description.trim().to_string(),
                author: author.trim().to_string(),
            };
            res.push(artile);
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get() -> anyhow::Result<()> {
        let rss = get().await?;
        assert_eq!(rss.channel.title, "Golang Weekly".to_string());
        assert_eq!(
            rss.channel.description,
            "A weekly newsletter about the Go programming language".to_string()
        );
        assert_eq!(rss.channel.link, "https://golangweekly.com/".to_string());
        println!("{:?}", rss.channel.item);
        Ok(())
    }

    #[test]
    fn test_resolve_item_description() {
        let data = include_str!("../tests/item_description.xml");
        let articles = resolve_item_description(data);
        assert_eq!(2, articles.len());
        // assert_eq!(
        //     articles[0],
        //     Article {
        //         url: "https://golangweekly.com/link/154763/rss".to_string(),
        //         title: "Logdy: A Web-Based Viewer for Logs".to_string(),
        //         description: "— Web based real-time log viewer. Stream any content to a web UI with autogenerated filters, then parse any format with TypeScript.".to_string(),
        //         author: "Peter Osinski".to_string(),
        //     }
        // )
    }
}
