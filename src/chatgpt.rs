use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Req {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize, Default)]
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

pub async fn send_request(req: &Req, key: impl Into<String>) -> Result<Resp, anyhow::Error> {
    let client = reqwest::Client::new();
    let resp: Resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", key.into()))
        .json(req)
        .send()
        .await?
        .json()
        .await?;
    Ok(resp)
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[tokio::test]
//     async fn test_send_request() -> anyhow::Result<()> {
//         let resp = send_request(&Req::new("gpt-3.5-turbo", "什么是Rust?"), "xxx").await?;
//         println!("{:?}", resp.choices[0].message.content);
//         Ok(())
//     }
// }
