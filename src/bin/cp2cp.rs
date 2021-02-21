use clap::{Arg, App};
use std::{process, io, fs};
use std::io::{BufReader, BufRead, BufWriter, stdout};
use chainpack::{CponReader, ChainPackReader, ChainPackWriter, CponWriter};
use chainpack::Reader;
use chainpack::Writer;

use fern::colors::ColoredLevelConfig;
use colored::Color;
use colored::Colorize;

fn setup_logging(verbosity: Option<& str>) -> Result<Vec<(String, log::LevelFilter)>, fern::InitError> {
    let mut ret: Vec<(String, log::LevelFilter)> = Vec::new();
    let colors = ColoredLevelConfig::new()
        // use builder methods
        .error(Color::BrightRed)
        .warn(Color::BrightMagenta)
        .info(Color::Cyan)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    let mut base_config = fern::Dispatch::new();
    base_config = match verbosity {
        None => {
            ret.push(("".into(), log::LevelFilter::Info));
            base_config
                .level(log::LevelFilter::Info)
        }
        Some(levels) => {
            for level_str in levels.split(',') {
                let parts: Vec<&str> = level_str.split(':').collect();
                let (target, level_abbr) = if parts.len() == 1 {
                    (parts[0], "T")
                } else if parts.len() == 2 {
                    (parts[0], parts[1])
                } else {
                    panic!("Cannot happen");
                };
                let level = match level_abbr {
                    "D" => log::LevelFilter::Debug,
                    "I" => log::LevelFilter::Info,
                    "W" => log::LevelFilter::Warn,
                    "E" => log::LevelFilter::Error,
                    _ => log::LevelFilter::Trace,
                };
                ret.push((target.to_string(), level));
                if target.is_empty() {
                    base_config = base_config.level(level);
                } else {
                    base_config = base_config.level_for(target.to_string(), level);
                }
            }
            base_config
        }
    };
    let stderr_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            let level_color: fern::colors::Color = colors.get_color(&record.level());
            out.finish(format_args!(
                "{}{}{} module: {} {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S.%3f").to_string().green(),
                match record.line() {
                    None => format!("({})", record.target(), ),
                    Some(line) => format!("({}:{})", record.target(), line),
                }.yellow(),
                format!("[{}]", &record.level().as_str()[..1]).color(level_color),
                record.module_path().unwrap_or(""),
                format!("{}", message).color(level_color)
            ))
        })
        .chain(io::stderr());
    base_config
        //.chain(file_config)
        .chain(stderr_config)
        .apply()?;
    Ok(ret)
}

fn main() {
    let matches = App::new("cp2cp")
        .version("0.0.2")
        .author("Fanda Vacek <fanda.vacek@gmail.com>")
        .about("ChainPack to Cpon converter")
        .arg(Arg::with_name("INPUT")
            .help("File to convert")
            .required(false)
            .index(1))
        .arg(Arg::with_name("indent")
            .short("i")
            .long("indent")
            .takes_value(true)
            .help("Indentation string"))
        .arg(Arg::with_name("cponInput")
            // .short("n")
            .long("ip")
            .takes_value(false)
            .help("Input is Cpon"))
        .arg(Arg::with_name("chainpackOutput")
            // .short("n")
            .long("oc")
            .takes_value(false)
            .help("Output is ChainPack"))
        .arg(Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .takes_value(true)
                .help("Verbosity levels for targets, for example: rpcmsg:W or :T"),
        )
        .get_matches();

    let o_file = matches.value_of("INPUT");
    let o_indent = matches.value_of("indent");
    let o_chainpack_output = matches.is_present("chainpackOutput");
    let o_cpon_input = matches.is_present("cponInput");
    let o_verbose = matches.value_of("verbose");

    let levels = setup_logging(o_verbose).expect("failed to initialize logging.");
    log::info!("=====================================================");
    log::info!("{} starting up!", std::module_path!());
    log::info!("=====================================================");
    log::info!("Verbosity levels: {}", levels.iter()
        .map(|(target, level)| format!("{}:{}", target, level))
        .fold(String::new(), |acc, s| if acc.is_empty() { s } else { acc + "," + &s }));

    // log::trace!("trace log test");
    // log::debug!("debug log test");
    // log::info!("info log test");
    // log::warn!("warn log test");
    // log::error!("error log test");
    // log::debug!(target: "rpcmsg", "info with target log test");
    // return;

    let mut reader: Box<dyn BufRead> = match o_file {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => Box::new(BufReader::new(fs::File::open(filename).unwrap()))
    };

    let res = if o_cpon_input {
        let mut rd = CponReader::new(&mut reader);
        rd.read()
    } else {
        let mut rd = ChainPackReader::new(&mut reader);
        rd.read()
    };
    let rv = match res {
        Err(e) => {
            log::error!("Parse input error: {:?}", e);
            process::exit(1);
        }
        Ok(rv) => rv,
    };
    let mut writer = BufWriter::new(stdout());
    let res = if o_chainpack_output {
        let mut wr = ChainPackWriter::new(&mut writer);
        wr.write(&rv)
    } else {
        let mut wr = CponWriter::new(&mut writer);
        if let Some(s) = o_indent {
            if s == "\\t" {
                wr.set_indent("\t");
            } else {
                wr.set_indent(s);
            }
        }
        wr.write(&rv)
    };
    if let Err(e) = res {
        log::error!("Write output error: {:?}", e);
        process::exit(1);
    }
}

