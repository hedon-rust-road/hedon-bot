use tracing::{info, Level};
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub fn init_logger() {
    let info = tracing_appender::rolling::daily("logs", "info.log").with_max_level(Level::INFO);
    let error = tracing_appender::rolling::daily("logs", "error.log").with_max_level(Level::ERROR);

    tracing_subscriber::fmt()
        // TODO: Investigate why the background program does not work when using with_writer(stdout)
        .with_writer(info)
        .with_writer(error)
        .init();

    info!("hedon_bot started");
}
