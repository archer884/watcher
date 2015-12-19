use std::str::FromStr;

pub enum Command {
    Chuck,

    // bot options
    SetNick(String),
    SetDebug(bool),

    // channel options
    JoinChannel(String),
    LeaveChannel(String),

    // admin options
    SetTopic(String),
    SetGreeting(String),
    Kill,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<_> = s.split_whitespace().map(AsRef::as_ref).collect();
        match &data[..] {
            [".chuck"] => Ok(Command::Chuck),

            // bot options
            [".debug", enabled] => Ok(Command::SetDebug(enabled.parse().unwrap_or(false))),
            [".nick", nick] => Ok(Command::SetNick(nick.to_owned())),

            // channel options
            [".join", channel] => Ok(Command::JoinChannel(channel.to_owned())),
            [".leave", channel] => Ok(Command::LeaveChannel(channel.to_owned())),

            // admin options
            [".topic", _..] => Ok(Command::SetTopic(s.replace(".topic ", ""))),
            [".greet", _..] => Ok(Command::SetGreeting(s.replace(".greet ", ""))),
            [".kill"] => Ok(Command::Kill),

            _ => Err(()),
        }
    }
}
