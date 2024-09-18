use std::{thread, time::Duration};

use serde_json::json;
use tracing::{error, info};

use crate::{
    chatgpt::build_feishu_content,
    feeds::{Atom, Entry},
    feishu_bot,
    redis_base::{self, Redis},
    DEFAULT_ONCE_POST_LIMIT,
};

pub const RUST_INSIDE_BLOG_ATOM_URL: &str = "https://blog.rust-lang.org/inside-rust/feed.xml";

pub async fn send_feishu_msg(
    redis: &redis_base::Redis,
    webhooks: Vec<String>,
    once_post_limit: u8,
    openai_api_key: Option<String>,
    openai_host: Option<String>,
    proxy: Option<String>,
) -> anyhow::Result<()> {
    info!("start fetching rust inside blogs");
    let entries = get_atom_articles(Some(redis), once_post_limit, proxy.clone()).await?;
    info!(
        "fetch rust inside blogs success, entries: {}",
        entries.len()
    );
    let client = reqwest::Client::new();
    for entry in entries {
        thread::sleep(Duration::from_secs(3));
        let content = build_content(
            &entry,
            openai_api_key.clone(),
            openai_host.clone(),
            proxy.clone(),
        )
        .await;
        info!("build rust inside blogs content success");
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
                                        "url": format!("{}", entry.link.href),
                                        "type": "default",
                                        "value": {}
                                }],
                                "tag": "action"
                             }
                        ],
                        "header": {
                                "title": {
                                        "content": format!("{} \n           -- {}", entry.title,  entry.updated),
                                        "tag": "plain_text"
                                },
                                "template": "blue",
                        }
                }
        });
        for webhook in &webhooks {
            let res: feishu_bot::SendMessageResp =
                client.post(webhook).json(req).send().await?.json().await?;
            if res.code != 0 {
                error!(
                    "send rust inside blogs to feishu failed, code: {}, msg: {}",
                    res.code, res.msg
                );
            }
        }
    }
    info!("finish fetching rust inside blogs");
    Ok(())
}

async fn get_atom_articles(
    redis: Option<&redis_base::Redis>,
    mut once_post_limit: u8,
    proxy: Option<String>,
) -> anyhow::Result<Vec<Entry>> {
    if once_post_limit == 0 {
        once_post_limit = DEFAULT_ONCE_POST_LIMIT
    }
    let atom = Atom::try_new(RUST_INSIDE_BLOG_ATOM_URL, proxy).await?;

    let entries = atom
        .entry
        .into_iter()
        .filter(|v| {
            if let Some(r) = redis {
                r.setnx(Redis::HSET_RUST_INSIDE_BLOG_KEY, &v.id)
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
        .collect::<Vec<_>>();

    Ok(entries)
}

fn build_req_content(content: &str) -> String {
    let mut res = String::with_capacity(content.len() + 128);
    res.push_str("这是 Rust inside 的一篇文章的详细内容：\n");
    res.push_str(content);
    res.push('\n');
    res.push_str("请你使用中文对文章进行总结概括，不要超过150个字。\n");
    res.push_str("如果文中有列出参考链接的话，也请你整理并放置在回复的最下面。");
    res
}

async fn build_content(
    entry: &Entry,
    openai_api_key: Option<String>,
    openai_host: Option<String>,
    proxy: Option<String>,
) -> String {
    let mut content = String::with_capacity(4096);
    content.push_str(&entry.summary);
    content.push_str(
        &build_feishu_content(
            openai_api_key.clone(),
            openai_host.clone(),
            proxy.clone(),
            build_req_content(&entry.content),
        )
        .await,
    );
    content
}
