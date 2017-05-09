use eirsee::message::OutgoingMessage;
use eirsee::responder::Responder;
use notifications::{NotificationResult, NotificationFailure};
use watcher::Watcher;

impl Responder for Watcher {
    fn channel_message(&self, sender: String, channel: String, content: String) -> Option<OutgoingMessage> {
        // Log chat.
        self.log(&sender, &content);
        println!("#{} ({}): {}", channel, sender, content);

        // Handle public chat commands.
        if content.starts_with('.') {
            self.handle_command(sender, channel, content)
        } else {
            None
        }
    }

    fn private_message(&self, sender: String, content: String) -> Option<OutgoingMessage> {
        match self.messaging.write() {
            // No idea under what circumstances we would actually get to this.
            Err(_) => panic!("well, shit"),

            Ok(mut messaging) => {
                if self.debug.get() {
                    println!("PM from {}: {}", sender, content);
                }

                // This code was in the original Watcher listener implementation built on top of hiirc. I do not know
                // if this code is still required with the new implementation, since I believe it does a better job of
                // filtering private messages. Even if it doesn't, I intend to ensure that it does before I'm done.
                //
                // Do not waste time talking ot StatServ.
                // if sender == "StatServ" {
                //     return None;
                // }

                let notification_result = messaging.notify_pm(&sender, &content);
                if self.debug.get() {
                    log_message_result(&notification_result);
                }

                Some(OutgoingMessage::to_private(sender, String::from("Sorry, I'm AFK right now. Or a bot. Take your pick.")))
            }
        }
    }

    fn user_join(&self, user: String) -> Option<OutgoingMessage> {
        // Do not greet yourself.
        if self.identity.nick == user { return None; }

        // +o bot admin.
        if self.admin_mode && self.is_admin(&user) {
            return Some(OutgoingMessage::Raw(
                format!("MODE {} +o {}", self.server.channel, user)
            ));
        }

        match self.messaging.write() {
            Err(_) => panic!("fml"),
            Ok(mut messaging) => {
                if self.admin_mode || self.watching(&user) {
                    if self.debug.get() {
                        println!("sending SMS notification for {} in {}", user, self.server.channel);
                    }

                    let message_result = messaging.notify_channel(&user, &self.server.channel);

                    if self.debug.get() {
                        log_message_result(&message_result);
                    }
                }
            }
        }

        // greet user
        self.greet_user(user)
    }

    fn user_part(&self, user: String) -> Option<OutgoingMessage> {
        unimplemented!()
    }
}

fn log_message_result(message_result: &NotificationResult) {
    match *message_result {
        Ok(()) => println!("notification sent"),
        Err(NotificationFailure::RecentlyNotified) => println!("notification withheld: recently notified"),
        Err(NotificationFailure::Throttled) => println!("notification withheld: too many messages sent recently"),
        Err(NotificationFailure::Failure(ref e)) => println!("notification failed: {:?}", e),
    }
}
