use std::time::Duration;

use job_scheduler::{Job, JobScheduler};
use tokio::runtime::Runtime;

use crate::{conf::Conf, go_weekly, redis_base::Redis};

pub fn run_every_10_30pm(redis: &Redis, conf: &Conf) {
    let mut sched = JobScheduler::new();
    let go_weekly_conf = &conf.go_weekly;
    sched.add(Job::new(
        go_weekly_conf.cron_expression.parse().unwrap(),
        || {
            let rt = Runtime::new().unwrap();
            let _ = rt.block_on(go_weekly::send_feishu_msg(
                redis,
                go_weekly_conf.webhooks.clone(),
                go_weekly_conf.once_post_limit,
            ));
        },
    ));

    loop {
        sched.tick();
        std::thread::sleep(Duration::from_millis(500)); // 短暂休眠以减少CPU使用率
    }
}
