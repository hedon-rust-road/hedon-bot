use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SendMessageResp {
    pub code: u8,
    pub msg: String,
}
