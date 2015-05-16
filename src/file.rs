use std;
use std::io::{Write, Read};

use std::fs::OpenOptions;
use client::Key;

pub trait KeyStore {
    fn write_keys(&self, Vec<Key>) -> std::io::Result<usize>;
}

// TODO: figure out how to resolve ~/ or get the user's home directory path.
const AUTHORIZED_KEYS_PATH: &'static str = "/Users/dpetersen/.ssh/authorized_keys";

pub struct AuthorizedKeyFileStore;

impl KeyStore for AuthorizedKeyFileStore {
    // TODO look at making this not a &self method? Can you impl a trait for a struct in that way?
    fn write_keys(&self, keys: Vec<Key>) -> std::io::Result<usize> {
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
                        Err(e) => return Err(e),
                    }
                }

                Ok(written_count)
            },
            Err(e) => Err(e),
        }
    }
}
