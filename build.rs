extern crate inflector;
#[macro_use] extern crate log;
#[macro_use] #[no_link]
extern crate matches;
#[macro_use] extern crate nom;

#[path = "src/clientgen/mod.rs"]
mod clientgen;

use std::ffi::OsString;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    clients(&out_dir);
}

fn clients(out_dir: &OsString) {
    struct WarningLogger;
    impl log::Log for WarningLogger {
        fn enabled(&self, metadata: &log::LogMetadata) -> bool {
            metadata.level() <= log::LogLevel::Warn
        }

        fn log(&self, record: &log::LogRecord) {
            if self.enabled(record.metadata()) {
                println!("cargo:warning={}", record.args());
            }
        }
    }

    log::set_logger(|max_log_level| {
        max_log_level.set(log::LogLevelFilter::Warn);
        Box::new(WarningLogger)
    }).unwrap();

    let mut dst_file = fs::File::create(Path::new(out_dir).join("clients.rs")).unwrap();
    clientgen::generate_clients(&mut dst_file, "./CoreTweet/ApiTemplates", "./api_templates_override").unwrap();
}
