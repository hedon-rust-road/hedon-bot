use crate::{
    chatgpt::build_feishu_content,
    redis_base::Redis,
    rss::{resolve_xml_data, send_request, Rss, DEFAULT_ONCE_POST_LIMIT},
    trim_str,
};
use core::fmt;
use std::vec;

use log::{info, warn};
use scraper::{Html, Selector};
use serde_json::json;

use crate::{feishu_bot, redis_base};

const GO_WEEKLY_RSS_URL: &str = "https://cprss.s3.amazonaws.com/golangweekly.com.xml";

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
    openai_api_key: Option<String>,
    openai_host: Option<String>,
    proxy: Option<String>,
) -> anyhow::Result<()> {
    info!("start fetching go weekly blogs");
    let (rss, articles) = get_rss_articles(Some(redis), once_post_limit).await?;
    let client = reqwest::Client::new();
    for wa in articles {
        if wa.articles.is_empty() {
            continue;
        }
        let content = build_content(
            wa.articles,
            openai_api_key.clone(),
            openai_host.clone(),
            proxy.clone(),
        )
        .await;
        for webhook in &webhooks {
            let res: feishu_bot::SendMessageResp = client
            .post(webhook)
            .json(&json!({
                           "msg_type": "interactive",
                           "card": {
                               "elements": [
                                    {
                                        "tag": "markdown",
                                        "content": content,
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
                warn!(
                    "fetch go weekly blogs failed, code: {}, msg: {}",
                    res.code, res.msg
                );
            }
        }
    }
    info!("finish fetching go weekly blogs");
    Ok(())
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

async fn get_rss_articles(
    redis: Option<&redis_base::Redis>,
    mut once_post_limit: u8,
) -> anyhow::Result<(Rss, Vec<WeeklyArticle>)> {
    if once_post_limit == 0 {
        once_post_limit = DEFAULT_ONCE_POST_LIMIT
    }
    let data = send_request(GO_WEEKLY_RSS_URL).await?;
    let rss = resolve_xml_data(&data)?;
    let mut articles = vec![];
    for item in &rss.channel.items {
        let arts: Vec<Article> = resolve_item_description(&item.description)
            .into_iter()
            .filter(|item| {
                if let Some(redis) = redis {
                    redis.setnx(Redis::HSET_GO_WEEKLY_KEY, &item.url)
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

async fn build_content(
    articles: Vec<Article>,
    openai_api_key: Option<String>,
    openai_host: Option<String>,
    proxy: Option<String>,
) -> String {
    let mut content = String::new();
    for (i, article) in articles.iter().enumerate() {
        content.push_str(format!("{}", article).as_str());
        if i != articles.len() - 1 {
            content.push_str("---\n");
        }
    }
    let c = build_feishu_content(
        openai_api_key,
        openai_host,
        proxy,
        build_req_content(content.clone()),
    )
    .await;
    content.push_str(&c);
    content
}

fn build_req_content(content: String) -> String {
    let mut res = String::with_capacity(content.len() + 128);
    res.push_str("这是 go weekly 本周的重点文章\n");
    res.push_str(&content);
    res.push('\n');
    res.push_str("请你使用中文每篇文章进行总结概括，不要超过100个字。\n");
    res
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
        Ok(())
    }

    #[test]
    fn test_resolve_item_description() {
        let data = include_str!("../../tests/item_description.xml");
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
