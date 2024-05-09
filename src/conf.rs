use std::{
    fs::{self},
    path::Path,
};

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Conf {
    pub redis: RedisConf,
    pub webhook: WebHook,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RedisConf {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct WebHook {
    pub go_weekly: Vec<String>,
}

impl Conf {
    pub fn load<P: AsRef<Path>>(p: P) -> anyhow::Result<Conf> {
        let f = fs::read_to_string(p)?;
        let conf: Conf = serde_yml::from_str(&f)?;
        Ok(conf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_conf() {
        let conf = Conf::load("./config.template.yml");
        assert!(conf.is_ok());
        let conf = conf.unwrap();
        assert_eq!(
            conf,
            Conf {
                redis: RedisConf {
                    username: "user".to_string(),
                    password: "password123".to_string(),
                    host: "localhost".to_string(),
                    port: 6379,
                },
                webhook: WebHook {
                    go_weekly: vec![
                        "http://example.com/webhook1".to_string(),
                        "http://example.com/webhook2".to_string()
                    ]
                }
            }
        )
    }
}
