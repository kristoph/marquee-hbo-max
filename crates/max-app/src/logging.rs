//! Prints what the app and its own crates log; the libraries beneath them are heard only when
//! something has gone wrong. `MAX_LOG` sets how much is said: off, error, warn, info or debug.

use log::{Level, LevelFilter, Log, Metadata, Record};

const LEVEL_VARIABLE: &str = "MAX_LOG";
const OWN_CRATES: [&str; 3] = ["max", "max_api", "max_media"];
const LEVEL_OF_OTHER_CRATES: LevelFilter = LevelFilter::Error;

struct Console {
    own_level: LevelFilter,
}

impl Log for Console {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let own = metadata.target().split("::").next().is_some_and(|crate_name| OWN_CRATES.contains(&crate_name));
        metadata.level() <= if own { self.own_level } else { LEVEL_OF_OTHER_CRATES.min(self.own_level) }
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        match record.level() {
            Level::Error => eprintln!("error: {}", record.args()),
            Level::Warn => eprintln!("warning: {}", record.args()),
            Level::Info | Level::Debug | Level::Trace => println!("{}", record.args()),
        }
    }

    fn flush(&self) {}
}

pub fn start() {
    let own_level = std::env::var(LEVEL_VARIABLE).ok().and_then(|level| level.parse().ok()).unwrap_or(LevelFilter::Info);
    if log::set_boxed_logger(Box::new(Console { own_level })).is_ok() {
        log::set_max_level(own_level);
    }
}
