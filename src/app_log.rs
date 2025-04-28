// This file is part of the `rusty-logger` project.
use env_logger;
use env_logger::fmt::Formatter;
use log::Record;
use std::io::Write;

/// Initialize log environment variables
pub fn log_env_init() {
    env_logger::builder()
        .format(log_formatter)
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .format_level(false)
        .target(env_logger::Target::Stdout)
        .init();
}

/// Logging formatter function
pub fn log_formatter(
    buf: &mut Formatter,
    record: &Record,
) -> std::result::Result<(), std::io::Error> {
    let prefix = match record.level() {
        log::Level::Error => "⛔ ".to_string(),
        log::Level::Warn => "⚠️ ".to_string(),
        _ => "".to_string(),
    };
    writeln!(buf, "{}{}", prefix, record.args())
}
