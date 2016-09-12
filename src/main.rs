#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![feature(slice_patterns)]

extern crate dice;
extern crate fortune_cookie;
extern crate hiirc;
extern crate icndb;
extern crate rand;
extern crate regex;
extern crate rsilio;
extern crate rustc_serialize;
extern crate time;
extern crate toml;

mod command;
mod config;
mod greetings;
mod notifications;
mod watcher;

use config::Config;
use hiirc::{ReconnectionSettings, Settings};
use std::time::Duration;
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
            delay_between_attempts: Duration::from_secs(5),
            delay_after_disconnect: Duration::from_secs(15),
        })
        .auto_ping(true)
        .dispatch(Watcher::from_config(config))
}
