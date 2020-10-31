use clap::{Arg, App};
use std::{process, io, fs, env};
use std::io::{BufReader, BufRead, BufWriter, stdout};
use chainpackrpc::{CponReader, ChainPackReader, ChainPackWriter, CponWriter};
use chainpackrpc::reader::Reader;
use chainpackrpc::writer::Writer;
use env_logger::{Builder, WriteStyle};
use chrono::Local;
use std::io::Write;
use std::path::Path;
use std::ffi::OsString;
use env_logger;

fn main() {
    //env_logger::init();
    env_logger::builder()
        .format(|buf, record| {
            fn short_name(file_path: &str) -> &str {
                let base_name = Path::new(file_path).file_name();
                match base_name {
                    Some(n) => {
                        match n.to_str() {
                            Some(s) => s,
                            None => "",
                        }
                    },
                    None => "",
                }
            }
            let mut style = buf.style();
            style.set_color(env_logger::fmt::Color::Green);
            write!(buf,
                   "{}",
                   style.value(Local::now().format("%H:%M:%S.%3f")),
            );
            style.set_color(env_logger::fmt::Color::Yellow);
            write!(buf, "{}", style.value(&format!("[{}:{}]", short_name(record.file().unwrap_or("")),
                     record.line().unwrap_or(0))),
            );
            match record.level() {
                log::Level::Error => {style.set_color(env_logger::fmt::Color::Red).set_bold(true);}
                log::Level::Warn => {style.set_color(env_logger::fmt::Color::Magenta).set_bold(true);}
                log::Level::Info => {style.set_color(env_logger::fmt::Color::Cyan);}
                log::Level::Debug => {style.set_color(env_logger::fmt::Color::White);}
                log::Level::Trace => {style.set_color(env_logger::fmt::Color::White);}
            }
            fn level_abbr(level: log::Level) -> char {
                let s = format!("{}", level).chars().next();
                match s {
                    Some(c) => c,
                    None => ' ',
                }
            }
            writeln!(buf, "{}", style.value(&format!("|{}| {}",
                                                     level_abbr(record.level()),
                                                     record.args()))
            )
        })
        .write_style(WriteStyle::Auto)
        .init();

    let matches = App::new("cp2cp")
        .version("0.0.1")
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
        .get_matches();

    // log::trace!("trace log test");
    // log::debug!("debug log test");
    // log::info!("info log test");
    // log::warn!("warn log test");
    // log::error!("error log test");

    let o_file = matches.value_of("INPUT");
    let o_indent = matches.value_of("indent");
    let o_chainpack_output = matches.is_present("chainpackOutput");
    let o_cpon_input = matches.is_present("cponInput");

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
            eprintln!("Parse input error: {:?}", e);
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
        eprintln!("Write output error: {:?}", e);
        process::exit(1);
    }
}

