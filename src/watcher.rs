use std::collections::{HashMap, HashSet};

use command::Command;
use config::{Config, ServerChannel, User};
use hiirc::{Channel, ChannelUser, Code, Event, Listener, Message, Prefix, Irc};
use icndb::next as get_awesome;
use notifications::{NotificationService, Sms};
use time::Duration;

pub struct Watcher {
    admin: String,
    identity: User,
    channels: HashMap<String, ServerChannel>,
    watch_list: HashSet<String>,
    messaging: NotificationService<Sms>,
    debug: bool,
}

impl Watcher {
    pub fn from_config(config: &Config) -> Watcher {
        Watcher {
            admin: config.bot.admin.to_owned(),
            identity: config.user.clone(),
            channels: config.server.channels.iter().cloned().map(|channel| (channel.name.to_owned(), channel)).collect(),
            watch_list: config.bot.watch_list.iter().cloned().collect(),
            messaging: create_notification_service(config),
            debug: false,
        }
    }

    fn handle_message(&mut self, irc: &Irc, message: &Message) {
        // If we're in debug mode, print this message to the screen no matter what it is.
        if self.debug {
            println!("{:?}", message);
        }

        // We're going to need the channel later on
        let channel = message.args.get(0)
            .map(|s| s.as_ref())
            .unwrap_or("unknown channel");

        // If there's no user prefix on this message, we can't determine
        // the user associated with it and there's nothing to do
        match message.prefix {
            Some(Prefix::User(ref user)) => match message.code {
                // Bot admin has joined channel
                Code::Join if self.is_admin(&user.nickname) => {
                    match irc.raw(format!("MODE {} +o {}", channel, user.nickname)) {
                        Err(e) => println!("{:?}", e),
                        Ok(_) => println!("+o {}", user.nickname),
                    }
                }

                // Any user has joined an admin channel OR a watched user has joined any channel
                Code::Join => {
                    if self.admin_channel(&channel) || self.watching(&user.nickname) {
                        self.messaging.notify_channel(&user.nickname, &channel);
                    }
                },

                // Bot has received private message; for right now, we're just going to respond
                // that we're AFK and call it good. Later on, we could handle these messages the
                // way we handle channel messages. It is unbelievably complicated to detect a pm.
                Code::Privmsg if message.args.get(0).map(|s| s.as_ref()) == Some("UnendingWatcher") => {
                    // Hack to ignore StatServ
                    if user.nickname == "StatServ" {
                        return;
                    }

                    let content = message.args.get(1).map(|s| s.as_ref()).unwrap_or("");
                    if content.starts_with(".") {
                        self.handle_command(irc, &user.nickname, &user.nickname, content);
                    } else {
                        irc.privmsg(&user.nickname, "AFK").ok();
                    }

                    // Going to try letting the user know that the bot has received a PM, just...
                    self.messaging.notify_pm(
                        &user.nickname,
                        content,
                    );
                },

                // This is an event code we don't cover yet
                _ => (),
            },

            // We have no other cases to handle at present, but... Whatever
            _ => (),
        }
    }

    fn handle_command(&mut self, irc: &Irc, channel: &str, nick: &str, command: &str) {
        if let Some(command) = command.parse::<Command>().ok() {
            match command {
                Command::Chuck => {
                    println!("{} has requested some CHUCK ACTION!", nick);
                    match get_awesome() {
                        None => irc.privmsg(channel, "Sorry, I can't think of one."),
                        Some(res) => irc.privmsg(channel, &res.joke),
                    }
                    .ok();
                }

                // Bot settings
                Command::SetNick(ref nick) => {
                    if self.is_admin(nick) && irc.nick(nick).is_ok() {
                        self.identity.nick = nick.to_owned();
                    }
                },
                Command::SetDebug(enabled) => {
                    if self.is_admin(nick) {
                        self.debug = enabled;
                        println!("debug mode {}", if enabled { "enabled" } else { "disabled" });
                    }
                },

                // Channel commands
                Command::JoinChannel(ref channel) => {
                    if self.is_admin(nick) && !self.channels.contains_key(channel) && irc.join(channel, None).is_ok() {
                        self.channels.insert(
                            channel.to_owned(),
                            ServerChannel { name: channel.to_owned(), topic: None, admin: false, log_chat: true },
                        );
                    }
                },
                Command::LeaveChannel(ref channel) => {
                    if self.is_admin(nick) && self.channels.contains_key(channel) && irc.part(channel, None).is_ok() {
                        self.channels.remove(channel);
                    }
                },

                // Admin options
                Command::SetTopic(ref topic) => {
                    if self.is_admin(nick) {
                        match irc.set_topic(channel, topic) {
                            Err(e) => println!("{:?}", e),
                            Ok(_) => println!("{}: {}", channel, topic),
                        }
                    }
                }
                Command::SetGreeting(ref greeting) => {
                    // In theory, this will be used to set the greeting the bot uses for people who enter its channel
                }
                Command::Kill => {
                    irc.close().ok();
                }
            }
        }
    }

    fn is_admin(&self, nick: &str) -> bool {
        self.admin == nick
    }

    fn admin_channel(&self, channel: &str) -> bool {
        match self.channels.get(channel) {
            None => false,
            Some(ref channel) => channel.admin
        }
    }

    fn watching(&self, nick: &str) -> bool {
        self.watch_list.contains(nick)
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
        // Log chat
        // if self.watch_list.contains(&user.nickname) || msg.contains("UnendingWatcher") {
            println!("{}: {}", user.nickname, msg);
        // }

        // Handle public chat commands
        if msg.starts_with(".") {
            self.handle_command(irc, &channel.name, &user.nickname, msg);
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
        for channel in self.channels.values() {
            irc.join(&channel.name, None).ok();
            match channel.topic {
                Some(ref topic) if channel.admin => match irc.set_topic(&channel.name, &topic) {
                    Err(e) => println!("{:?}", e),
                    Ok(_) => println!("{}: {}", channel.name, topic),
                },
                _ => ()
            }
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
        Duration::minutes(config.bot.message_frequency),
    )
}
