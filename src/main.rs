#[macro_use]
extern crate log;
extern crate fern;
extern crate time;
extern crate hyper;
extern crate rustc_serialize;
extern crate getopts;

use getopts::Options;
use std::env;

mod client;
mod file;

use client::{Key, KeySource};
use file::{AuthorizedKeyFileStore, KeyStore};

fn main() {
    init_logging();

    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("r", "real", "actually hit the GitHub API. This is a dev option that I'm going to remove eventually");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    let keys = if matches.opt_present("r") {
        keys_from_source(client::GitHubAPI)
    } else {
        keys_from_source(client::Hardcoded)
    };

    match write_keys(AuthorizedKeyFileStore, keys) {
        Ok(s) => println!("{}", s),
        Err(e) => panic!("There was a problem writing the keys: {}", e),
    }
}

fn init_logging() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}", time::now().strftime("%H:%M:%S").unwrap(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Debug,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
}

fn keys_from_source<T: KeySource>(source: T) -> Vec<Key> {
    source.get_keys()
}

fn write_keys<T: KeyStore>(store: T, keys: Vec<Key>) -> std::io::Result<String> {
    let count = try!(store.write_keys(keys));
    Ok(format!("Wrote {} key(s)!", count))
}
