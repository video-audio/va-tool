use failure::Error;

use regex::Regex;

bitflags! {
    struct State: u8 {
        const POS = 0x01; // positional arguments
        const KEY = 0x02; // keyword/option arguments (--i ..., -i ..., i: ..., i=...)
        const END = 0x04; // got "--"
    }
}

impl State {
    #[inline(always)]
    pub fn set_pos(&mut self) {
        self.bits &= !Self::KEY.bits;
        self.bits |= Self::POS.bits;
    }

    #[inline(always)]
    pub fn set_key(&mut self) {
        self.bits &= !Self::POS.bits;
        self.bits |= Self::KEY.bits;
    }

    #[inline(always)]
    pub fn set_end(&mut self) {
        self.bits |= Self::END.bits
    }

    #[inline(always)]
    pub fn is_pos(self) -> bool {
        (self.bits & Self::POS.bits) != 0
    }

    #[inline(always)]
    pub fn is_key(self) -> bool {
        (self.bits & Self::KEY.bits) != 0
    }

    #[inline(always)]
    pub fn is_end(self) -> bool {
        (self.bits & Self::END.bits) != 0
    }
}

#[derive(Debug)]
pub enum Match {
    Positional(String),       // positional parameter
    Key(String),              // --key
    KeyValue(String, String), // --key=value
}

#[derive(Debug)]
pub enum OptKind {
    Arg,   // required-argument
    NoArg, // no argument required (bool flag) e.g. --help, --v, --vv, --vvv
}

#[derive(Debug)]
pub struct Opt<'a>(pub &'a str, pub &'a [&'a str], pub OptKind);

pub struct Matcher<'a> {
    args: Vec<String>,
    opts: &'a [&'a Opt<'a>],
}

impl<'a> Matcher<'a> {
    pub fn new(args: Vec<String>, opts: &'a [&'a Opt<'a>]) -> Self {
        Matcher { args, opts }
    }
}

impl<'a> IntoIterator for Matcher<'a> {
    type Item = Match;
    type IntoIter = MatcherIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MatcherIter {
            iter: self.args.into_iter().peekable(),
            opts: self.opts,
            state: State::POS,
        }
    }
}

pub struct MatcherIter<'a> {
    iter: std::iter::Peekable<std::vec::IntoIter<String>>,
    state: State,
    opts: &'a [&'a Opt<'a>],
}

impl Iterator for MatcherIter<'_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        let mut arg = Arg::new(self.iter.next()?);

        if arg.is_end() {
            self.state.set_pos();
            self.state.set_end();
            arg = Arg::new(self.iter.next()?);
        }

        if !self.state.is_end() && arg.is_option() {
            self.state.set_key();
        }

        if self.state.is_pos() {
            Some(Match::Positional(arg.val))
        } else if self.state.is_key() {
            println!("{:?}", arg.extract_key_and_value());

            if let Some(v) = self.iter.peek() {
                let arg_next = Arg::new(v.to_string());

                if arg_next.is_end() || arg_next.is_option() {
                    return Some(Match::Key(arg.val));
                }

                {
                    self.iter.next(); // advance iterator
                    self.state.set_pos(); // reset state
                }

                Some(Match::KeyValue(arg.val, arg_next.val))
            } else {
                Some(Match::Key(arg.val))
            }
        } else {
            None
        }
    }
}

struct Arg {
    val: String,
}

impl Arg {
    fn new(val: String) -> Arg {
        Arg { val }
    }

    #[inline(always)]
    fn is_option(&self) -> bool {
        self.val.starts_with('-')
    }

    #[inline(always)]
    fn is_end(&self) -> bool {
        self.val == "--"
    }

    /// extract
    ///
    ///  --key => (Some(key), None)
    ///  --key:value => (Some(key), Some(value))
    #[inline(always)]
    fn extract_key_and_value(&self) -> Result<(Option<String>, Option<String>), Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r#"(?x)
                -?-?               # "--" or "-"
                (?P<key>[a-zA-Z0-9_\-]+)  # key
                (?:
                    (:?
                        =|:
                    )?
                    (?P<value>[a-zA-Z0-9_\-]+)  # value
                )?
                "#,
            )
            .unwrap();
        }

        let caps = RE.captures(&self.val).unwrap();

        let key = caps.name("key").map(|m| m.as_str().to_string());
        let value = caps.name("value").map(|m| m.as_str().to_string());

        Ok((key, value))
    }
}
