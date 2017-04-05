use log;
use fern;
use chrono;

pub fn init_logging() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(move |msg: &str, level: &log::LogLevel, location: &log::LogLocation| {
            format!("[{}][{}][{}] {}", chrono::Local::now().format("%Y-%m-%d][%H:%M:%S.%f"), level, location.module_path(), msg)
        }),
        output: vec![fern::OutputConfig::file("rust-keylock.log")],
        level: log::LogLevelFilter::Debug,
    };
    let _ = fern::init_global_logger(logger_config, log::LogLevelFilter::Debug);
}
