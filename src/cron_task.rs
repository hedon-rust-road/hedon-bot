use log::{info, warn};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{conf::Conf, go_weekly, redis_base::Redis, redis_blog};

pub async fn run_every_10_30pm(redis: Arc<Redis>, conf: Arc<Conf>) -> anyhow::Result<()> {
    let sched = JobScheduler::new().await?;

    let go_weekly_conf = Arc::new(conf.go_weekly.clone());
    let go_weekly_job = Job::new_async(go_weekly_conf.cron_expression.as_str(), {
        let redis = Arc::clone(&redis);
        let conf = Arc::clone(&conf);
        let go_weekly_conf = Arc::clone(&go_weekly_conf);
        move |uuid, mut l| {
            let redis = Arc::clone(&redis);
            let conf = Arc::clone(&conf);
            let go_weekly_conf = Arc::clone(&go_weekly_conf);
            Box::pin(async move {
                if let Ok(Some(ts)) = l.next_tick_for_job(uuid).await {
                    info!("Run go_weekly {}", ts);
                    if let Err(e) = go_weekly::send_feishu_msg(
                        &redis,
                        go_weekly_conf.webhooks.clone(),
                        go_weekly_conf.once_post_limit,
                        conf.openai_api_key.clone(),
                        conf.proxy.clone(),
                    )
                    .await
                    {
                        warn!("go_weekly error: {:?}", e);
                    }
                }
            })
        }
    })?;

    sched.add(go_weekly_job).await?;
    info!("add go_weekly job");

    let redis_official_blog_conf = Arc::new(conf.redis_official_blog.clone());
    let redis_job = Job::new_async(redis_official_blog_conf.cron_expression.as_str(), {
        let redis = Arc::clone(&redis);
        let conf = Arc::clone(&conf);
        let redis_official_blog_conf = Arc::clone(&redis_official_blog_conf);
        move |uuid, mut l| {
            let redis = Arc::clone(&redis);
            let conf = Arc::clone(&conf);
            let redis_official_blog_conf = Arc::clone(&redis_official_blog_conf);
            Box::pin(async move {
                if let Ok(Some(ts)) = l.next_tick_for_job(uuid).await {
                    info!("Run redis_official_blog {}", ts);
                    if let Err(e) = redis_blog::send_feishu_msg(
                        &redis,
                        redis_official_blog_conf.webhooks.clone(),
                        redis_official_blog_conf.once_post_limit,
                        conf.openai_api_key.clone(),
                        conf.proxy.clone(),
                    )
                    .await
                    {
                        warn!("redis_official_blog error: {:?}", e);
                    }
                }
            })
        }
    })?;
    sched.add(redis_job).await?;
    info!("add redis_official_blog job");

    sched.start().await?;
    info!("start scheduler");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;
    }
}
