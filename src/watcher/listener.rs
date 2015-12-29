use super::Watcher;
use hiirc::{Channel, ChannelUser, Event, Irc, Listener};

impl Listener for Watcher {
    #[allow(unused)]
    fn any(&mut self, irc: &Irc, event: &Event) {
        if let &Event::Message(ref message) = event {
            self.handle_message(irc, &message);
        }
    }

    fn channel_msg(&mut self, irc: &Irc, channel: &Channel, user: &ChannelUser, msg: &str) {
        // Log chat
        self.log(&channel.name, &user.nickname, msg);
        println!("{}: {}", user.nickname, msg);

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
