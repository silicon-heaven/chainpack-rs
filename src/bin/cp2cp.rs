use clap::{Arg, App};
use std::{process, io, fs};
use std::fs::File;
use std::io::{BufReader, Read, BufRead, BufWriter, stdout};
use chainpackrpc::{CponReader, ChainPackReader, ChainPackWriter, CponWriter};
use chainpackrpc::reader::Reader;
use chainpackrpc::writer::Writer;

fn main() {
    env_logger::init();
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


    let o_file = matches.value_of("INPUT");
    let o_indent = matches.value_of("intent");
    let o_chainpack_output = matches.is_present("chainpackOutput");
    let o_cpon_input = matches.is_present("cponInput");

    let mut reader: Box<BufRead> = match o_file {
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

