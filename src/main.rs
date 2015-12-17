extern crate getopts;
extern crate hiirc;
extern crate time;

use getopts::Options;
use hiirc::{Channel, ChannelUser, Event, Listener, Irc, ReconnectionSettings, Settings};
use time::Duration;

use std::collections::HashSet;

struct ProgramOptions {
    // names
    nick: String,
    user: String,
    real: String,

    // server
    address: String,
    channel: String,
}

struct Watcher {
    channel: String,
    watch_list: HashSet<String>
}

impl Listener for Watcher {
    fn any(&mut self, _: &Irc, event: &Event) {
        println!("{:?}", event);
    }

    fn channel_msg(&mut self, irc: &Irc, channel: &Channel, _: &ChannelUser, msg: &str) {
        println!("{}", msg);
        irc.privmsg(&channel.name, "message received").ok();
    }

    fn ping(&mut self, irc: &Irc, server: &str) {
        println!("ping received");
        irc.pong(server).ok();
    }

    fn reconnect(&mut self, _: &Irc) {
        // no idea what this needs to do here
    }

    fn welcome(&mut self, irc: &Irc) {
        irc.join(&self.channel, None).ok();
    }
}

fn main() {
    match read_options() {
        Err(message) => panic!("{}", message),
        Ok(options) => {
            Settings::new(&options.address, &options.nick)
                .username(&options.user)
                .realname(&options.real)
                .reconnection(ReconnectionSettings::Reconnect {
                    max_attempts: 5,
                    delay_between_attempts: Duration::seconds(5),
                    delay_after_disconnect: Duration::seconds(15),
                })
                .auto_ping(true)
                .dispatch(Watcher {
                    channel: options.channel.to_owned(),
                    watch_list: ["hello", "hi"].iter().map(|&s| s.to_owned()).collect()
                }).unwrap()
        }
    }
}

fn read_options() -> Result<ProgramOptions, String> {
    let mut options = Options::new();

    // names
    options.optopt("n", "nick", "nickname", "e.g. Watcher");
    options.optopt("u", "user", "username", "e.g. irc.watcher");
    options.optopt("r", "real", "realname", "Watch out for loose seal!");

    // server
    options.optopt("s", "server", "server address", "e.g. irc.freenode.whatever");
    options.optopt("c", "channel", "channel name", "e.g. #rust");

    match options.parse(std::env::args()).ok() {
        None => Err("unable to read options".to_owned()),
        Some(matches) => Ok(ProgramOptions {
            nick: matches.opt_str("n").unwrap_or("Watcher".to_owned()),
            user: matches.opt_str("u").unwrap_or("irc.watcher".to_owned()),
            real: matches.opt_str("r").unwrap_or("Watch out for loose seal!".to_owned()),
            address: try!(matches.opt_str("s").ok_or("server not provided".to_owned())),
            channel: try!(matches.opt_str("c").ok_or("channel not provided".to_owned())),
        })
    }
}
