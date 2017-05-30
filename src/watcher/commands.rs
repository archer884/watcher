use dice::Dice;
use fortune_cookie;
use icndb::next as get_awesome;
use watcher::Watcher;
use eirsee::message::OutgoingMessage;

const DEFAULT_CHUCK: &'static str = "No one really knows Chuck Norris. Not even Chuck Norris!";
const DEFAULT_COOKIE: &'static str = "Man who run in front of car get tired. Man who run behind \
                                      car get exhausted.";
const DEFAULT_QUOTE: &'static str = "Talk low, talk slow, and don't say too much. -John Wayne";

pub fn chuck(sender: String) -> Option<OutgoingMessage> {
    println!("{} has requested some CHUCK ACTION!", sender);
    Some(
        get_awesome()
            .map(|res| OutgoingMessage::to_channel(res.joke))
            .unwrap_or_else(|| OutgoingMessage::to_channel(String::from(DEFAULT_CHUCK)))
    )
}

pub fn cookie(sender: String) -> Option<OutgoingMessage> {
    println!("{} has requested a FORTUNE COOKIE", sender);
    Some(
        fortune_cookie::cookie().ok()
            .map(|res| OutgoingMessage::to_channel(res))
            .unwrap_or_else(|| OutgoingMessage::to_channel(String::from(DEFAULT_COOKIE)))
    )
}

pub fn list_commands() -> Option<OutgoingMessage> {
    Some(OutgoingMessage::ChannelMessage {
        content: String::from(".chuck .cookie .quote .quote <category> .roll <1d6>")
    })
}

pub fn quote(sender: String, category: Option<String>) -> Option<OutgoingMessage> {
    use quote_rs::Service;

    println!("{} has requested a QUOTE", sender);

    let service = Service::new();
    let quote = match category {
        None => service.qod(),
        Some(ref category) => service.qod_for_category(category),
    };

    Some(
        quote.ok()
            .map(|res| OutgoingMessage::to_channel(format!("{} -{}", res.quote, res.author)))
            .unwrap_or_else(|| OutgoingMessage::to_channel(String::from(DEFAULT_QUOTE)))
    )
}

pub fn list_quote_categories() -> Option<OutgoingMessage> {
    Some(OutgoingMessage::const_to_channel("inspire, management, sports, life, funny, love, art, students"))
}

pub fn roll(sender: String, dice: Vec<Dice>) -> Option<OutgoingMessage> {
    use rand;

    println!("{} has requested DICE ROLLS: {:?}", sender, dice);

    let mut rng = rand::thread_rng();
    let results: Vec<u32> = dice.iter().flat_map(|roll| roll.gen_result(&mut rng)).collect();
    let formatted_results = format_dice_results(&results);

    Some(OutgoingMessage::to_channel(
        format!("{} rolled {} ({})", sender, formatted_results, results.iter().sum::<u32>())
    ))
}

pub fn set_nick(watcher: &Watcher, sender: String, nick: String) -> Option<OutgoingMessage> {
    if watcher.is_admin(&sender) {
        Some(OutgoingMessage::Nick(Some(nick)))
    } else {
        None
    }
}

// FIXME: Admin commands like this one need a totally separate path, because checking this here is some 
// total bullshit.
pub fn set_debug(watcher: &Watcher, sender: String, enabled: bool) -> Option<OutgoingMessage> {
    if watcher.is_admin(&sender) {
        watcher.debug.set(enabled);
        println!("debug mode {}", if enabled { "enabled" } else { "disabled" });

        Some(OutgoingMessage::to_private(sender, format!("debug mode set to {}", enabled)))
    } else {
        None
    }
}

// Watcher is unused here because currently we're just setting the topic on the server, but
// the idea is that we'll be storing the topic string as part of the ServerChannel object in
// our list of channels, so, for the future, I'm leaving the Watcher object as part of this
// function signature.
//
// Also, see FIXME note in handle_command for what needs to happen here.
pub fn set_topic(watcher: &Watcher, sender: String, topic: String) -> Option<OutgoingMessage> {
    if watcher.is_admin(&sender) {
        Some(OutgoingMessage::Topic(topic))
    } else {
        None
    }
}

fn format_dice_results(values: &[u32]) -> String {
    use std::fmt::Write;

    if values.len() == 1 {
        return values.first().unwrap().to_string();
    }

    let mut buf = String::new();
    for (idx, &n) in values.iter().enumerate() {
        let count = values.len();
        if idx + 1 == count {
            write!(buf, "{}", n).ok();
        } else {
            write!(buf, "{}, ", n).ok();
        }
    }
    buf
}
