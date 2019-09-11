use std::env;
use url;

use crate::opt::{Match as OptMatch, Matcher as OptMatcher, Opt, OptKind, Opts};
use crate::result::Result;

#[rustfmt::skip]
const OPTS: Opts = &[
    &Opt(&["vv", "verbose"], OptKind::NoArg),
    &Opt(&["vvv", "very-verbose"], OptKind::NoArg),
    &Opt(&["h", "help"], OptKind::NoArg),
    &Opt(&["daemonize", "background"], OptKind::NoArg),
    &Opt(&["foreground"], OptKind::NoArg),
    &Opt(&["print-config"], OptKind::NoArg),

    &Opt(&["c", "cfg", "config"], OptKind::Arg),

    &Opt(&["i", "input"], OptKind::Arg),
        &Opt(&["id"], OptKind::Arg),
        &Opt(&["name"], OptKind::Arg),
        &Opt(&["m", "map"], OptKind::Arg),
        &Opt(&["o", "out", "output"], OptKind::Arg),
];

pub struct ConfigOutput {
    url: url::Url,
}

pub struct ConfigMap {
    outputs: Vec<ConfigOutput>,
}

pub struct ConfigInput {
    url: url::Url,

    maps: Vec<ConfigMap>,
}

pub struct Config {
    pub print_help: bool,
    pub print_version: bool,
    pub print_config: bool,

    pub log_level: log::Level,

    inputs: Vec<ConfigInput>,
}

impl Config {
    pub fn parse() -> Result<Config> {
        let mut c = Config {
            print_help: false,
            print_version: false,
            print_config: false,
            log_level: log::Level::Info,

            inputs: Vec::new(),
        };

        let opt_matcher = OptMatcher::new(env::args().skip(1).collect(), OPTS);

        for (i, mtch) in opt_matcher.into_iter().enumerate() {
            match mtch {
                OptMatch::Key(ref key, _) if key == "vv" => c.log_level = log::Level::Debug,
                OptMatch::Key(ref key, _) if key == "vvv" => c.log_level = log::Level::Trace,
                OptMatch::Key(ref key, _) if key == "help" => c.print_help = true,
                OptMatch::Key(ref key, _) if key == "version" => c.print_version = true,
                OptMatch::Key(ref key, _) if key == "print-config" => c.print_config = true,

                OptMatch::KeyValue(ref key, ref value) if key == "input" || key == "i" => {
                    let cfg_input = ConfigInput {
                        url: url::Url::parse(value)?,
                        maps: Default::default(),
                    };

                    c.inputs.push(cfg_input);
                }

                OptMatch::Positional(ref value) | OptMatch::ExtraPositional(ref value) => {
                    if i == 0 && value == "analyze" {
                    } else {
                        let cfg_input = ConfigInput {
                            url: url::Url::parse(value)?,
                            maps: Default::default(),
                        };

                        c.inputs.push(cfg_input);
                    }
                }

                OptMatch::UnknownKey(key) => {
                    log::warn!(r#"unrecognized option "{}""#, key);
                }
                OptMatch::UnknownKeyValue(key, value) => {
                    log::warn!(r#"unrecognized option "{}" with argument "{}""#, key, value);
                }
                OptMatch::No(key) => {
                    log::warn!(r#"unknown argument "{}""#, key);
                }

                _ => {}
            }
        }

        Ok(c)
    }

    pub fn print_help(&self) {
        println!(r#"Usage: va-tool [...] [-arg ...] [--arg[="..."]] [--] [...]"#);
        println!();
        println!("  -vv, --verbose                 | <bool>    | ... ");
        println!("  -vvv, --very-verbose           | <bool>    | ... ");
        println!("  -i, --intput                   | <str/url> | Where to read from");
        println!("  -o, --output, --out            | <str/url> | Where to write to");
        println!();
    }

    pub fn print_version(&self) {}

    pub fn print_config(&self) {
        println!("inputs:");
        for input in self.inputs.iter() {
            println!("  - url: {}", input.url);
        }
    }
}
