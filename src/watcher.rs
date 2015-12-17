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
    message_recipient: String,
    message_frequency: Duration,
    debug: bool,
}

impl Watcher {
    pub fn from_config(config: &Config) -> Watcher {
        Watcher {
            channels: config.server.channels.iter().cloned().collect(),
            watch_list: config.watch_list.iter().cloned().collect(),
            sent_messages: HashMap::new(),
            message_recipient: config.messaging.recipient.to_owned(),
            message_frequency: config.message_frequency,
            messaging: MessagingService::new(
                config.messaging.sid.as_ref(),
                config.messaging.token.as_ref(),
                config.messaging.number.as_ref()
            ),
            debug: false,
        }
    }

    fn handle_message(&mut self, irc: &Irc, message: &Message) {
        // If we're in debug mode, print this message to the screen no matter what it is.
        if self.debug {
            println!("{:?}", message);
        }

        // If there's no user prefix on this message, we can't determine
        // the user associated with it and there's nothing to do
        match message.prefix {
            Some(Prefix::User(ref user)) => match message.code {
                // Watched user has joined channel
                Code::Join if self.watch_list.contains(&user.nickname) => {
                    let channel = message.args.get(0)
                        .map(|s| s.as_ref())
                        .unwrap_or("unknown channel");

                    self.notify(&user.nickname, &channel);
                },

                // Bot has received private message; for right now, we're just going to respond
                // that we're AFK and call it good. Later on, we could handle these messages the
                // way we handle channel messages.
                Code::Privmsg => {
                    irc.privmsg(&user.nickname, "AFK").ok();
                },

                // This is an event code we don't cover yet
                _ => (),
            },

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
                &self.message_recipient,
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
    fn any(&mut self, irc: &Irc, event: &Event) {
        if let &Event::Message(ref message) = event {
            self.handle_message(irc, &message);
        }
    }

    fn channel_msg(&mut self, irc: &Irc, channel: &Channel, user: &ChannelUser, msg: &str) {
        // If this message is on a channel not on our join list, it's a private message and we
        // want to handle it differently, theoretically... I think we just want to print it to
        // the console right now. We could be using a hashset for this, but... Yeah. That's just
        // not going to be a significant performance boost for a bot in only a few channels.
        if !self.channels.contains(&channel.name) {
            println!("PM from {}: {}", user.nickname, msg);
        }

        // Chuck Norris crap
        if msg.starts_with(".chuck") {
            println!("{} has requested some CHUCK ACTION!", user.nickname);
            match get_awesome() {
                None => irc.privmsg(&channel.name, "Sorry, I can't think of one."),
                Some(res) => irc.privmsg(&channel.name, &res.joke),
            }
            .ok();
        }

        // Debug flag toggle
        if msg.starts_with(".debug") {
            println!("toggle debug mode");
            self.debug = !self.debug;
        }
    }

    fn ping(&mut self, irc: &Irc, server: &str) {
        irc.pong(server).ok();
    }

    fn reconnect(&mut self, _: &Irc) {
        // no idea what this needs to do here
    }

    fn welcome(&mut self, irc: &Irc) {
        // join all our channels
        for channel in &self.channels {
            irc.join(channel, None).ok();
        }
    }
}
