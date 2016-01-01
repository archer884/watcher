use super::Watcher;

use config::ServerChannel;
use fortune_cookie;
use icndb::next as get_awesome;
use hiirc::Irc;

pub fn chuck(irc: &Irc, channel: &str, nick: &str) {
    println!("{} has requested some CHUCK ACTION!", nick);
    match get_awesome() {
        None => irc.privmsg(channel, "Sorry, I can't think of one."),
        Some(res) => irc.privmsg(channel, &res.joke),
    }
    .ok();
}

pub fn cookie(irc: &Irc, channel: &str, nick: &str) {
    println!("{} has requested a FORTUNE COOKIE", nick);
    match fortune_cookie::cookie().ok() {
        None => irc.privmsg(channel, "Man who run in front of car get tired. Man who run behind car get exhausted. You have only yourself to blame for this."),
        Some(res) => irc.privmsg(channel, &res),
    }
    .ok();
}

pub fn set_nick(watcher: &mut Watcher, irc: &Irc, nick: &str) {
    if irc.nick(nick).is_ok() {
        watcher.identity.nick = nick.to_owned();
    }
}

pub fn set_debug(watcher: &mut Watcher, enabled: bool) {
    watcher.debug = enabled;
    println!("debug mode {}", if enabled { "enabled" } else { "disabled" });
}

pub fn join_channel(watcher: &mut Watcher, irc: &Irc, channel: &str) {
    if !watcher.channels.contains_key(channel) && irc.join(channel, None).is_ok() {
        watcher.channels.insert(
            channel.to_owned(),
            ServerChannel {
                name: channel.to_owned(),
                topic: None,
                admin: false,
                log_chat: true,
                greetings: vec![],
            },
        );
    }
}

pub fn leave_channel(watcher: &mut Watcher, irc: &Irc, channel: &str) {
    if watcher.channels.contains_key(channel) && irc.part(channel, None).is_ok() {
        watcher.channels.remove(channel);
    }
}

// Watcher is unused here because currently we're just setting the topic on the server, but
// the idea is that we'll be storing the topic string as part of the ServerChannel object in
// our list of channels, so, for the future, I'm leaving the Watcher object as part of this
// function signature.
#[allow(unused)]
pub fn set_topic(watcher: &mut Watcher, irc: &Irc, channel: &str, topic: &str) {
    match irc.set_topic(channel, topic) {
        Err(e) => println!("{:?}", e),
        Ok(_) => println!("{}: {}", channel, topic),
    }
}
