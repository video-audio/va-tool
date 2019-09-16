#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

mod config;
mod error;
mod input;
mod logger;
mod mediacontainer;
mod opt;
mod source;

use std::process;

use crossbeam_channel::{bounded, select, Receiver};
use log::info;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::input::{InputFile, InputUdp};
use crate::mediacontainer::Mediacontainer;
use crate::source::Source;

fn signal_chan() -> Result<Receiver<()>> {
    let (sender, receiver) = bounded(16);

    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })
    .map_err(Error::signal)?;

    Ok(receiver)
}

struct App {
    config: Config,
}

impl App {
    fn new(config: Config) -> App {
        App { config }
    }

    fn start(&self) -> Result<()> {
        for input in self.config.inputs.iter() {
            match input.url.scheme() {
                "udp" => {
                    let mut udp = InputUdp::new(input.url.clone());
                    udp.fifo_sz(input.udp_fifo_sz);

                    let mut source = Source::new(udp);
                    source.start()?;

                    let mc = Mediacontainer::from(&input.url);
                    if mc == Mediacontainer::Ts {
                        // source.add_consumer(ts-demuxer)
                    }
                }
                "file" => {
                    let input = InputFile::new(input.url.clone());

                    let mut source = Source::new(input);
                    source.start()?;
                }
                _ => {}
            };
        }

        let chan = signal_chan()?;
        select! {
            recv(chan) -> _ => {
                info!("(SIGINT) will shutdown!");
            }
        }

        Ok(())
    }
}

/// main with optional Error
fn try_main() -> Result<()> {
    logger::init()?;

    let config = Config::parse().map_err(Error::config)?;

    log::set_max_level(config.log_level.to_level_filter());

    if config.print_help || config.print_version || config.print_config {
        if config.print_help {
            config.print_help()
        }

        if config.print_version {
            config.print_version()
        }

        if config.print_config {
            config.print_config();
        }

        return Ok(());
    }

    config.validate()?;

    let app = App::new(config);
    app.start()?;

    Ok(())
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);

        process::exit(1);
    }
}
