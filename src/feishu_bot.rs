use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SendMessageResp {
    pub code: i64,
    pub msg: String,
}
