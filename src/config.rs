use std::env;

use regex::Regex;
use url::Url;

use crate::error::{Error, Result};
use crate::opt::{Match as OptMatch, Matcher as OptMatcher, Opt, OptKind, Opts};

#[rustfmt::skip]
const OPTS: Opts = &[
    &Opt(&"vv", &["verbose"], OptKind::NoArg),
    &Opt(&"vvv", &["very-verbose"], OptKind::NoArg),
    &Opt(&"help", &["h"], OptKind::NoArg),
    &Opt(&"version", &["v"], OptKind::NoArg),
    &Opt(&"print-config", &[], OptKind::NoArg),

    &Opt(&"config", &["c", "cfg"], OptKind::Arg),

    &Opt(&"input", &["i"], OptKind::Arg),
        &Opt(&"fifo-sz", &["udp-fifo-sz", "udp-fifo-size","fifo-size"], OptKind::Arg),
        &Opt(&"out", &["o", "output"], OptKind::Arg),
];

#[allow(dead_code)]
pub struct ConfigOutput {
    url: Url,
}

pub struct ConfigInput {
    id: u64,
    pub url: Url,
    pub udp_fifo_sz: usize,
}

pub struct Config {
    // pub _action: analyze | report | plot
    pub print_help: bool,
    pub print_version: bool,
    pub print_config: bool,

    pub log_level: log::Level,

    pub inputs: Vec<ConfigInput>,
}

impl Config {
    pub(crate) fn parse() -> Result<Config> {
        let mut c = Config {
            print_help: false,
            print_version: false,
            print_config: false,
            log_level: log::Level::Info,

            inputs: Default::default(),
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
                    "input" => c.push_input(value)?,
                    "fifo-sz" => {
                        let udp_fifo_sz = value.parse::<usize>().unwrap();
                        c.inputs.last_mut().and_then(|input| {
                            input.udp_fifo_sz = udp_fifo_sz;
                            Some(input)
                        });
                    }

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

    pub(crate) fn print_help(&self) {
        println!("Video/Audio tool version {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Usage:");
        println!(r#"  va-tool [...] [-arg ...] [--arg[="..."]] [--] [...]"#);
        println!();
        println!("Flags:");
        println!("  -vv, --verbose                 | <bool>    | ... ");
        println!("  -vvv, --very-verbose           | <bool>    | ... ");
        println!("  -i, --intput                   | <str/url> | Where to read from");
        println!("    --fifo-sz                    | <size>    | circular buffer size; result allocaed size");
        println!("                                             . is $(mpeg-ts-packer-size) * $(fifo-size)");
        println!("                                             . mpeg-ts-packer-size is 188");
        println!("  -o, --output, --out            | <str/url> | Where to write to");
        println!();
    }

    pub(crate) fn print_version(&self) {
        println!("version: {}", env!("CARGO_PKG_VERSION"));
    }

    pub(crate) fn print_config(&self) {
        println!("log-level: {}", self.log_level.to_string().to_lowercase());
        println!("inputs:");
        for input in self.inputs.iter() {
            println!("  - id: {}", input.id);
            println!("    url: {}", input.url);
            if input.url.scheme() == "udp" {
                println!("    udp-fifo-sz: {}", input.udp_fifo_sz);
            }
        }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        Ok(())
    }

    fn push_input(&mut self, url_raw: String) -> Result<()> {
        let cfg_input = ConfigInput {
            id: 0,
            url: url_parse(&url_raw)?,
            udp_fifo_sz: 5 * 1000,
        };

        self.inputs.push(cfg_input);

        Ok(())
    }
}

/// patched version of url-parse
/// add udp:// to udp-like host
/// add file:// to file-like paths
fn url_parse<UR: AsRef<str>>(url_raw: UR) -> Result<url::Url> {
    lazy_static! {
        /// 224.0.0.0-224.0.0.255:     "Reserved for special 'well-known' multicast addresses."
        /// 224.0.1.0-238.255.255.255: "Globally-scoped (Internet-wide) multicast addresses."
        /// 239.0.0.0-239.255.255.255: "Administratively-scoped (local) multicast addresses."
        static ref RE_UDP_MCAST_GROUP: Regex = Regex::new(
            r#"(?x)
                ^
                2(?:2[4-9]|3[0-9])
                (?:
                    \.
                    (?:
                            25[0-5]
                        |   2[0-4][0-9]
                        |   1[0-9]{2}
                        |   [1-9][0-9]
                        |   [0-9]
                    )
                ){3}
                "#,
        )
        .unwrap();
    }

    let mut url_raw = url_raw.as_ref().to_string();

    if RE_UDP_MCAST_GROUP.is_match(&url_raw) {
        url_raw.insert_str(0, "udp://");
    } else if url_raw.starts_with('.') || url_raw.starts_with('/') {
        url_raw.insert_str(0, "file://");
    }

    Url::parse(&url_raw).map_err(|err| Error::url_parse(err, url_raw))
}
