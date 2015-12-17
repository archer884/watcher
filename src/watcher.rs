use std::collections::{HashMap, HashSet};

use config::Config;
use hiirc::{Channel, ChannelUser, Code, Event, Listener, Message, Prefix, Irc};
use icndb::next as get_awesome;
use rsilio::MessagingService;
use time::{Duration, Timespec, get_time};

pub struct Watcher {
    channels: Vec<String>,
    watch_list: HashSet<String>,
    messaging: MessagingService,
    sent_messages: HashMap<String, Option<Timespec>>,
    message_frequency: Duration,
}

impl Watcher {
    pub fn from_config(config: &Config) -> Watcher {
        Watcher {
            channels: config.server.channels.iter().cloned().collect(),
            watch_list: config.watch_list.iter().cloned().collect(),
            sent_messages: HashMap::new(),
            message_frequency: config.message_frequency,
            messaging: MessagingService::new(
                config.messaging.sid.as_ref(),
                config.messaging.token.as_ref(),
                config.messaging.number.as_ref()
            ),
        }
    }

    fn handle_message(&mut self, message: &Message) {
        // If there's no user prefix on this message, we can't determine
        // the user associated with it and there's nothing to do
        match message.prefix {
            Some(Prefix::User(ref user)) => {
                if message.code == Code::Join && self.watch_list.contains(&user.nickname) {
                    let channel = message.args.get(0)
                        .map(|s| s.as_ref())
                        .unwrap_or("unknown channel");

                    self.notify(&user.nickname, &channel);
                }
            }

            // We have no other cases to handle at present, but... Whatever
            _ => (),
        }
    }

    fn notify(&mut self, nick: &str, channel: &str) -> bool {
        let entry = self.sent_messages.entry(nick.to_owned()).or_insert(None);
        let frequency = self.message_frequency;

        let can_send = entry.clone().map(|tm|
            (get_time() - tm) > frequency
        ).unwrap_or(true);

        if can_send {
            self.messaging.send_message(
                "8063416455",
                &format!("{} has joined {}", nick, channel)
            ).ok();

            *entry = Some(get_time());
            true
        } else {
            false
        }
    }
}

impl Listener for Watcher {
    #[allow(unused)]
    fn any(&mut self, _: &Irc, event: &Event) {
        if let &Event::Message(ref message) = event {
            self.handle_message(&message);
        }
    }

    fn channel_msg(&mut self, irc: &Irc, channel: &Channel, user: &ChannelUser, msg: &str) {
        // Chuck Norris crap
        if msg.starts_with(".chuck") {
            println!("{} has requested some CHUCK ACTION!", user.nickname);
            match get_awesome() {
                None => irc.privmsg(&channel.name, "Sorry, I can't think of one."),
                Some(res) => irc.privmsg(&channel.name, &res.joke),
            }
            .ok();
        }
    }

    fn ping(&mut self, irc: &Irc, server: &str) {
        // Not interested in logging this right now...
        // println!("ping received");
        irc.pong(server).ok();
    }

    fn reconnect(&mut self, _: &Irc) {
        // no idea what this needs to do here
    }

    fn welcome(&mut self, irc: &Irc) {
        // rejoin all our channels
        for channel in &self.channels {
            irc.join(channel, None).ok();
        }
    }
}
