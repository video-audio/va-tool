#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

mod config;
mod logger;
mod opt;
mod result;

use std::process;

use config::Config;
use result::Result;

/// main with optional Error
fn try_main() -> Result<()> {
    logger::init()?;

    let cfg = Config::parse()?;

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

    // config.validate();

    Ok(())
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);

        process::exit(1);
    }
}
