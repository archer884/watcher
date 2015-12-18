use std::collections::HashMap;

use time::{Duration, Timespec};
use time::get_time as current_time;

pub use rsilio::MessagingService as Sms;

pub enum NotificationResult {
    Sent,
    Withheld,
    Failure(String),
}

pub trait NotificationSink {
    fn send_message(&self, recipient: &str, message: &str) -> NotificationResult;
}

// The idea here is that a notification service wraps any two notification mechanisms, which
// represent, respectively, SMS or email sinks.
pub struct NotificationService<T: NotificationSink> {
    sink: T,
    sent: HashMap<String, Option<Timespec>>,
    recipient: String,
    frequency: Duration,
}

impl<T: NotificationSink> NotificationService<T> {
    pub fn new<S: Into<String>>(sink: T, recipient: S, frequency: Duration) -> NotificationService<T> {
        NotificationService {
            sink: sink,
            sent: HashMap::new(),
            recipient: recipient.into(),
            frequency: frequency,
        }
    }

    /// Notifies the user that a watched nick has entered a watched channel
    pub fn notify_channel(&mut self, nick: &str, channel: &str) -> NotificationResult {
        if self.can_send(nick) {
            let ret = self.sink.send_message(
                &self.recipient,
                &format!("{} has joined {}", nick, channel)
            );

            self.update_sent(nick);
            ret
        } else {
            NotificationResult::Withheld
        }
    }

    fn can_send(&mut self, nick: &str) -> bool {
        let entry = self.sent.entry(nick.to_owned()).or_insert(None);
        let frequency = self.frequency;

        entry.clone().map(|tm| (current_time() - tm) > frequency).unwrap_or(true)
    }

    fn update_sent(&mut self, nick: &str) {
        self.sent.insert(nick.to_owned(), Some(current_time()));
    }
}

impl NotificationSink for Sms {
    fn send_message(&self, recipient: &str, message: &str) -> NotificationResult {
        match self.send_message(recipient, &message) {
            Ok(_) => NotificationResult::Sent,
            Err(e) => NotificationResult::Failure(e),
        }
    }
}
