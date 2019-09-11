use regex::Regex;

bitflags! {
    struct State: u8 {
        /// parser inside positional arguments
        const POS = 0x01;
        /// parser inside keyword/option arguments (--i ..., -i ..., i: ..., i=...)
        const OPT = 0x02;
        /// got "--"
        /// all next arguments are positional
        const END = 0x04;
    }
}

impl State {
    #[inline(always)]
    pub fn set_pos(&mut self) {
        self.bits &= !Self::OPT.bits;
        self.bits |= Self::POS.bits;
    }

    #[inline(always)]
    pub fn set_opt(&mut self) {
        self.bits &= !Self::POS.bits;
        self.bits |= Self::OPT.bits;
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
    pub fn is_opt(self) -> bool {
        (self.bits & Self::OPT.bits) != 0
    }

    #[inline(always)]
    pub fn is_end(self) -> bool {
        (self.bits & Self::END.bits) != 0
    }
}

#[derive(Debug)]
pub enum Match {
    /// positional parameter
    Positional(String),
    /// --key | -key
    Key(String, Option<String>),
    /// --key=value | -key value | --key:value | --key value
    KeyValue(String, String),

    /// got option but no argument provided
    NoArg(String),

    /// unknown option
    UnknownKey(String),
    /// unknown option with value
    UnknownKeyValue(String, String),
    /// extra positional argument inside optional
    ExtraPositional(String),

    /// wtf?!! no match - should never happen
    No(String),
}

#[derive(Debug)]
pub enum OptKind {
    /// required-argument
    Arg,
    /// no argument required (bool flag) e.g. --help, --v, --vv, --vvv
    NoArg,
}

#[derive(Debug)]
pub struct Opt<'a>(pub &'a [&'a str], pub OptKind);

pub type Opts<'opt> = &'opt [&'opt Opt<'opt>];

fn opts_get<'opt, 's>(opts: Opts<'opt>, key: &'s str) -> Option<&'opt Opt<'opt>> {
    for opt in opts {
        for k in opt.0 {
            if *k == key {
                return Some(opt);
            }
        }
    }

    None
}

pub struct Matcher<'a> {
    args: Vec<String>,
    opts: Opts<'a>,
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
    #[allow(unused)]
    opts: Opts<'a>,
}

impl<'a> MatcherIter<'a> {
    /// check result of extracion value from (--key=value, --key:value etc)
    /// if no value provided - try to peek next value from iterator
    #[inline(always)]
    fn check_value_or_try_peek_next(&mut self, value: Option<String>) -> Option<String> {
        match value {
            Some(value) => Some(value),
            None => {
                // try to get option value from next argument
                match self.iter.peek() {
                    Some(arg_raw) => {
                        let arg = Arg::new(arg_raw.to_string());

                        if arg.is_end() || arg.is_option() {
                            return None;
                        }

                        self.iter.next(); // advance iterator

                        Some(arg.val)
                    }
                    None => None,
                }
            }
        }
    }
}

impl Iterator for MatcherIter<'_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        let arg_raw = self.iter.next()?;
        let mut arg = Arg::new(arg_raw);

        // got "-", just skip it
        if arg.val == "-" {
            arg = Arg::new(self.iter.next()?);
        }

        // got "--"
        if arg.is_end() {
            self.state.set_end();
            self.state.set_pos(); // only positional arguments after --
            arg = Arg::new(self.iter.next()?);
        }

        // got "-key" or "--key" for the first time
        if arg.is_option() && !self.state.is_end() {
            self.state.set_opt(); // only optional arguments after first option and before --
        }

        if self.state.is_pos() {
            Some(Match::Positional(arg.val))
        } else if self.state.is_opt() && arg.is_option() {
            let (key, value) = arg.extract_key_and_value();

            match (key, &value) {
                // should never hapen
                (None, _) => Some(Match::No(arg.val)),

                // check if opting defined in options array
                (Some(key), _) => match opts_get(self.opts, &key) {
                    // known option
                    Some(opt) => match opt.1 {
                        // no argument required but try to consume next positional if exists
                        // e.g. --debug=true or --debug true
                        OptKind::NoArg => match self.check_value_or_try_peek_next(value) {
                            Some(value) => Some(Match::Key(key, Some(value))),
                            None => Some(Match::Key(key, None)),
                        },

                        // must be with argument
                        OptKind::Arg => match self.check_value_or_try_peek_next(value) {
                            Some(value) => Some(Match::KeyValue(key, value)),
                            None => Some(Match::NoArg(key)),
                        },
                    },
                    // unknown option
                    None => match self.check_value_or_try_peek_next(value) {
                        Some(value) => Some(Match::UnknownKeyValue(key, value)),
                        None => Some(Match::UnknownKey(key)),
                    },
                },
            }
        } else {
            Some(Match::ExtraPositional(arg.val))
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
    fn extract_key_and_value(&self) -> (Option<String>, Option<String>) {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r#"(?x)
                -?-?                      # "--" or "-"
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

        let caps = match RE.captures(&self.val) {
            Some(caps) => caps,
            None => return (None, None),
        };

        let key = caps.name("key").map(|m| m.as_str().to_string());
        let value = caps.name("value").map(|m| m.as_str().to_string());

        (key, value)
    }
}
