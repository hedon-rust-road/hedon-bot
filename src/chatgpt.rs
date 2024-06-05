use anyhow::anyhow;
use log::warn;
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};

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

pub async fn send_request(
    req: Req,
    key: impl Into<String>,
    proxy: Option<impl Into<String>>,
) -> Result<Resp, anyhow::Error> {
    let client: Client;
    if let Some(proxy) = proxy {
        let proxy = Proxy::https(proxy.into())?;
        client = Client::builder().proxy(proxy).build()?;
    } else {
        client = reqwest::Client::new();
    }
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
        warn!(
            "Request failed with status: {} and body: {}",
            status, error_text
        );
        Err(anyhow!("{}: {}", status, error_text))
    }
}

pub async fn build_feishu_content(
    openai_api_key: Option<String>,
    proxy: Option<String>,
    content: String,
) -> String {
    if openai_api_key.is_none() {
        return "".to_string();
    }

    let openai_api_key = openai_api_key.unwrap();
    let mut res = String::with_capacity(4096);
    res.push_str("\n---\n");
    res.push_str("\n**以下内容为 OpenAI 生成，仅供参考：**\n\n");
    let req = Req::new("gpt-4o", content);
    let resp = send_request(req, openai_api_key, proxy).await;
    match resp {
        Err(e) => res.push_str(e.to_string().as_str()),
        Ok(v) => {
            if v.choices.is_empty() {
                res.push_str(format!("{:#?}", v).as_str())
            } else {
                res.push_str(&v.choices[0].message.content);
            }
        }
    }
    res.push_str("\n---\n");
    res.to_string()
}
