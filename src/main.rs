extern crate hiirc;
extern crate icndb;
extern crate rsilio;
extern crate time;
extern crate toml;

use hiirc::{ReconnectionSettings, Settings};
use time::Duration;

mod config;
mod notifications;
mod watcher;

use config::Config;
use watcher::Watcher;

fn main() {
    match config::read_config(&std::env::args().nth(1).unwrap_or("bot.toml".to_owned())) {
        Err(e) => panic!("{:?}", e),
        Ok(ref config) => {
            match run_bot(config) {
                Ok(_) => println!("Running..."),
                Err(e) => println!("{:?}", e),
            }
        }
    }
}

fn run_bot(config: &Config) -> Result<(), hiirc::Error> {
    Settings::new(&config.server.address, &config.user.nick)
        .username(&config.user.user)
        .realname(&config.user.real)
        .reconnection(ReconnectionSettings::Reconnect {
            max_attempts: 5,
            delay_between_attempts: Duration::seconds(5),
            delay_after_disconnect: Duration::seconds(15),
        })
        .auto_ping(true)
        .dispatch(Watcher::from_config(config))
}
