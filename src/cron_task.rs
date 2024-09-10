use chrono::{FixedOffset, Local, TimeZone};
use cron_tab::AsyncCron;
use std::sync::Arc;
use tracing::info;

use crate::{
    channels::{go_blog, go_weekly, redis_blog, rust_blog, rust_inside_blog},
    conf::Conf,
    redis_base::Redis,
};

async fn run_jobs(redis: Arc<Redis>, conf: Arc<Conf>) -> anyhow::Result<()> {
    let local_tz = Local::from_offset(&FixedOffset::east_opt(8 * 3600).unwrap());
    let mut cron = AsyncCron::new(local_tz);

    let redis_clone = redis.clone();
    let conf_clone = conf.clone();
    cron.add_fn(&conf.go_weekly.cron_expression, move || {
        let redis = redis_clone.clone();
        let conf = conf_clone.clone();
        async move {
            go_weekly::send_feishu_msg(
                redis.as_ref(),
                conf.go_weekly.webhooks.clone(),
                conf.go_weekly.once_post_limit,
                conf.openai_api_key.clone(),
                conf.openai_host.clone(),
                conf.proxy.clone(),
            )
            .await
            .unwrap();
        }
    })
    .await?;
    info!("add go_weekly job");

    let redis_clone = redis.clone();
    let conf_clone = conf.clone();
    cron.add_fn(&conf.redis_official_blog.cron_expression, move || {
        let redis = redis_clone.clone();
        let conf = conf_clone.clone();
        async move {
            redis_blog::send_feishu_msg(
                redis.as_ref(),
                conf.redis_official_blog.webhooks.clone(),
                conf.redis_official_blog.once_post_limit,
                conf.openai_api_key.clone(),
                conf.openai_host.clone(),
                conf.proxy.clone(),
            )
            .await
            .unwrap();
        }
    })
    .await?;
    info!("add redis_official_blog job");

    let redis_clone = redis.clone();
    let conf_clone = conf.clone();
    cron.add_fn(&conf.go_blog.cron_expression, move || {
        let redis = redis_clone.clone();
        let conf = conf_clone.clone();
        async move {
            go_blog::send_feishu_msg(
                redis.as_ref(),
                conf.go_blog.webhooks.clone(),
                conf.go_blog.once_post_limit,
                conf.openai_api_key.clone(),
                conf.openai_host.clone(),
                conf.proxy.clone(),
            )
            .await
            .unwrap();
        }
    })
    .await?;
    info!("add go_blog job");

    let redis_clone = redis.clone();
    let conf_clone = conf.clone();
    cron.add_fn(&conf.rust_blog.cron_expression, move || {
        let redis = redis_clone.clone();
        let conf = conf_clone.clone();
        async move {
            rust_blog::send_feishu_msg(
                redis.as_ref(),
                conf.rust_blog.webhooks.clone(),
                conf.rust_blog.once_post_limit,
                conf.openai_api_key.clone(),
                conf.openai_host.clone(),
                conf.proxy.clone(),
            )
            .await
            .unwrap();
        }
    })
    .await?;
    info!("add rust_blog job");

    let redis_clone = redis.clone();
    let conf_clone = conf.clone();
    cron.add_fn(&conf.rust_inside_blog.cron_expression, move || {
        let redis = redis_clone.clone();
        let conf = conf_clone.clone();
        async move {
            rust_inside_blog::send_feishu_msg(
                redis.as_ref(),
                conf.rust_inside_blog.webhooks.clone(),
                conf.rust_inside_blog.once_post_limit,
                conf.openai_api_key.clone(),
                conf.openai_host.clone(),
                conf.proxy.clone(),
            )
            .await
            .unwrap();
        }
    })
    .await?;
    info!("add rust_inside_blog job");

    cron.start().await;

    info!("cron task started");
    Ok(())
}

pub async fn run(redis: Arc<Redis>, conf: Arc<Conf>) -> anyhow::Result<()> {
    run_jobs(redis, conf).await?;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;
    }
}
