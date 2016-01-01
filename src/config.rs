use std::fs;
use std::fs::File;
use std::io::Read;

use greetings::Greeting;
use rustc_serialize::Decodable;
use toml::{decode, Value};

#[derive(RustcDecodable)]
pub struct Bot {
    pub admin: Vec<String>,
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
    pub greetings: Vec<Greeting>,
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

#[derive(RustcDecodable)]
pub struct Logging {
    pub path: String,
}

pub struct Config {
    pub bot: Bot,
    pub server: Server,
    pub user: User,
    pub twilio: Twilio,
    pub logging: Logging,
}

#[derive(Debug)]
pub enum ConfigError {
    Unavailable, // Config file not available
    Unreadable(String), // couldn't read toml data
    BadElement(String),
    MissingElement(String),
    InvalidLoggingConfig(String), // could not create/access path
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

            let logging = try!(decode_section("logging", &table));
            if let Err(message) = validate_logging(&logging) {
                return Err(ConfigError::InvalidLoggingConfig(message));
            }

            Ok(Config {
                bot: try!(decode_section("bot", &table)),
                server: try!(decode_section("server", &table)),
                user: try!(decode_section("user", &table)),
                twilio: try!(decode_section("twilio", &table)),
                logging: logging,
            })
        }
    }
}

fn validate_logging(logging: &Logging) -> Result<(), String> {
    match fs::create_dir_all(&logging.path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{:?}", e)),
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
