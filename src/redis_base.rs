use redis::{Client, Commands, RedisError, RedisResult};

pub struct Redis {
    pub client: Client,
}

impl Redis {
    pub const HSET_GO_WEEKLY_KEY: &'static str = "hedon-bot:go-weekly-memory";
    pub const HSET_REDIS_BLOG_KEY: &'static str = "hedon-bot:redis-blog-memory";
    pub const HSET_GO_BLOG_KEY: &'static str = "hedon-bot:go-blog-memory";
    pub const HSET_RUST_BLOG_KEY: &'static str = "hedon-bot:rust-blog-memory";

    pub fn new(username: &str, password: &str, host: &str, port: u32) -> anyhow::Result<Redis> {
        let client = connect_redis(username, password, host, port)?;
        Ok(Redis { client })
    }

    pub fn setnx(&self, key: &str, url: &str) -> bool {
        let conn = self.client.get_connection();
        if conn.is_err() {
            return true;
        }
        let mut conn = conn.unwrap();
        let res: Result<bool, RedisError> = conn.hset_nx(key, url, "1");
        res.unwrap_or(true)
    }

    pub fn delete(&self, key: &str, url: &str) {
        let conn = self.client.get_connection();
        if conn.is_err() {
            return;
        }
        let mut conn = conn.unwrap();
        let _res: Result<i8, RedisError> = conn.hdel(key, url);
    }

    // TODO: clear the post marker three months ago.
}

fn connect_redis(username: &str, password: &str, host: &str, mut port: u32) -> RedisResult<Client> {
    if port == 0 {
        port = 6379;
    }
    let client = redis::Client::open(format!("redis://{username}:{password}@{host}:{port}"))?;
    Ok(client)
}

#[cfg(test)]
mod tests {

    use redis::ConnectionLike;

    use super::{connect_redis, Redis};

    #[test]
    fn test_connect_redis() {
        let client = connect_redis("", "", "localhost", 6379);
        if client.is_err() {
            let err = client.err().unwrap();
            println!("new redis client error: {:?}", err);
            return;
        }
        assert!(client.is_ok());
        let client = client.unwrap();
        let conn = client.get_connection();
        if conn.is_err() {
            let err = conn.err().unwrap();
            println!("new redis connection error: {:?}", err);
            return;
        }
        let conn = conn.unwrap();
        assert!(conn.is_open())
    }

    #[test]
    fn test_setnx() {
        let redis = Redis::new("", "", "localhost", 6379);
        if redis.is_err() {
            println!("connect redis error");
            return;
        }
        assert!(redis.is_ok());
        let redis = redis.unwrap();
        if !redis.client.is_open() {
            println!("connect redis error");
            return;
        }
        redis.delete(Redis::HSET_GO_WEEKLY_KEY, "go_weekly_url1");
        assert!(redis.setnx(Redis::HSET_GO_WEEKLY_KEY, "go_weekly_url1"));
        assert!(!redis.setnx(Redis::HSET_GO_WEEKLY_KEY, "go_weekly_url1"));
    }
}
