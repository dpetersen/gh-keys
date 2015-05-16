use rustc_serialize::json;
use hyper;
use hyper::{Client, header};

use std::io::Read;

#[derive(RustcDecodable, Debug)]
pub struct Key {
    pub id: u32,
    pub key: String
}

impl Key {
    pub fn to_authorized_keys_line(&self) -> String {
        // TODO hardcoded username, need to set it for the whole collection?
        return format!("{} hardcodedusername\n", self.key)
    }
}

pub trait KeySource {
    fn get_keys(&self) -> Vec<Key>;
}

pub struct Hardcoded;

impl KeySource for Hardcoded {
    fn get_keys(&self) -> Vec<Key> {
        return vec![
            Key{id: 111, key: "ssh-rsa AAAAkey111blah".to_string()},
            Key{id: 222, key: "ssh-rsa AAAAkey222blah".to_string()},
        ]
    }
}

pub struct GitHubAPI;

impl KeySource for GitHubAPI {
    fn get_keys(&self) -> Vec<Key> {
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
}
