use tracing::{info, Level};
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub fn init_logger() {
    let info = tracing_appender::rolling::daily("logs", "info.log")
        .with_max_level(Level::INFO)
        .with_min_level(Level::INFO);
    let error = tracing_appender::rolling::daily("logs", "error.log")
        .with_max_level(Level::ERROR)
        .with_min_level(Level::ERROR);
    let stdout = std::io::stdout.with_max_level(Level::INFO);

    tracing_subscriber::fmt()
        .with_writer(stdout.and(info).and(error))
        .init();

    info!("hedon_bot started");
}
