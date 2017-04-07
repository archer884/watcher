#![feature(conservative_impl_trait, custom_derive, proc_macro, slice_patterns)]

#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate dice;
extern crate eirsee;
extern crate fortune_cookie;
extern crate icndb;
extern crate quote_rs;
extern crate rand;
extern crate regex;
extern crate rsilio;
extern crate serde;
extern crate toml;

mod command;
mod config;
mod greetings;
mod notifications;
mod watcher;

use config::Config;
use eirsee::message::OutgoingMessage;
use std::sync::mpsc;
use watcher::Watcher;

fn main() {
    use std::io::BufRead;

    match config::read_config(&std::env::args().nth(1).unwrap_or("bot.toml".to_owned())) {
        Err(e) => panic!("{:?}", e),
        Ok(ref config) => {
            let handle = run_bot(config);
            let stdin = std::io::stdin();

            for mut line in stdin.lock().lines().filter_map(|s| s.ok()) {
                match line.pop() {
                    Some('#') => handle.send(OutgoingMessage::ChannelMessage { content: line }).unwrap(),
                    Some('r') => handle.send(OutgoingMessage::Raw(line)).unwrap(),

                    _ => (), // wtf who cares.
                }
            }
        }
    }
}

fn run_bot(config: &Config) -> mpsc::Sender<OutgoingMessage> {
    use eirsee::core::Core;
    Core::new(Watcher::from_config(config)).connect(&config.server.address)
}
