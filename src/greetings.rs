use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::slice;

#[derive(Clone)]
pub struct Greeting {
    passthru: bool,
    filter: Option<Regex>, // filter is optional
    message: String,
}

impl Greeting {
    #[inline]
    pub fn message(&self, nick: &str) -> String {
        self.message.replace("{nick}", nick)
    }

    #[inline]
    pub fn is_valid(&self, nick: &str) -> bool {
        self.filter.as_ref().map(|p| p.is_match(nick)).unwrap_or(true)
    }
}

pub struct GreetingsForUser<'a>
{
    user: &'a str,
    greetings: slice::Iter<'a, Greeting>,
    take: bool,
}

impl<'a> Iterator for GreetingsForUser<'a> {
    type Item = &'a Greeting;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.take { return None; }

        loop {
            match self.greetings.next() {
                None => return None,

                Some(greeting) if greeting.is_valid(self.user) => {
                    self.take = greeting.passthru;
                    return Some(greeting);
                }

                _ => (),
            }
        }
    }
}

pub trait Greetings<'a> {
    fn for_user(&'a self, user: &'a str) -> GreetingsForUser<'a>;
}

impl<'a> Greetings<'a> for Vec<Greeting> {
    fn for_user(&'a self, user: &'a str) -> GreetingsForUser<'a> {
        GreetingsForUser {
            user: user,
            greetings: self.iter(),
            take: true,
        }
    }
}

impl Deserialize for Greeting {
    fn deserialize<D: Deserializer>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Template {
            passthru: bool,
            filter: Option<String>,
            message: String,
        }

        let template = Template::deserialize(d)?;
        let filter = match template.filter {
            None => None,

            // Our friendly bot should refuse to start if your message filters are invalid
            Some(ref filter) => Some(Regex::new(filter).expect(&format!("bad greeting filter: {}", filter))),
        };

        Ok(Greeting {
            passthru: template.passthru,
            filter: filter,
            message: template.message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Greeting, Greetings};
    use regex::Regex;

    #[test]
    fn plain_matches_are_accepted() {
        assert!(greeting().is_valid("John"));
    }

    #[test]
    fn message_value() {
        assert_eq!("Hello, John!", greeting().message("John"));
    }

    #[test]
    fn iterator_handles_filter_and_passthru_correctly() {
        let greetings = vec![Greeting {
            passthru: false,
            filter: Regex::new("Jack").ok(),
            message: "Hit the road, Jack.".to_owned(),
        }, greeting(), greeting(), Greeting {
            passthru: false,
            filter: None,
            message: "Hello, {nick}!".to_owned(),
        }, greeting()];

        assert_eq!(3, greetings.for_user("John").count());
    }

    fn greeting() -> Greeting {
        Greeting {
            passthru: true,
            filter: Regex::new("John").ok(),
            message: "Hello, {nick}!".to_owned(),
        }
    }
}
