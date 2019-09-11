use std::result;

use log::{self, Log};

use crate::result::Result;

pub fn init() -> Result<()> {
    Logger::init()?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}

/// simple basic logger
#[derive(Debug)]
struct Logger(());

const LOGGER: &Logger = &Logger(());

impl Logger {
    fn init() -> result::Result<(), log::SetLoggerError> {
        log::set_logger(LOGGER)
    }
}

impl Log for Logger {
    /// always enabled
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        eprintln!("[{}] {}", level_char(record.level()), record.args());
    }

    /// no need to flush stderr/stdout
    fn flush(&self) {}
}

#[inline(always)]
fn level_char(level: log::Level) -> char {
    match level {
        log::Level::Error => 'e',
        log::Level::Warn => 'w',
        log::Level::Info => 'i',
        log::Level::Debug => 'd',
        log::Level::Trace => 't',
    }
}
