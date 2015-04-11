extern crate log;

/// This is a simple implentation of a logging library for
/// the server to print to standard output using the log crate
/// from the standard library in Rust.

use self::log::{LogRecord, LogLevel, LogMetadata};

struct SwellLogger;

impl log::Log for SwellLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}

/// This initializes the logging functionality. It should only be called
/// one time in the binary.
pub fn init() -> Result<(), log::SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(log::LogLevelFilter::Info);
        Box::new(SwellLogger)
    })
}
