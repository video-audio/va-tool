use chrono;
use clap;
use fern;
use log;
use log::info;

fn setup_logger() -> Result<(), fern::InitError> {
    let colors = fern::colors::ColoredLevelConfig::new()
        .trace(fern::colors::Color::Cyan)
        .debug(fern::colors::Color::Magenta)
        .info(fern::colors::Color::Green);

    fern::Dispatch::new()
        .format(move |out, msg, record| {
            out.finish(format_args!(
                "[{}] [{}] {}",
                chrono::Local::now().format("%d/%b/%Y %H:%M:%S"),
                colors.color(record.level()),
                msg
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

fn main() {
    let matches = clap::App::new("V/A tool")
        .version("0.0.3")
        .author("Ivan Egorov <vany.egorov@gmail.com>")
        .about("analyze, dump, mux, demux, encode, decode, filter video/audio streams")
        .arg(
            clap::Arg::with_name("input")
                .index(1)
                .short("i")
                .long("input")
                .help("Sets the input file to use")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    setup_logger().unwrap();

    info!("==> {}", matches.value_of("input").unwrap());
}
