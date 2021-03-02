use log::LevelFilter;

#[derive(Debug)]
struct Logger;

use log::{Level, Metadata, Record};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{:5}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: &SimpleLogger = &SimpleLogger;

pub fn init() {
    if let Ok(value) = std::env::var("LSD_LOGGER") {
        match value.as_str() {
            _ => {
                println!("Logger started");
                log::set_logger(LOGGER)
                    .map(|()| log::set_max_level(LevelFilter::Debug))
                    .unwrap();
            }
        }
    }
}
