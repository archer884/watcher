use greetings::Greeting;
use serde::Deserialize;
use std::fs::File;
use std::fs;
use std::io::Read;
use toml::{self, Value};

#[derive(Deserialize)]
pub struct Bot {
    pub admin: Vec<String>,
    pub message_frequency: u64,
    pub watch_list: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub struct Server {
    pub address: String,
    pub channel: String,
    pub greetings: Vec<Greeting>,
}

#[derive(Deserialize)]
pub struct Twilio {
    pub sid: String,
    pub token: String,
    pub number: String,
    pub recipient: String,
}

#[derive(Clone, Deserialize)]
pub struct User {
    pub nick: String,
    pub user: String,
    pub real: String,
}

#[derive(Clone, Deserialize)]
pub struct Logging {
    pub path: String,
}

pub struct Config {
    pub bot: Bot,
    pub server: Server,
    pub user: User,
    pub twilio: Twilio,
    pub logging: Option<Logging>,
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

            let table: Value = data.parse()
                .map_err(|e| ConfigError::Unreadable(format!("{:?}", e)))?;

            let logging = match decode_section("logging", &table) {
                Err(ConfigError::MissingElement(_)) => None,
                Err(e) => return Err(e),
                Ok(logging) => {
                    if let Err(message) = validate_logging(&logging) {
                        return Err(ConfigError::InvalidLoggingConfig(message))
                    } else {
                        Some(logging)
                    }
                }
            };

            Ok(Config {
                bot: decode_section("bot", &table)?,
                server: decode_section("server", &table)?,
                user: decode_section("user", &table)?,
                twilio: decode_section("twilio", &table)?,
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

fn decode_section<T: Deserialize>(name: &str, table: &Value) -> Result<T, ConfigError> {
    match table.get(name) {
        None => Err(ConfigError::MissingElement(name.to_owned())),
        Some(value) => {
            value.as_str()
                .and_then(|s| toml::from_str(s).ok())
                .ok_or(ConfigError::BadElement(format!("unable to decode {:?} :: {:?}", name, table)))
        }
    }
}
