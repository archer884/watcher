use std::collections::{HashMap, HashSet};

use config::Messaging;
use hiirc::{Channel, ChannelUser, Event, Listener, Irc};
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
    pub fn new(channels: &[String], watch_list: &[String], messaging: &Messaging) -> Watcher {
        Watcher {
            channels: channels.iter().cloned().collect(),
            watch_list: watch_list.iter().cloned().collect(),
            sent_messages: HashMap::new(),
            message_frequency: Duration::minutes(180),
            messaging: MessagingService::new(
                messaging.sid.as_ref(),
                messaging.token.as_ref(),
                messaging.number.as_ref()
            ),
        }
    }

    fn notify(&mut self, nick: &str) -> bool {
        let entry = self.sent_messages.entry(nick.to_owned()).or_insert(None);
        let frequency = self.message_frequency;
        let can_send = entry.clone().map(|tm|
            (get_time() - tm) > frequency
        ).unwrap_or(true);

        if can_send {
            self.messaging.send_message("8063416455", &format!("heard from: {}", nick)).ok();
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
        // Don't log everything for now...
        // println!("{:?}", event);
    }

    fn channel_msg(&mut self, irc: &Irc, channel: &Channel, user: &ChannelUser, msg: &str) {
        if self.watch_list.contains(&user.nickname) {
            match self.notify(&user.nickname) {
                true => println!("Notification sent"),
                false => println!("Notification withheld"),
            }
        }

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
