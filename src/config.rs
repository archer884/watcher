use std::fs::File;
use std::io::Read;

use rustc_serialize::Decodable;
use toml::{decode, Value};

#[derive(RustcDecodable)]
pub struct Bot {
    pub admin: String,
    pub message_frequency: i64,
    pub watch_list: Vec<String>,
}

#[derive(RustcDecodable)]
pub struct Server {
    pub address: String,
    pub channels: Vec<ServerChannel>,
}

#[derive(Clone, RustcDecodable)]
pub struct ServerChannel {
    pub name: String,
    pub admin: bool,
    pub log_chat: bool,
    pub topic: Option<String>,
}

#[derive(RustcDecodable)]
pub struct Twilio {
    pub sid: String,
    pub token: String,
    pub number: String,
    pub recipient: String,
}

#[derive(Clone, RustcDecodable)]
pub struct User {
    pub nick: String,
    pub user: String,
    pub real: String,
}

pub struct Config {
    pub bot: Bot,
    pub server: Server,
    pub user: User,
    pub twilio: Twilio,
}

#[derive(Debug)]
pub enum ConfigError {
    Unavailable, // Config file not available
    Unreadable(String), // couldn't read toml data
    BadElement(String),
    MissingElement(String),
}

pub fn read_config(path: &str) -> Result<Config, ConfigError> {
    match File::open(path) {
        Err(_) => Err(ConfigError::Unavailable),
        Ok(mut file) => {
            let data = {
                let mut buf = String::new();
                file.read_to_string(&mut buf).ok();
                buf
            };

            let table: Value = try!(data.parse().map_err(|e| ConfigError::Unreadable(
                format!("{:?}", e)    
            )));

            Ok(Config {
                bot: try!(decode_section("bot", &table)),
                server: try!(decode_section("server", &table)),
                user: try!(decode_section("user", &table)),
                twilio: try!(decode_section("twilio", &table)),
            })
        }
    }
}

fn decode_section<T: Decodable>(name: &str, table: &Value) -> Result<T, ConfigError> {
    match table.lookup(name) {
        None => Err(ConfigError::MissingElement(name.to_owned())),
        Some(value) => decode(value.clone()).ok_or(ConfigError::BadElement(
            format!("unable to decode {:?} :: {:?}", name, table)
        ))
    }
}
