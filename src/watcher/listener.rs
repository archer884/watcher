use super::{IrcHndl, ChnHndl, UsrHndl, Watcher};
use hiirc::{Event, IrcWrite, Listener};

impl Listener for Watcher {
    #[allow(unused)]
    fn any(&mut self, irc: IrcHndl, event: &Event) {
        if let &Event::Message(ref message) = event {
            self.handle_message(irc, &message);
        }
    }

    fn channel_msg(&mut self, irc: IrcHndl, channel: ChnHndl, user: UsrHndl, msg: &str) {
        // Log chat
        self.log(&channel.name(), &user.nickname(), msg);
        println!("{}: {}", user.nickname(), msg);

        // Handle public chat commands
        if msg.starts_with(".") {
            self.handle_command(irc, channel, user, msg);
        }
    }

    fn ping(&mut self, irc: IrcHndl, server: &str) {
        irc.pong(server).ok();
    }

    fn reconnect(&mut self, _: IrcHndl) {
        // no idea what this needs to do here
    }

    fn welcome(&mut self, irc: IrcHndl) {
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
