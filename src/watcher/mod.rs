mod commands;
mod listener;

use command::Command;
use config::{Config, ServerChannel, User};
use hiirc::{Channel, ChannelUser, Code, Irc, IrcWrite, Message, Prefix};
use notifications::{NotificationService, Sms};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::Error as IoError;
use std::io::Write;
use time::Duration;
use time;

pub type IrcHndl = ::std::sync::Arc<Irc>;
pub type ChnHndl = ::std::sync::Arc<Channel>;
pub type UsrHndl = ::std::sync::Arc<ChannelUser>;

pub struct Watcher {
    admin: HashSet<String>,
    identity: User,
    channels: HashMap<String, ServerChannel>,
    watch_list: HashSet<String>,
    messaging: NotificationService<Sms>,
    log_path: String,
    debug: bool,
}

impl Watcher {
    pub fn from_config(config: &Config) -> Watcher {
        Watcher {
            admin: config.bot.admin.iter().cloned().collect(),
            identity: config.user.clone(),
            channels: config.server.channels.iter().cloned().map(|channel| (channel.name.to_owned(), channel)).collect(),
            watch_list: config.bot.watch_list.iter().cloned().collect(),
            messaging: create_notification_service(config),
            log_path: config.logging.path.clone(),
            debug: false,
        }
    }

    fn handle_message(&mut self, irc: IrcHndl, message: &Message) {
        // If we're in debug mode, print this message to the screen no matter what it is.
        if self.debug {
            println!("{:?}", message);
        }

        // We're going to need the channel later on
        let channel = match message.args.get(0).and_then(|s| irc.channel(s.as_ref())) {
            Some(channel) => channel,
            None if self.debug => {
                println!("Unable to determine channel; not handling message");
                return;
            },
            None => return,
        };

        // If there's no user prefix on this message, we can't determine
        // the user associated with it and there's nothing to do
        if let Some(Prefix::User(ref user)) = message.prefix {
            let user = match channel.user(&user.nickname) {
                Some(user) => user,
                None if self.debug => {
                    println!("User not in channel; not handling message");
                    return;
                },
                None => return,
            };

            match message.code {
                // Make bot stop greeting itself -.-
                Code::Join if &self.identity.nick.as_ref() == user.nickname().as_ref() => return,

                // Bot admin has joined channel
                Code::Join if self.is_admin(&user.nickname()) => {
                    match irc.raw(format!("MODE {} +o {}", channel.name(), user.nickname())) {
                        Err(e) => println!("{:?}", e),
                        Ok(_) => println!("+o {}", user.nickname()),
                    }
                    self.greet_user(irc, channel, user);
                }

                Code::Join => {
                    // A user has joined an admin channel OR a watched user has joined any channel
                    if self.admin_channel(channel.name()) || self.watching(&user.nickname()) {
                        self.messaging.notify_channel(&user.nickname(), channel.name());
                    }

                    // A user has joined an admin channel
                    if self.admin_channel(channel.name()) {
                        self.greet_user(irc, channel, user);
                    }
                },

                // Bot has received private message; for right now, we're just going to respond
                // that we're AFK and call it good. Later on, we could handle these messages the
                // way we handle channel messages. It is unbelievably complicated to detect a pm.
                Code::Privmsg if message.args.get(0) == Some(&self.identity.nick) => {
                    // Hack to ignore StatServ
                    if user.nickname().as_ref() == "StatServ" {
                        return;
                    }

                    let content = message.args.get(1).map_or("", |s| s.as_ref());
                    if content.starts_with('.') {
                        self.handle_command(irc, channel, user.clone(), content);
                    } else {
                        irc.privmsg(&user.nickname(), "AFK").ok();
                    }

                    // Going to try letting the user know that the bot has received a PM, just...
                    self.messaging.notify_pm(
                        &user.nickname(),
                        content,
                    );
                },

                // This is an event code we don't cover yet
                _ => (),
            }
        }
    }

