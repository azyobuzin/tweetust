//! build.rs test

extern crate inflector;
#[macro_use] extern crate log;
#[macro_use] #[no_link]
extern crate matches;
#[macro_use] extern crate nom;

#[path = "../../src/clientgen/mod.rs"]
mod clientgen;

use std::io;
use std::io::prelude::*;

fn main() {
    log::set_logger(|max_log_level| {
        max_log_level.set(log::LogLevelFilter::Trace);
        Box::new(StderrLogger)
    }).unwrap();

    let mut buf = Vec::new();
    let ret = clientgen::generate_clients(&mut buf, "../clientgen/CoreTweet/ApiTemplates");

    if let Err(x) = ret {
        write!(io::stderr(), "{}", x).unwrap();
    } else {
        io::stdout().write_all(&buf).unwrap();
    }
}

struct StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, _: &log::LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &log::LogRecord) {
        writeln!(io::stderr(), "{}: {}", record.level(), record.args()).unwrap();
    }
}
