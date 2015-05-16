#[macro_use]
extern crate log;
extern crate fern;
extern crate time;
extern crate hyper;
extern crate rustc_serialize;


use std::fs::OpenOptions;
use std::io::{Write, Read};

mod keys;

use keys::{GitHubKey, KeySource};

// TODO: figure out how to resolve ~/ or get the user's home directory path.
const AUTHORIZED_KEYS_PATH: &'static str = "/Users/dpetersen/.ssh/authorized_keys";

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

fn main() {
    init_logging();
    match write_keys(get_keys(keys::Hardcoded)) {
        Ok(count) => println!("Wrote {} key(s)!", count),
        Err(e) => panic!("There was a problem writing the keys: {}", e),
    }
}

fn get_keys<T: KeySource>(source: T) -> Vec<GitHubKey> {
    source.get_keys()
}

fn write_keys(keys: Vec<keys::GitHubKey>) -> std::io::Result<usize> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(AUTHORIZED_KEYS_PATH);

    match file {
        Ok(mut f) => {
            let mut written_count = 0;
            let mut existing_keys = String::new();
            f.read_to_string(&mut existing_keys).ok().expect("Failed reading authorized_keys!");

            for key in &keys {
                if existing_keys.contains(&key.key) {
                    debug!("Skipping key '{}', already exists", key.id);
                    continue
                }

                info!("Writing key '{}'", key.id);
                match f.write_all(&key.to_authorized_keys_line().as_bytes()) {
                    Ok(_) => written_count += 1,
                    Err(e) => return Result::Err(e),
                }
            }

            return Result::Ok(written_count)
        },
        Err(e) => return Result::Err(e),
    }
}