    // TODO: change this so that we're not *just* parsing the command, but also validating the
    // user's permissions before we actually get to dispatching the command.
    fn handle_command(&mut self, irc: IrcHndl, channel: ChnHndl, user: UsrHndl, command: &str) {
        if let Some(command) = command.parse::<Command>().ok() {
            match command {
                Command::Chuck => commands::chuck(irc, channel, user),
                Command::Cookie => commands::cookie(irc, channel, user),
                Command::Roll(dice) => commands::roll(irc, channel, user, dice),

                // Bot settings
                Command::SetNick(ref new_nick) if self.is_admin(&user.nickname()) => commands::set_nick(self, irc, new_nick),
                Command::SetDebug(enabled) if self.is_admin(&user.nickname()) => commands::set_debug(self, enabled),

                // Channel commands
                Command::JoinChannel(ref channel) if self.is_admin(&user.nickname()) => commands::join_channel(self, irc, channel),
                Command::LeaveChannel(ref channel) if self.is_admin(&user.nickname()) => commands::leave_channel(self, irc, channel),

                // Admin options
                Command::SetTopic(ref topic) if self.is_admin(&user.nickname()) => commands::set_topic(self, irc, channel, topic),
                Command::SetGreeting(ref greeting) => (), // In theory, this will be used to set the greeting the bot uses for people who enter its channel
                Command::Kill => (), // irc.close().ok() // this was used to kill the IRC connection, but that results in Bad Things(TM)

                _ => (), // probably an unauthorized command
            }
        }
    }

    fn greet_user(&mut self, irc: IrcHndl, channel: ChnHndl, user: UsrHndl) {
        let mut take = true;
        let greetings = self.channels.get(channel.name()).map(|channel| {
            let user = user.clone();
            channel.greetings.iter()
                .filter(move |greeting| greeting.is_valid(&user.nickname()))
                .take_while(move |greeting| {
                    let ret = take;
                    take = greeting.passthru();
                    ret
                })
        });

        if let Some(greetings) = greetings {
            for greeting in greetings {
                irc.privmsg(channel.name(), &greeting.message(&user.nickname())).ok();
            }
        }
    }

    #[inline]
    fn is_admin(&self, nick: &str) -> bool {
        self.admin.contains(nick)
    }

    fn admin_channel(&self, channel: &str) -> bool {
        match self.channels.get(channel) {
            None => false,
            Some(channel) => channel.admin
        }
    }

    #[inline]
    fn watching(&self, nick: &str) -> bool {
        self.watch_list.contains(nick)
    }

    #[inline]
    fn logging(&self, channel: &str) -> bool {
        self.channels.get(channel).map_or(false, |channel| channel.log_chat)
    }

    fn open_log(&self, channel: &str) -> Result<File, IoError> {
        // I was going to write a test for this unwrap call, but, honestly, I figure everyone
        // and their dog knows that this particular format specifier is fine...
        let path = format!(
            "{}/{}",
            self.log_path,
            time::strftime("%F", &time::now()).unwrap() + "_" + channel.trim_left_matches('#') + ".log",
        );

        OpenOptions::new().write(true).create(true).append(true).open(&path)
    }

    #[allow(unused)] // once again, we are swallowing the result of this write
    fn log(&self, channel: &str, nick: &str, message: &str) {
        if !self.logging(channel) {
            return;
        }

        match self.open_log(channel) {
            Err(e) => println!("{:?}", e),
            Ok(mut file) => {
                writeln!(file, "{}: {}", nick, message);
            }
        };
    }
}

fn create_notification_service(config: &Config) -> NotificationService<Sms> {
    NotificationService::new(
        Sms::new(
            &*config.twilio.sid,
            &*config.twilio.token,
            &*config.twilio.number,
        ),
        &*config.twilio.recipient,
        Duration::minutes(config.bot.message_frequency),
    )
}
