#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

mod config;
mod error;
mod logger;
mod opt;

use std::process;

use config::Config;
use error::{Error, Result};

/// main with optional Error
fn try_main() -> Result<()> {
    logger::init()?;

    let cfg = Config::parse().map_err(Error::config)?;

    log::set_max_level(cfg.log_level.to_level_filter());

    if cfg.print_help || cfg.print_version || cfg.print_config {
        if cfg.print_help {
            cfg.print_help()
        }

        if cfg.print_version {
            cfg.print_version()
        }

        if cfg.print_config {
            cfg.print_config();
        }
    }

    cfg.validate()?;

    Ok(())
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);

        process::exit(1);
    }
}
