use std::time::Duration;

use job_scheduler::{Job, JobScheduler};
use log::info;
use tokio::runtime::Runtime;

use crate::{conf::Conf, go_weekly, redis_base::Redis, redis_blog};

pub fn run_every_10_30pm(redis: &Redis, conf: &Conf) {
    let mut sched = JobScheduler::new();
    let go_weekly_conf = &conf.go_weekly;
    sched.add(Job::new(
        go_weekly_conf.cron_expression.parse().unwrap(),
        || {
            info!("Run go_weekly");
            let rt = Runtime::new().unwrap();
            let _ = rt.block_on(go_weekly::send_feishu_msg(
                redis,
                go_weekly_conf.webhooks.clone(),
                go_weekly_conf.once_post_limit,
                conf.openai_api_key.clone(),
                conf.proxy.clone(),
            ));
        },
    ));

    info!("add go_weekly job");

    let redis_official_blog_conf = &conf.redis_official_blog;
    sched.add(Job::new(
        redis_official_blog_conf.cron_expression.parse().unwrap(),
        || {
            info!("Run redis_official_blog");
            let rt = Runtime::new().unwrap();
            let _ = rt.block_on(redis_blog::send_feishu_msg(
                redis,
                redis_official_blog_conf.webhooks.clone(),
                redis_official_blog_conf.once_post_limit,
                conf.openai_api_key.clone(),
                conf.proxy.clone(),
            ));
        },
    ));

    info!("add redis_official_blog job");

    loop {
        sched.tick();
        std::thread::sleep(Duration::from_millis(500)); // 短暂休眠以减少CPU使用率
    }
}
