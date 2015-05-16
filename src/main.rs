#[macro_use]
extern crate log;
extern crate fern;
extern crate time;
extern crate hyper;
extern crate rustc_serialize;

use std::fs::OpenOptions;
use std::io::{Write, Read};

use rustc_serialize::json;
use hyper::{Client, header};

// TODO: figure out how to resolve ~/ or get the user's home directory path.
const AUTHORIZED_KEYS_PATH: &'static str = "/Users/dpetersen/.ssh/authorized_keys";

#[derive(RustcDecodable, Debug)]
pub struct GitHubKey {
    id: u32,
    key: String
}

impl GitHubKey {
    pub fn to_authorized_keys_line(&self) -> String {
        // TODO hardcoded username, need to set it for the whole collection?
        return format!("{} hardcodedusername\n", self.key)
    }
}

fn init_logging() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}", time::now().strftime("%H:%M:%S").unwrap(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
}

fn main() {
    init_logging();
    match write_keys(get_hardcoded_keys()) {
        Ok(count) => println!("Wrote {} key(s)!", count),
        Err(e) => panic!("There was a problem writing the keys: {}", e),
    }
}

fn write_keys(keys: Vec<GitHubKey>) -> std::io::Result<usize> {
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

// TODO: get rid of dead_code when you start using get_keys again.
#[allow(dead_code)]
fn get_hardcoded_keys() -> Vec<GitHubKey> {
    return vec![
        GitHubKey{id: 111, key: "ssh-rsa AAAAkey111blah".to_string()},
        GitHubKey{id: 222, key: "ssh-rsa AAAAkey222blah".to_string()},
    ]
}

// TODO: get rid of dead_code when you start using it again.
#[allow(dead_code)]
fn get_keys() -> Vec<GitHubKey> {
    let mut client = Client::new();
    let mut response = client
        // TODO: hardcoded name. HTML escape!
        .get("https://api.github.com/users/dpetersen/keys")
        .header(header::UserAgent("gh-keys".to_string()))
        // TODO: Error handle
        // - User doesn't exist
        // - GitHub problem
        .send().unwrap();
    let mut body = String::new();
    // TODO: Error handle
    response.read_to_string(&mut body).ok().expect("Failed to read response!");

    // TODO: Error handle
    if response.status != hyper::Ok {
        panic!("Unxpected status {}:\n\n{}", response.status, body);
    }

    // TODO: Error handle
    return json::decode(&body).unwrap();
}
