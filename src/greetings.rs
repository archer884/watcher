use regex::Regex;
use rustc_serialize::{Decodable, Decoder};

#[derive(Clone)]
pub struct Greeting {
    passthru: bool,
    filter: Option<Regex>, // filter is optional
    message: String,
}

impl Greeting {
    #[inline]
    pub fn passthru(&self) -> bool {
        self.passthru
    }

    #[inline]
    pub fn message(&self, nick: &str) -> String {
        self.message.replace("{nick}", nick)
    }

    #[inline]
    pub fn is_valid(&self, nick: &str) -> bool {
        match self.filter {
            None => true,
            Some(ref pattern) => pattern.is_match(nick),
        }
    }
}

// This decoding implementation took forever to come up with, and I only achieved this by the
// assistance of Shepmaster from the Rust reddit. The main thing that threw me off was that I
// couldn't figure out how to do this `read_struct_field` nonsense, and he was instrumental in
// assisting me with what I have here. He also provided an alternate solution, however, in the
// form of a better implementation for the internal struct that I tried earlier: you deserialize
// to something like this:
//
// #[derive(RustcDecodable)]
// pub struct GreetingCore {
//     passthru: bool,
//     filter: Option<String>, // filter is optional
//     message: String,
// }
//
// ...and then you use that value as the basis for your `Decodable` implementation for the real
// struct, like this:
//
// impl Decodable for Greeting {
//     fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
//         let core = try!(GreetingCore::decode(d));
//
//         Ok(Greeting {
//             passthru: core.passthru,
//             filter: core.filter.and_then(|f| Regex::new(&f).ok()),
//             message: core.message,
//         })
//     }
// }
//
// I like this way of doing things, absolutely--I just want to make sure that I learn to do it
// both ways, so I'm implementing it the hard way in my actual program (...well, it's not *that*
// hard, is it?), but I'm making this comment. Hopefully I can internalize all this. :)
//
// Ok, upon further examination of the circumstances, it does look like the easiest way to make
// the filter *genuinely* optional is to actually go with the core struct approach, so I'm going
// to preserve the original code for posterity and then the stuff I have above in the comment will
// be the actual implementation. :p

// Original code:
// impl Decodable for Greeting {
//     fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
//         Ok(Greeting {
//             passthru: try!(d.read_struct_field("passthru", 0, |d| d.read_bool())),
//             filter: try!(build_filter(d.read_struct_field("filter", 0, |d| d.read_str()))),
//             message: try!(d.read_struct_field("message", 0, |d| d.read_str())),
//         })
//     }
// }
//
// fn build_filter<T>(s: Result<String, T>) -> Result<Option<Regex>, T> {
//     match s {
//         Err(error) => Err(error),
//         Ok(s) => Ok(Regex::new(&s).ok()),
//     }
// }

impl Decodable for Greeting {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        #[derive(RustcDecodable)]
        struct Core {
            passthru: bool,
            filter: Option<String>,
            message: String,
        }

        let core = try!(Core::decode(d));
        Ok(Greeting {
            passthru: core.passthru,
            filter: core.filter.and_then(|f| Regex::new(&f).ok()),
            message: core.message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Greeting;
    use regex::Regex;

    #[test]
    fn plain_matches_are_accepted() {
        assert!(greeting().is_valid("John"));
    }

    #[test]
    fn message_value() {
        assert_eq!("Hello, John!", greeting().message("John"));
    }

    fn greeting() -> Greeting {
        Greeting {
            passthru: true,
            filter: Regex::new("John").ok(),
            message: "Hello, {nick}!".to_owned(),
        }
    }
}
