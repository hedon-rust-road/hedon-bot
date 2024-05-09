use std::time::{Duration, SystemTime};

use job_scheduler::Job;

pub fn run_every_10_30pm() {
    let mut scheduler = job_scheduler::JobScheduler::new();
    scheduler.add(Job::new("1/5 * * * * *".parse().unwrap(), || {
        eprintln!("{:?}", SystemTime::now());
    }));

    loop {
        scheduler.tick();
        std::thread::sleep(Duration::from_millis(500));
    }
}

// #[cfg(test)]
// mod tests {
//     use super::run_every_10_30pm;

//     #[test]
//     fn test_run_every_10_30pm() {
//         run_every_10_30pm()
//     }
// }
