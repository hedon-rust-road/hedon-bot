use std::sync::Arc;

use hedon_bot::{conf::Conf, cron_task, redis_base};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();
    let conf = Conf::load("config.yml").unwrap();
    let redis = redis_base::Redis::new(
        &conf.redis.username,
        &conf.redis.password,
        &conf.redis.host,
        conf.redis.port,
    )
    .unwrap();

    let redis = Arc::new(redis);
    let conf = Arc::new(conf);
    cron_task::run_every_10_30pm(redis, conf).await?;
    Ok(())
}

fn init_logger() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap()
}
