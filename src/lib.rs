use regex::Regex;

pub mod chatgpt;
pub mod conf;
pub mod cron_task;
pub mod feishu_bot;
pub mod go_weekly;
pub mod log;
pub mod redis_base;
pub mod redis_blog;
pub mod rss;

pub fn trim_str(str: &str) -> String {
    let str = str.trim().replace(['\t', '\n'], " ");
    let str = str.replace("<p>", "");
    let str = str.replace("</p>", "");
    let re = Regex::new(r"\s+").unwrap(); // 匹配一个或多个空白字符
    re.replace_all(&str, " ").to_string() // 将匹配到的替换成单个空格
}

pub fn remove_tailing_slash(s: &str) -> &str {
    if let Some(s) = s.strip_suffix('/') {
        s
    } else {
        s
    }
}

#[cfg(test)]
mod test_trim_str {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert_eq!(trim_str(""), "");
    }

    #[test]
    fn test_only_whitespace() {
        assert_eq!(trim_str(" \t\n \t"), "");
    }

    #[test]
    fn test_leading_and_trailing_whitespace() {
        assert_eq!(trim_str(" \t\n Hello, World! \n\t "), "Hello, World!");
    }

    #[test]
    fn test_consecutive_spaces() {
        assert_eq!(trim_str("Hello    World!"), "Hello World!");
    }

    #[test]
    fn test_mixed_whitespace() {
        assert_eq!(trim_str("Hello \t\n  World!\n"), "Hello World!");
    }

    #[test]
    fn test_no_whitespace() {
        assert_eq!(trim_str("HelloWorld!"), "HelloWorld!");
    }

    #[test]
    fn test_unicode_characters() {
        assert_eq!(trim_str("   Привет\tмир  \n"), "Привет мир");
    }
}

#[cfg(test)]
mod test_remove_trailing_slash {
    use super::*;

    #[test]
    fn need_to_remove() {
        let s = "https://example.com/";
        assert_eq!(remove_tailing_slash(s), "https://example.com");
    }

    #[test]
    fn no_need_to_remove() {
        let s = "https://example.com";
        assert_eq!(remove_tailing_slash(s), "https://example.com");
    }
}
