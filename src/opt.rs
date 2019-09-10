const END_OPTIONS: &str = "--";
const KEY_VALUE_SEPARATOR: char = '=';

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
    End,                      // --
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
        let arg = Arg::new(self.iter.next()?);

        if arg.is_option_end() {
            self.state.set_pos();
            self.state.set_end();
            return Some(Match::End);
        }

        if !self.state.is_end() && arg.is_option() {
            self.state.set_key();
        }

        if self.state.is_pos() {
            Some(Match::Positional(arg.val))
        } else if self.state.is_key() {
            if let Some(v) = self.iter.peek() {
                let arg_next = Arg::new(v.to_string());

                if arg_next.is_option_end() || arg_next.is_option() {
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
    fn is_option_end(&self) -> bool {
        self.val == END_OPTIONS
    }
}
