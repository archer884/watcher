use std::fs::File;
use std::io::Read;

use toml::Value;
use time::Duration;

pub struct Server {
    pub address: String,
    pub channels: Vec<String>,
}

pub struct User {
    pub nick: String,
    pub user: String,
    pub real: String,
}

pub struct Config {
    pub server: Server,
    pub user: User,
    pub twilio: Twilio,
    pub watch_list: Vec<String>,
    pub message_frequency: Duration,
}

pub struct Twilio {
    pub sid: String,
    pub token: String,
    pub number: String,
    pub recipient: String,
}

#[derive(Debug)]
pub enum ConfigError {
    Unavailable, // Config file not available
    Unreadable, // couldn't read toml data
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

            let table: Value = try!(data.parse().map_err(|_| ConfigError::Unreadable));
            Ok(Config {
                server: Server {
                    address: try!(read_string("server.address", &table)),
                    channels: try!(read_array("server.channels", &table)),
                },
                user: User {
                    nick: try!(read_string("user.nick", &table)),
                    user: try!(read_string("user.user", &table)),
                    real: try!(read_string("user.real", &table)),
                },

                // At some point, you should make this part optional; not everyone's going to
                // want to send text messages, after all...
                twilio: Twilio {
                    sid: try!(read_string("twilio.sid", &table)),
                    token: try!(read_string("twilio.token", &table)),
                    number: try!(read_string("twilio.number", &table)),
                    recipient: try!(read_string("twilio.recipient", &table)),
                },
                watch_list: try!(read_array("bot.watch_list", &table)),
                message_frequency: Duration::minutes(
                    try!(read_int("bot.message_frequency", &table))
                ),
            })
        }
    }
}

fn read_string(element: &str, table: &Value) -> Result<String, ConfigError> {
    table.lookup(element)
         .and_then(|element| element.as_str().map(|s| s.to_owned()))
         .ok_or(ConfigError::MissingElement(element.to_owned()))
}

fn read_array(element: &str, table: &Value) -> Result<Vec<String>, ConfigError> {
    table.lookup(element).and_then(|element| {
        element.as_slice().map(|slice| {
            // This collects all valid elements of the element collection but will silently drop
            // any invalid elements. I do not know what would consitute an invalid element.
            // Possibly a numeric value?
            slice.iter()
                .map(|element| element.as_str().map(|s| s.to_owned()))
                .filter_map(|s| s)
                .collect()
        })
    }).ok_or(ConfigError::MissingElement(element.to_owned()))
}

fn read_int(element: &str, table: &Value) -> Result<i64, ConfigError> {
    table.lookup(element)
        .and_then(|element| element.as_integer())
        .ok_or(ConfigError::MissingElement(element.to_owned()))
}
