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

    match config::read_config(&std::env::args().nth(1).unwrap_or_else(|| String::from("bot.toml"))) {
        Err(e) => panic!("{:?}", e),
        Ok(ref config) => {
            let handle = run_bot(config);
            let stdin = std::io::stdin();

            for mut line in stdin.lock().lines().filter_map(|s| s.ok()) {
                match line.pop() {

                    // To be clear, what happens here is that just forwarding messages from console
                    // input. Lines beginning with `#` are sent as channel messages, while lines
                    // beginning with `r` are sent as raw IRC messages. Note: if you add a space
                    // after the format specifier `(#|r)`, that space will be included in the
                    // message as sent.
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
    use eirsee::config::Config;

    let core = Core::with_config(Config {
        user: config.user.nick.clone(),
        name: config.user.real.clone(),
        channel: config.server.channel.clone(),
    });

    core.connect(&config.server.address, Watcher::with_config(config))
}
