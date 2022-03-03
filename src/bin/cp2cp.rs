use std::{process, io, fs};
use std::io::{BufReader, BufRead, BufWriter, stdout};
use std::path::PathBuf;
use flexi_logger::Logger;
use chainpack::{CponReader, ChainPackReader, ChainPackWriter, CponWriter};
use chainpack::Reader;
use chainpack::Writer;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cp2cp", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = "ChainPack to Cpon and back utility")]
struct Cli {
    #[structopt(short, long, help = "Cpon indentation string")]
    indent: Option<String>,
    #[structopt(long = "--ip", help = "Cpon input")]
    cpon_input: bool,
    #[structopt(long = "--oc", help = "ChainPack output")]
    chainpack_output: bool,
    /// File to process
    #[structopt(name = "FILE", parse(from_os_str))]
    file: Option<PathBuf>,
}

fn main() {

    Logger::try_with_env().unwrap().start().unwrap();

    // Parse command line arguments
    let cli = Cli::from_args();

    //log::info!("=====================================================");
    //log::info!("{} starting up!", std::module_path!());
    //log::info!("=====================================================");
    //log::info!("Verbosity levels: {}", verbosity_string);

    let mut reader: Box<dyn BufRead> = match cli.file {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => Box::new(BufReader::new(fs::File::open(filename).unwrap()))
    };

    let res = if cli.cpon_input {
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
    let res = if cli.chainpack_output {
        let mut wr = ChainPackWriter::new(&mut writer);
        wr.write(&rv)
    } else {
        let mut wr = CponWriter::new(&mut writer);
        if let Some(s) = cli.indent {
            if s == "\\t" {
                wr.set_indent("\t".as_bytes());
            } else {
                wr.set_indent(s.as_bytes());
            }
        }
        wr.write(&rv)
    };
    if let Err(e) = res {
        eprintln!("Write output error: {:?}", e);
        process::exit(1);
    }
}

