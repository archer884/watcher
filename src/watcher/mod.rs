mod commands;
mod responder;

use command::Command;
use config::{Config, User, Server};
use eirsee::message::OutgoingMessage;
use greetings::Greeting;
use notifications::{NotificationService, Sms};
use std::cell::Cell;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct Watcher {
    admin: HashSet<String>,
    identity: User,
    server: Server,
    greetings: Vec<Greeting>,
    watch_list: HashSet<String>,
    messaging: RwLock<NotificationService<Sms>>,
    log_path: Option<String>,
    admin_mode: bool,
    debug: Cell<bool>,
}

impl Watcher {
    pub fn from_config(config: &Config) -> Watcher {
        Watcher {
            admin: config.bot.admin.iter().cloned().collect(),
            identity: config.user.clone(),
            server: config.server.clone(),
            greetings: config.server.greetings.clone(),
            watch_list: config.bot.watch_list.iter().cloned().collect(),
            messaging: RwLock::new(create_notification_service(config)),
            log_path: config.logging.clone().map(|logging| logging.path),
            // FIXME: this should be set in the config file somewhere.
            admin_mode: true,
            debug: Cell::new(true),
        }
    }

    // TODO: change this so that we're not *just* parsing the command, but also validating the
    // user's permissions before we actually get to dispatching the command.
    fn handle_command(&self, sender: String, channel: String, command: String) -> Option<OutgoingMessage> {
        match command.parse::<Command>() {
            Ok(command) => match command {
                Command::Chuck => commands::chuck(sender),
                Command::Cookie => commands::cookie(sender),
                Command::Quote(category) => commands::quote(sender, category),
                Command::Roll(dice) => commands::roll(sender, dice),

                // FIXME: Admin commands like these need a separate pathway.
                // Bot settings
                Command::SetNick(nick) => commands::set_nick(self, sender, nick),
                Command::SetDebug(enabled) => commands::set_debug(self, sender, enabled),
                Command::SetTopic(topic) => commands::set_topic(self, sender, topic),

                // FIXME: In theory, we want to use this to add greetings to the bot's repertoire.
                Command::SetGreeting(ref greeting) => None,

                // This one looks odd, but the reason that a lot of these just send back None as their
                // channel message or whatever is just that they are meant to do work only on the
                // local machine. This one prints a report of all the messages sent out via SMS; we
                // really don't care to spew that across the network, do we?
                Command::ListMessages => {
                    match self.messaging.read() {
                        // Still don't think this is actually possible...
                        Err(_) => panic!("ugh"),
                        Ok(ref messaging) => list_notifications(messaging.sent()),
                    }

                    None
                },

                // FIXME: This is meant to cause the bot to exit, but so far all attempts to actually *do* this
                // have resulted in Very(TM) Bad Things(TM) happening as a result. We were initially using
                // `irc.close().ok()` from hiirc; I think the way to go at this point is just to actually exit
                // the application and let destructors close the TCP connection to the server.
                Command::Kill => None, // irc.close().ok()

                _ => None, // probably an unauthorized command
            },

            _ => None,
        }
    }

    fn greet_user(&self, user: String) -> Option<OutgoingMessage> {
        use greetings::Greetings;

        let mut greeting = self.greetings.for_user(&user)
            .fold(String::new(), |mut s, greeting| {
                s.push_str(&greeting.message(&user));
                s.push(' ');
                s
            });

        match greeting.len() {
            0 => None,
            x => {
                greeting.truncate(x - 1);
                Some(OutgoingMessage::to_channel(greeting))
            }
        }
    }

    #[inline]
    fn is_admin(&self, nick: &str) -> bool {
        self.admin.contains(nick)
    }

    #[inline]
    fn admin_mode(&self) -> bool {
        self.admin_mode
    }

    #[inline]
    fn watching(&self, nick: &str) -> bool {
        self.watch_list.contains(nick)
    }

    #[inline]
    fn logging(&self) -> bool {
        self.log_path.is_some()
    }

    fn open_log(&self) -> Result<File, io::Error> {
        use chrono::UTC;

        // I was going to write a test for this unwrap call, but, honestly, I figure everyone
        // and their dog knows that this particular format specifier is fine...
        let path = self.log_path.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "path not provided")
        })?;

        let path = format!("{}/{}_{}.log", path, UTC::now().format("%F"), self.server.channel.trim_left_matches('#'));
        OpenOptions::new().write(true).create(true).append(true).open(&path)
    }

    fn log(&self, nick: &str, message: &str) {
        if !self.logging() {
            return;
        }

        match self.open_log() {
            Err(e) => println!("{:?}", e),
            Ok(mut file) => {
                writeln!(file, "{}: {}", nick, message).ok();
            }
        };
    }
}

fn create_notification_service(config: &Config) -> NotificationService<Sms> {
    NotificationService::new(Sms::new(&*config.twilio.sid,
                                      &*config.twilio.token,
                                      &*config.twilio.number),
                             &*config.twilio.recipient,
                             Duration::from_secs(config.bot.message_frequency))
}

fn list_notifications<'a, T: Iterator<Item = (&'a String, &'a Instant)> + 'a>(notifications: T) {
    for subject in notifications {
        println!("{:?}", subject);
    }
}
