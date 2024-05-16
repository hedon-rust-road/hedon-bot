use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Serialize)]
pub struct Req {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Resp {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Choice {
    pub index: i64,
    pub message: Message,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Req {
    pub fn new(model: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: vec![Message {
                role: "user".into(),
                content: message.into(),
            }],
        }
    }
}

pub async fn send_request(req: Req, key: impl Into<String>) -> Result<Resp, anyhow::Error> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", key.into()))
        .json(&req)
        .send()
        .await?;

    if resp.status().is_success() {
        let resp: Resp = resp.json().await?;
        Ok(resp)
    } else {
        let status = resp.status();
        let error_text = resp
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read response body".to_string());
        error!(
            "Request failed with status: {} and body: {}",
            status, error_text
        );
        Err(anyhow!("{}: {}", status, error_text))
    }
}

pub fn build_req_content(content: &str) -> String {
    let mut res = String::with_capacity(content.len() + 128);
    res.push_str("这事一篇文章的详细内容：\n");
    res.push_str(content);
    res.push('\n');
    res.push_str("请你使用中文对文章进行总结概括，不要超过150个字。\n");
    res.push_str("如果文中有列出参考链接的话，也请你整理并放置在回复的最下面。");
    res
}
