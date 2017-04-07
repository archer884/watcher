use std::collections::HashMap;
use std::time::{Duration, Instant};

pub use rsilio::MessagingService as Sms;

pub type NotificationResult = Result<(), NotificationFailure>;

struct ThrottleWindow {
    pub period: Duration,
    pub max_count: usize,
}

impl ThrottleWindow {
    fn can_send<T: Iterator<Item=Instant>>(&self, items: T) -> bool {
        items.filter(|&item| self.in_window(item)).count() < self.max_count
    }

    fn in_window(&self, time: Instant) -> bool {
        time.elapsed() < self.period
    }
}

#[derive(Debug)]
pub enum NotificationFailure {
    RecentlyNotified,
    Throttled,
    Failure(String),
}

pub trait NotificationSink {
    fn send_message(&self, recipient: &str, message: &str) -> NotificationResult;
}

pub struct NotificationService<T: NotificationSink> {
    sink: T,
    sent: HashMap<String, Instant>,
    recipient: String,
    frequency: Duration,
    window: ThrottleWindow,
}

impl<T: NotificationSink> NotificationService<T> {
    pub fn new<S: Into<String>>(sink: T, recipient: S, frequency: Duration) -> NotificationService<T> {
        NotificationService {
            sink: sink,
            sent: HashMap::new(),
            recipient: recipient.into(),
            frequency: frequency,
            window: ThrottleWindow {
                period: Duration::from_secs(60 * 60 * 3),
                max_count: 30,
            }
        }
    }

    /// Notify the user that a watched nick has entered a watched channel.
    pub fn notify_channel(&mut self, nick: &str, channel: &str) -> NotificationResult {
        match self.can_send(nick) {
            Err(e) => Err(e),
            Ok(_) => self.sink.send_message(&self.recipient, &format!("{} has joined {}", nick, channel))
        }
    }

    /// Notify the user that the bot has recieved a private message.
    pub fn notify_pm(&mut self, nick: &str, message: &str) -> NotificationResult {
        match self.can_send(nick) {
            Err(e) => Err(e),
            Ok(_) => self.sink.send_message(&self.recipient, &format!("PM from {}: {}", nick, message)),
        }
    }

    fn can_send(&mut self, nick: &str) -> NotificationResult {
        if !self.window.can_send(self.sent.iter().map(|(_, &instant)| instant)) {
            return Err(NotificationFailure::Throttled);
        }

        let frequency = self.frequency;
        let can_send = self.sent
            .insert(nick.to_owned(), Instant::now())
            .map_or(true, |last| last.elapsed() > frequency);

        if !can_send {
            return Err(NotificationFailure::RecentlyNotified)
        }

        Ok(())
    }

    pub fn sent<'a>(&'a self) -> impl Iterator<Item = (&'a String, &'a Instant)> {
        self.sent.iter()
    }
}

impl NotificationSink for Sms {
    fn send_message(&self, recipient: &str, message: &str) -> NotificationResult {
        match self.send_message(recipient, message) {
            Ok(_) => Ok(()),
            Err(e) => Err(NotificationFailure::Failure(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use super::ThrottleWindow;

    #[test]
    fn messages_allowed_when_threshold_not_passed() {
        let items = vec![Instant::now(), Instant::now(), Instant::now()];
        let window = ThrottleWindow {
            period: Duration::from_secs(60),
            max_count: 5,            
        };

        assert!(window.can_send(items.iter().cloned()));
    }

    #[test]
    fn messages_withheld_when_threshold_passed() {
        let items = vec![Instant::now(), Instant::now(), Instant::now(), Instant::now(), Instant::now(), Instant::now()];
        let window = ThrottleWindow {
            period: Duration::from_secs(60),
            max_count: 5,            
        };

        assert!(!window.can_send(items.iter().cloned()));
    }

    #[test]
    fn old_messages_do_not_count_against_threshold() {
        let items = vec![Instant::now() - Duration::from_secs(120), Instant::now(), Instant::now(), Instant::now(), Instant::now()];
        let window = ThrottleWindow {
            period: Duration::from_secs(60),
            max_count: 5,            
        };

        assert!(window.can_send(items.iter().cloned()));
    }
}
