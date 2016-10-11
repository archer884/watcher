mod commands;
mod listener;

use command::Command;
use config::{Config, ServerChannel, User};
use hiirc::{Channel, ChannelUser, Irc, IrcWrite};
use notifications::{NotificationService, Sms};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::Error as IoError;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub type IrcHndl = Arc<Irc>;
pub type ChnHndl = Arc<Channel>;
pub type UsrHndl = Arc<ChannelUser>;

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
            channels: config.server.channels.iter().cloned()
                .map(|channel| (channel.name.to_owned(), channel))
                .collect(),
            watch_list: config.bot.watch_list.iter().cloned().collect(),
            messaging: create_notification_service(config),
            log_path: config.logging.path.clone(),
            debug: true,
        }
    }

    // TODO: change this so that we're not *just* parsing the command, but also validating the
    // user's permissions before we actually get to dispatching the command.
    fn handle_command(&mut self, irc: IrcHndl, channel: ChnHndl, user: UsrHndl, command: &str) {
        if let Some(command) = command.parse::<Command>().ok() {
            match command {
                Command::Chuck => commands::chuck(irc, channel, user),
                Command::Cookie => commands::cookie(irc, channel, user),
                Command::Quote(category) => commands::quote(irc, channel, user, category),
                Command::Roll(dice) => commands::roll(irc, channel, user, dice),

                // Bot settings
                Command::SetNick(ref new_nick) if self.is_admin(&user.nickname()) => {
                    commands::set_nick(self, irc, new_nick)
                }
                Command::SetDebug(enabled) if self.is_admin(&user.nickname()) => {
                    commands::set_debug(self, enabled)
                }

                // Channel commands
                Command::JoinChannel(ref channel) if self.is_admin(&user.nickname()) => {
                    commands::join_channel(self, irc, channel)
                }
                Command::LeaveChannel(ref channel) if self.is_admin(&user.nickname()) => {
                    commands::leave_channel(self, irc, channel)
                }

                // Admin options
                Command::SetTopic(ref topic) if self.is_admin(&user.nickname()) => {
                    commands::set_topic(self, irc, channel, topic)
                }
                Command::SetGreeting(ref greeting) => (), // In theory, this will be used to set the greeting the bot uses for people who enter its channel
                Command::ListMessages => list_notifications(self.messaging.sent()),
                Command::Kill => (), // irc.close().ok() // this was used to kill the IRC connection, but that results in Bad Things(TM)

                _ => (), // probably an unauthorized command
            }
        }
    }

    fn greet_user(&mut self, irc: IrcHndl, channel: ChnHndl, user: UsrHndl) {
        let mut take = true;
        let greetings = self.channels.get(channel.name()).map(|channel| {
            let user = user.clone();
            channel.greetings
                .iter()
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
            Some(channel) => channel.admin,
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
        use time;

        // I was going to write a test for this unwrap call, but, honestly, I figure everyone
        // and their dog knows that this particular format specifier is fine...
        let path = format!("{}/{}",
                           self.log_path,
                           time::strftime("%F", &time::now()).unwrap() + "_" +
                           channel.trim_left_matches('#') + ".log");
        OpenOptions::new().write(true).create(true).append(true).open(&path)
    }

    fn log(&self, channel: &str, nick: &str, message: &str) {
        if !self.logging(channel) {
            return;
        }

        match self.open_log(channel) {
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
