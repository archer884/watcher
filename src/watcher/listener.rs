use watcher::{IrcHndl, ChnHndl, UsrHndl, Watcher};
use hiirc::{Event, IrcWrite, Listener};
use notifications::{NotificationResult, NotificationFailure};

impl Listener for Watcher {
    fn any(&mut self, _: IrcHndl, event: &Event) {
        if self.debug {
            println!("{:?}", event);
        }
    }

    /// Handle general channel messages.
    ///
    /// The first thing we do here is log all incoming messages (if logging is active, anyway),
    /// and then we look to handle any commands that arrive via normal channel chat.
    fn channel_msg(&mut self, irc: IrcHndl, channel: ChnHndl, user: UsrHndl, msg: &str) {
        // Log chat
        self.log(channel.name(), &user.nickname(), msg);
        println!("{}/{}: {}", channel.name(), user.nickname(), msg);

        // Handle public chat commands
        if msg.starts_with('.') {
            self.handle_command(irc, channel, user, msg);
        }
    }

    fn private_msg(&mut self, irc: IrcHndl, sender: &str, message: &str) {
        if self.debug {
            println!("PM from {}: {}", sender, message);
        }

        // do not waste time talking to statserv
        if sender == "StatServ" {
            return;
        }

        // we cannot handle commands from PM right here (or, you know, we can't currently)
        // so just tell the bastard we're afk and call it good
        irc.privmsg(sender, "AFK").ok();

        let message_result = self.messaging.notify_pm(sender, message);

        if self.debug {
            log_message_result(&message_result);
        }
    }

    /// Handle user_join events.
    ///
    /// We currently handle user join events on the "any" listener, which is a damn stupid idea.
    /// Instead, we should be handling them here.
    fn user_join(&mut self, irc: IrcHndl, channel: ChnHndl, user: UsrHndl) {
        // do not greet yourself
        if &self.identity.nick == &*user.nickname() {
            return;
        }

        // +o bot admin
        if self.admin_channel(channel.name()) && self.is_admin(&user.nickname()) {
            match irc.raw(format!("MODE {} +o {}", channel.name(), user.nickname())) {
                Err(e) => println!("{:?}", e),
                Ok(_) => println!("+o {}", user.nickname()),
            }
        }

        // notify owner of watched user entering channel or of user entering watched channel
        if self.admin_channel(channel.name()) || self.watching(&user.nickname()) {
            if self.debug {
                println!("sending SMS notification for {} in {}",
                         user.nickname(),
                         channel.name());
            }

            let message_result = self.messaging.notify_channel(&user.nickname(), channel.name());

            if self.debug {
                log_message_result(&message_result);
            }
        }

        // greet user
        if self.admin_channel(channel.name()) {
            self.greet_user(irc, channel, user);
        }
    }

    fn ping(&mut self, irc: IrcHndl, server: &str) {
        irc.pong(server).ok();
    }

    fn reconnect(&mut self, _: IrcHndl) {
        use chrono::UTC;
        println!("reconnect occurred: {}", UTC::now().format("%F %T"));
    }

    fn welcome(&mut self, irc: IrcHndl) {
        for channel in self.channels.values() {
            irc.join(&channel.name, None).ok();
            if channel.admin {
                if let Some(ref topic) = channel.topic {
                    match irc.set_topic(&channel.name, topic) {
                        Err(e) => println!("{:?}", e),
                        Ok(_) => println!("topic set for {}: {}", channel.name, topic),
                    }
                }
            }
        }
    }
}

fn log_message_result(message_result: &NotificationResult) {
    match message_result {
        &Ok(()) => println!("notification sent"),
        &Err(NotificationFailure::RecentlyNotified) => println!("notification withheld: recently notified"),
        &Err(NotificationFailure::Throttled) => println!("notification withheld: too many messages sent recently"),
        &Err(NotificationFailure::Failure(ref e)) => println!("notification failed: {:?}", e),
    }
}
