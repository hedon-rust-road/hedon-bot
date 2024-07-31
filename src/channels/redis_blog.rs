use std::{thread, time::Duration};

use serde_json::json;
use tracing::{error, info};

use crate::{
    chatgpt::build_feishu_content,
    content_feed::Feed,
    feishu_bot,
    redis_base::{self, Redis},
    trim_str, DEFAULT_ONCE_POST_LIMIT,
};

const REDIS_BLOG_RSS_URL: &str = "https://redis.io/blog/feed/";

#[derive(Debug)]
pub struct Article {
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
    openai_api_key: Option<String>,
    openai_host: Option<String>,
    proxy: Option<String>,
) -> anyhow::Result<()> {
    info!("start fetching redis official blogs");
    let (_, articles) = get_rss_articles(Some(redis), once_post_limit).await?;
    let client = reqwest::Client::new();
    for article in articles {
        thread::sleep(Duration::from_secs(3));
        let content = build_content(
            &article,
            openai_api_key.clone(),
            openai_host.clone(),
            proxy.clone(),
        )
        .await;
        let req = &json!({
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
                                        "content": format!("{} ({}) \n           -- {}", article.title, article.author, article.date),
                                        "tag": "plain_text"
                                },
                                "template": "red",
                        }
                }
        });
        for webhook in &webhooks {
            let res: feishu_bot::SendMessageResp =
                client.post(webhook).json(req).send().await?.json().await?;
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
) -> anyhow::Result<(Feed, Vec<Article>)> {
    if once_post_limit == 0 {
        once_post_limit = DEFAULT_ONCE_POST_LIMIT
    }
    let rss = Feed::try_new(REDIS_BLOG_RSS_URL).await?;

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
        .filter(|v| {
            if let Some(r) = redis {
                r.setnx(Redis::HSET_REDIS_BLOG_KEY, &v.url)
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

    Ok((rss, articles))
}

fn build_req_content(content: &str) -> String {
    let mut res = String::with_capacity(content.len() + 128);
    res.push_str("这是一篇文章的详细内容：\n");
    res.push_str(content);
    res.push('\n');
    res.push_str("请你使用中文对文章进行总结概括，不要超过150个字。\n");
    res.push_str("如果文中有列出参考链接的话，也请你整理并放置在回复的最下面。");
    res
}

async fn build_content(
    article: &Article,
    openai_api_key: Option<String>,
    openai_host: Option<String>,
    proxy: Option<String>,
) -> String {
    let mut content = String::with_capacity(4096);
    content.push_str(&article.description);
    content.push_str(
        &build_feishu_content(
            openai_api_key.clone(),
            openai_host.clone(),
            proxy.clone(),
            build_req_content(&article.content),
        )
        .await,
    );
    content
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
