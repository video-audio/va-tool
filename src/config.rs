#![allow(dead_code)]

use std::env;
use url;

use crate::error::{Error, Result};
use crate::opt::{Match as OptMatch, Matcher as OptMatcher, Opt, OptKind, Opts};

#[rustfmt::skip]
const OPTS: Opts = &[
    &Opt(&"vv", &["verbose"], OptKind::NoArg),
    &Opt(&"vvv", &["very-verbose"], OptKind::NoArg),
    &Opt(&"help", &["h"], OptKind::NoArg),
    &Opt(&"daemonize", &["background"], OptKind::NoArg),
    &Opt(&"foreground", &[], OptKind::NoArg),
    &Opt(&"print-config", &[], OptKind::NoArg),

    &Opt(&"config", &["c", "cfg"], OptKind::Arg),

    &Opt(&"input", &["i"], OptKind::Arg),
        &Opt(&"id", &[], OptKind::Arg),
        &Opt(&"name", &[], OptKind::Arg),
        &Opt(&"map", &["m"], OptKind::Arg),
        &Opt(&"out", &["o", "output"], OptKind::Arg),
];

pub struct ConfigOutput {
    url: url::Url,
}

pub struct ConfigStream {
    id: u64,
    outputs: Vec<ConfigOutput>,
}

pub struct ConfigInput {
    id: u64,
    url: url::Url,
}

pub struct Config {
    // pub _action: analyze | report | plot
    pub print_help: bool,
    pub print_version: bool,
    pub print_config: bool,

    pub log_level: log::Level,

    inputs: Vec<ConfigInput>,
    streams: Vec<ConfigStream>,
}

impl Config {
    pub fn parse() -> Result<Config> {
        let mut c = Config {
            print_help: false,
            print_version: false,
            print_config: false,
            log_level: log::Level::Info,

            inputs: Default::default(),
            streams: Default::default(),
        };

        let opt_matcher = OptMatcher::new(env::args().skip(1).collect(), OPTS);

        for (i, mtch) in opt_matcher.into_iter().enumerate() {
            match mtch {
                OptMatch::Key(key, _) => match key {
                    "vv" => c.log_level = log::Level::Debug,
                    "vvv" => c.log_level = log::Level::Trace,
                    "help" => c.print_help = true,
                    "version" => c.print_version = true,
                    "print-config" => c.print_config = true,
                    _ => {}
                },

                OptMatch::KeyValue(key, value) => match key {
                    "input" | "i" => c.push_input(value)?,
                    _ => {}
                },

                OptMatch::Positional(value) | OptMatch::ExtraPositional(value) => {
                    if i == 0 && value == "analyze" {
                    } else {
                        c.push_input(value)?
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
        println!("log-level: {}", self.log_level.to_string().to_lowercase());
        println!("inputs:");
        for input in self.inputs.iter() {
            println!("  - url: {}", input.url);
        }
    }

    pub fn push_input(&mut self, url_raw: String) -> Result<()> {
        let cfg_input = ConfigInput {
            id: 0,
            url: url::Url::parse(&url_raw).map_err(|err| Error::url_parse(err, url_raw))?,
        };

        self.inputs.push(cfg_input);

        Ok(())
    }
}
