use config::ServerChannel;
use dice::Dice;
use fortune_cookie;
use hiirc::IrcWrite;
use icndb::next as get_awesome;
use std::thread;
use watcher::{ChnHndl, IrcHndl, UsrHndl, Watcher};

const DEFAULT_CHUCK: &'static str = "No one really knows Chuck Norris. Not even Chuck Norris!";
const DEFAULT_COOKIE: &'static str = "Man who run in front of car get tired. Man who run behind \
                                      car get exhausted.";
const DEFAULT_QUOTE: &'static str = "Talk low, talk slow, and don't say too much. -John Wayne";

pub fn chuck(irc: IrcHndl, channel: ChnHndl, user: UsrHndl) {
    println!("{} has requested some CHUCK ACTION!", user.nickname());
    thread::spawn(move || {
        match get_awesome() {
            None => irc.privmsg(channel.name(), DEFAULT_CHUCK),
            Some(res) => irc.privmsg(channel.name(), &res.joke),
        }
        .ok();
    });
}

pub fn cookie(irc: IrcHndl, channel: ChnHndl, user: UsrHndl) {
    println!("{} has requested a FORTUNE COOKIE", *user.nickname());
    thread::spawn(move || {
        match fortune_cookie::cookie().ok() {
            None => irc.privmsg(channel.name(), DEFAULT_COOKIE),
            Some(res) => irc.privmsg(channel.name(), &res),
        }
        .ok();
    });
}

pub fn quote(irc: IrcHndl, channel: ChnHndl, user: UsrHndl, category: Option<String>) {
    use quote_rs::Service;
    
    println!("{} has requested a QUOTE", *user.nickname());
    thread::spawn(move || {
        let service = Service::new();
        let quote = match category {
            None => service.qod(),
            Some(ref category) => service.qod_for_category(category),
        };

        match quote {
            Err(_) => irc.privmsg(channel.name(), DEFAULT_QUOTE),
            Ok(quote) => irc.privmsg(channel.name(), &format!("{} -{}", quote.quote, quote.author)),
        }.ok()
    });
}

pub fn roll(irc: IrcHndl, channel: ChnHndl, user: UsrHndl, dice: Vec<Dice>) {
    use rand;

    println!("{} has requested DICE ROLLS: {:?}", *user.nickname(), dice);
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        let results: Vec<u32> = dice.iter().flat_map(|roll| roll.gen_result(&mut rng)).collect();
        let formatted_results = format_dice_results(&results);

        irc.privmsg(channel.name(),
                     &format!("{} rolled {} ({})",
                              *user.nickname(),
                              formatted_results,
                              results.iter().sum::<u32>()))
            .ok();
    });
}

pub fn set_nick(watcher: &mut Watcher, irc: IrcHndl, nick: &str) {
    if irc.nick(nick).is_ok() {
        watcher.identity.nick = nick.to_owned();
    }
}

pub fn set_debug(watcher: &mut Watcher, enabled: bool) {
    watcher.debug = enabled;
    println!("debug mode {}",
             if enabled { "enabled" } else { "disabled" });
}

pub fn join_channel(watcher: &mut Watcher, irc: IrcHndl, channel: &str) {
    if !watcher.channels.contains_key(channel) && irc.join(channel, None).is_ok() {
        watcher.channels.insert(channel.to_owned(),
                                ServerChannel {
                                    name: channel.to_owned(),
                                    topic: None,
                                    admin: false,
                                    log_chat: true,
                                    greetings: vec![],
                                });
    }
}

pub fn leave_channel(watcher: &mut Watcher, irc: IrcHndl, channel: &str) {
    if watcher.channels.contains_key(channel) && irc.part(channel, None).is_ok() {
        watcher.channels.remove(channel);
    }
}

// Watcher is unused here because currently we're just setting the topic on the server, but
// the idea is that we'll be storing the topic string as part of the ServerChannel object in
// our list of channels, so, for the future, I'm leaving the Watcher object as part of this
// function signature.
#[allow(unused)]
pub fn set_topic(watcher: &mut Watcher, irc: IrcHndl, channel: ChnHndl, topic: &str) {
    match irc.set_topic(channel.name(), topic) {
        Err(e) => println!("{:?}", e),
        Ok(_) => println!("{}: {}", channel.name(), topic),
    }
}

fn format_dice_results(values: &[u32]) -> String {
    use std::fmt::Write;

    if values.len() == 1 {
        return values.first().unwrap().to_string();
    }

    let mut buf = String::new();
    for (idx, &n) in values.iter().enumerate() {
        let count = values.len();
        if idx + 1 == count {
            write!(buf, "{}", n).ok();
        } else {
            write!(buf, "{}, ", n).ok();
        }
    }
    buf
}
