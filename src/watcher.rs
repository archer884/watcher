use std::collections::HashSet;

use config::Config;
use hiirc::{Channel, ChannelUser, Code, Event, Listener, Message, Prefix, Irc};
use icndb::next as get_awesome;
use notifications::{NotificationService, Sms};

pub struct Watcher {
    channels: Vec<String>,
    watch_list: HashSet<String>,
    messaging: NotificationService<Sms>,
    debug: bool,
}

impl Watcher {
    pub fn from_config(config: &Config) -> Watcher {
        Watcher {
            channels: config.server.channels.iter().cloned().collect(),
            watch_list: config.watch_list.iter().cloned().collect(),
            messaging: create_notification_service(config),
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

                    self.messaging.notify_channel(&user.nickname, &channel);
                },

                // Bot has received private message; for right now, we're just going to respond
                // that we're AFK and call it good. Later on, we could handle these messages the
                // way we handle channel messages. It is unbelievably complicated to detect a pm.
                Code::Privmsg if message.args.get(0).map(|s| s.as_ref()) == Some("UnendingWatcher") => {
                    irc.privmsg(&user.nickname, "AFK").ok();
                },

                // This is an event code we don't cover yet
                _ => (),
            },

            // We have no other cases to handle at present, but... Whatever
            _ => (),
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
        if self.watch_list.contains(&user.nickname) {
            println!("{}: {}", user.nickname, msg);
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

fn create_notification_service(config: &Config) -> NotificationService<Sms> {
    NotificationService::new(
        Sms::new(
            config.twilio.sid.as_ref(),
            config.twilio.token.as_ref(),
            config.twilio.number.as_ref(),
        ),
        config.twilio.recipient.as_ref(),
        config.message_frequency,
    )
}
