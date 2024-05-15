use serde_json::json;
use tracing::{error, info};

use crate::{
    feishu_bot,
    redis_base::{self, Redis},
    rss::{resolve_xml_data, send_request, Rss, DEFAULT_ONCE_POST_LIMIT},
    trim_str,
};

const REDIS_BLOG_RSS_URL: &str = "https://redis.io/blog/feed/";

#[derive(Debug)]
struct Article {
    pub url: String,
    pub title: String,
    pub description: String,
    pub author: String,
    #[allow(dead_code)]
    pub content: String,
    pub date: String,
}

pub async fn send_feishu_msg(
    redis: &redis_base::Redis,
    webhooks: Vec<String>,
    once_post_limit: u8,
) -> anyhow::Result<()> {
    info!("start fetching redis official blogs");
    let (_, articles) = get_rss_articles(Some(redis), once_post_limit).await?;
    let client = reqwest::Client::new();
    for article in articles {
        for webhook in &webhooks {
            let res: feishu_bot::SendMessageResp = client
                .post(webhook)
                .json(&json!({
                               "msg_type": "interactive",
                               "card": {
                                   "elements": [
                                        {
                                            "tag": "markdown",
                                            "content": format!("{}\n\npublish date: {}", article.description, article.date)
                                        },
                                        {
                                           "actions": [{
                                                   "tag": "button",
                                                   "text": {
                                                           "content": "origin link",
                                                           "tag": "lark_md"
                                                   },
                                                   "url": format!("{}", article.url),
                                                   "type": "default",
                                                   "value": {}
                                           }],
                                           "tag": "action"
                                        }
                                   ],
                                   "header": {
                                           "title": {
                                                   "content": format!("{} ({})", article.title, article.author),
                                                   "tag": "plain_text"
                                           },
                                           "template": "red",
                                   }
                           }
                }))
                .send()
                .await?
                .json()
                .await?;

            if res.code != 0 {
                error!(
                    "send redis official blogs to feishu failed, code: {}, msg: {}",
                    res.code, res.msg
                );
            }
        }
    }
    info!("finish fetching redis official blogs");
    Ok(())
}

async fn get_rss_articles(
    redis: Option<&redis_base::Redis>,
    mut once_post_limit: u8,
) -> anyhow::Result<(Rss, Vec<Article>)> {
    if once_post_limit == 0 {
        once_post_limit = DEFAULT_ONCE_POST_LIMIT
    }
    let data = send_request(REDIS_BLOG_RSS_URL).await?;
    let rss = resolve_xml_data(&data)?;

    let articles: Vec<Article> = rss
        .channel
        .items
        .iter()
        .map(|v| Article {
            url: trim_str(&v.link),
            title: trim_str(&v.title),
            description: trim_str(&v.description),
            author: trim_str(&v.creator),
            content: trim_str(&v.content),
            date: v.pub_date.to_string(),
        })
        .take_while(|_| {
            if once_post_limit > 0 {
                once_post_limit -= 1;
                true
            } else {
                false
            }
        })
        .filter(|v| {
            if let Some(r) = redis {
                r.setnx(Redis::HSET_REDIS_BLOG_KEY, &v.url)
            } else {
                true
            }
        })
        .collect();

    Ok((rss, articles))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_get_rss_articles() -> anyhow::Result<()> {
        let (_, articles) = get_rss_articles(None, 0).await?;
        println!("articles: {:?}", articles);
        Ok(())
    }
}
