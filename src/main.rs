#[macro_use]
extern crate log;
extern crate fern;
extern crate time;
extern crate hyper;
extern crate rustc_serialize;
extern crate getopts;

use std::fs::OpenOptions;
use std::io::{Write, Read};
use getopts::Options;
use std::env;

mod keys;

use keys::KeySource;

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
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("r", "real", "actually hit the GitHub API. This is a dev option that I'm going to remove eventually");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    let write_result = if matches.opt_present("r") {
        write_keys(keys::GitHubAPI)
    } else {
        write_keys(keys::Hardcoded)
    };

    match write_result {
        Ok(count) => println!("Wrote {} key(s)!", count),
        Err(e) => panic!("There was a problem writing the keys: {}", e),
    }
}

fn write_keys<T: KeySource>(source: T) -> std::io::Result<usize> {
    let found_keys = source.get_keys();
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

            for key in &found_keys {
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
