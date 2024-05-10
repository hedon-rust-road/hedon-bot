use std::{
    fs::{self},
    path::Path,
};

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Conf {
    pub redis: RedisConf,
    pub go_weekly: ArticleSourceConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ArticleSourceConfig {
    // sec   min   hour   day of month   month   day of week   year
    // *     *     *      *              *       *             *
    pub cron_expression: String,

    // webhoos of Feishu robots.
    pub webhooks: Vec<String>,

    // a limit on the number of articles it can push at a time
    pub once_post_limit: u8,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RedisConf {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u32,
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
                go_weekly: ArticleSourceConfig {
                    cron_expression: "0 30 10 * * *".to_string(),
                    webhooks: vec![
                        "http://example.com/webhook1".to_string(),
                        "http://example.com/webhook2".to_string()
                    ],
                    once_post_limit: 5,
                }
            }
        )
    }
}
