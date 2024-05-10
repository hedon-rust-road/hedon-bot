use core::fmt;
use std::vec;

use log::info;
use quick_xml::de::from_str;
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::json;
use tracing::error;

use crate::{feishu_bot, redis_base};

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

#[derive(Debug, PartialEq, Clone)]
pub struct Article {
    pub url: String,
    pub title: String,
    pub description: String,
    pub author: String,
}

#[derive(Debug, Clone)]
pub struct WeeklyArticle {
    pub date: String,
    pub articles: Vec<Article>,
}

impl fmt::Display for Article {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "**[{}]({})**: {} (_{}_)",
            self.title, self.url, self.description, self.author
        )
    }
}

pub async fn send_feishu_msg(
    redis: &redis_base::Redis,
    webhooks: Vec<String>,
    once_post_limit: u8,
) -> anyhow::Result<()> {
    info!("start fetching go weekly blogs");
    let (rss, articles) = get_rss_articles(Some(redis), once_post_limit).await?;
    let client = reqwest::Client::new();
    for wa in articles {
        if wa.articles.is_empty() {
            continue;
        }
        for webhook in &webhooks {
            let res: feishu_bot::SendMessageResp = client
            .post(webhook)
            .json(&json!({
                           "msg_type": "interactive",
                           "card": {
                               "elements": [
                                    {
                                        "tag": "markdown",
                                        "content": build_feishu_content(wa.articles.clone()),
                                    },
                                    {
                                       "actions": [{
                                               "tag": "button",
                                               "text": {
                                                       "content": "More issues",
                                                       "tag": "lark_md"
                                               },
                                               "url": "https://golangweekly.com/",
                                               "type": "default",
                                               "value": {}
                                       }],
                                       "tag": "action"
                                    }
                               ],
                               "header": {
                                       "title": {
                                               "content": format!("[{}] - {}", rss.channel.title, wa.date.replace("00:00:00 +0000", "")),
                                               "tag": "plain_text"
                                       },
                                       "template": "green",
                               }
                       }
            }))
            .send()
            .await?
            .json()
            .await?;

            if res.code != 0 {
                error!(
                    "fetch go weekly blogs failed, code: {}, msg: {}",
                    res.code, res.msg
                );
            }
        }
    }
    info!("finish fetching go weekly blogs");
    Ok(())
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
            if !content.contains("https://golangweekly.com/link/")
                || (!content.contains("1.05em") && !content.contains("1.2em"))
            {
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
            let description_node = paragraphs[0];
            let mut description = String::new();
            for node in description_node.children() {
                if node.value().is_text() {
                    if let Some(text_node) = node.value().as_text() {
                        description.push_str(text_node.trim());
                    }
                }
            }

            // Extracting author
            let author = paragraphs[1]
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();

            res.push(Article {
                url: trim_str(link),
                title: trim_str(&title),
                description: trim_str(&description),
                author: trim_str(&author),
            });
        }
    }
    res
}

fn trim_str(str: &str) -> String {
    let str = str.trim().replace(['\t', '\n'], " ");
    let re = Regex::new(r"\s+").unwrap(); // 匹配一个或多个空白字符
    re.replace_all(&str, " ").to_string() // 将匹配到的替换成单个空格
}

async fn get_rss_articles(
    redis: Option<&redis_base::Redis>,
    mut once_post_limit: u8,
) -> anyhow::Result<(Rss, Vec<WeeklyArticle>)> {
    const DEFAULT_ONCE_POST_LIMIT: u8 = 5;
    if once_post_limit == 0 {
        once_post_limit = DEFAULT_ONCE_POST_LIMIT
    }
    let data = send_request().await?;
    let rss = resolve_xml_data(&data)?;
    let mut articles = vec![];
    for item in &rss.channel.item {
        let arts: Vec<Article> = resolve_item_description(&item.description)
            .into_iter()
            .filter(|item| {
                if let Some(redis) = redis {
                    redis.setnx_go_weekly(&item.url)
                } else {
                    true
                }
            })
            .take_while(|_| {
                if once_post_limit > 0 {
                    once_post_limit -= 1;
                    true
                } else {
                    false
                }
            })
            .collect();
        let art_count = arts.len();
        articles.push(WeeklyArticle {
            date: item.pub_date.clone(),
            articles: arts,
        });
        // Push just one week at once.
        if art_count > 0 {
            break;
        }
    }
    Ok((rss, articles))
}

fn build_feishu_content(articles: Vec<Article>) -> String {
    let mut content = String::new();
    for (i, article) in articles.iter().enumerate() {
        content.push_str(format!("{}", article).as_str());
        if i != articles.len() - 1 {
            content.push_str("---\n");
        }
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_rss_articles() -> anyhow::Result<()> {
        let (rss, _) = get_rss_articles(None, 0).await?;
        assert_eq!(rss.channel.title, "Golang Weekly".to_string());
        assert_eq!(
            rss.channel.description,
            "A weekly newsletter about the Go programming language".to_string()
        );
        assert_eq!(rss.channel.link, "https://golangweekly.com/".to_string());
        Ok(())
    }

    #[test]
    fn test_resolve_item_description() {
        let data = include_str!("../tests/item_description.xml");
        let articles = resolve_item_description(data);
        assert_eq!(3, articles.len());
        assert_eq!(
            articles[0],
            Article {
                url: "https://golangweekly.com/link/154746/rss".to_string(),
                title: "Evolving the Go Standard Library with <code>math/rand/v2</code>".to_string(),
                description: "— Generating random numbers takes much more than you might think. Go’s initial RNG has multiple flaws, but fixing it breaks repeatability requirements. So, the core team created a “version 2” package that keeps Go’s compatibility promise and sets forth principles for future such 'version 2' packages generally.".to_string(),
                author: "Russ Cox (The Go Team)".to_string(),
            }
        );
        assert_eq!(
            articles[1],
            Article {
                url: "https://golangweekly.com/link/154763/rss".to_string(),
                title: "Logdy: A Web-Based Viewer for Logs".to_string(),
                description: "— Web based real-time log viewer. Stream any content to a web UI with autogenerated filters, then parse any format with TypeScript.".to_string(),
                author: "Peter Osinski".to_string(),
            }
        );
    }
}

#[cfg(test)]
mod test_trim_str {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert_eq!(trim_str(""), "");
    }

    #[test]
    fn test_only_whitespace() {
        assert_eq!(trim_str(" \t\n \t"), "");
    }

    #[test]
    fn test_leading_and_trailing_whitespace() {
        assert_eq!(trim_str(" \t\n Hello, World! \n\t "), "Hello, World!");
    }

    #[test]
    fn test_consecutive_spaces() {
        assert_eq!(trim_str("Hello    World!"), "Hello World!");
    }

    #[test]
    fn test_mixed_whitespace() {
        assert_eq!(trim_str("Hello \t\n  World!\n"), "Hello World!");
    }

    #[test]
    fn test_no_whitespace() {
        assert_eq!(trim_str("HelloWorld!"), "HelloWorld!");
    }

    #[test]
    fn test_unicode_characters() {
        assert_eq!(trim_str("   Привет\tмир  \n"), "Привет мир");
    }
}
