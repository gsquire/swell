extern crate log;

/// This is a simple implentation of a logging library for
/// the server to print to standard output using the log crate
/// from the standard library in Rust.

struct SwellLogger;

impl log::Log for SwellLogger {
    fn enabled(&self, level: log::LogLevel, _module: &str) -> bool {
        level <= log::LogLevel::Info
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.level(), record.location().module_path) {
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
