// #![feature(stmt_expr_attributes)]

#[macro_use]
extern crate bitflags;

mod opt;

use std::env;

use chrono;
use fern;
use log;

use opt::{Opt, OptKind};

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(move |out, msg, record| {
            out.finish(format_args!(
                "[{}] [{}] {}",
                chrono::Local::now().format("%d/%b/%Y %H:%M:%S"),
                record.level(),
                msg
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

const OPTS: &[&opt::Opt] = &[
    &Opt("verbose", &["vv", "verbose"], OptKind::NoArg),
    &Opt("very-verbose", &["vvv", "very-verbose"], OptKind::NoArg),
    &Opt("daemonize", &["daemonize", "background"], OptKind::NoArg),
    &Opt("foreground", &["foreground"], OptKind::NoArg),
    /**/
    &Opt("cfg", &["c", "cfg", "config"], OptKind::Arg),
    /**/
    &Opt("input", &["i", "input"], OptKind::Arg),
    /**/ &Opt("id", &["id"], OptKind::Arg),
    /**/ &Opt("name", &["name"], OptKind::Arg),
    /**/ &Opt("map", &["m", "map"], OptKind::Arg),
];

fn main() {
    setup_logger().unwrap();

    let matcher = opt::Matcher::new(env::args().skip(1).collect(), OPTS);

    matcher.into_iter().for_each(|r| println!("{:?}", r))
}
