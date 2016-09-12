use std::collections::HashMap;
use std::time::{Duration, Instant};

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
    sent: HashMap<String, Instant>,
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
            self.sink.send_message(&self.recipient, &format!("{} has joined {}", nick, channel))
        } else {
            NotificationResult::Withheld
        }
    }

    /// Notifies the user that the bot has recieved a private message
    pub fn notify_pm(&mut self, nick: &str, message: &str) -> NotificationResult {
        if self.can_send(nick) {
            self.sink.send_message(&self.recipient, &format!("PM from {}: {}", nick, message))
        } else {
            NotificationResult::Withheld
        }
    }

    fn can_send(&mut self, nick: &str) -> bool {
        let frequency = self.frequency;
        self.sent
            .insert(nick.to_owned(), Instant::now())
            .map_or(true, |last| last.elapsed() > frequency)
    }
}

impl NotificationSink for Sms {
    fn send_message(&self, recipient: &str, message: &str) -> NotificationResult {
        match self.send_message(recipient, message) {
            Ok(_) => NotificationResult::Sent,
            Err(e) => NotificationResult::Failure(e),
        }
    }
}
