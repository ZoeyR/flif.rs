extern crate flif;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::fs::File;
use std::io::Read;

use flif::error::*;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "flif")]
struct Args {
    #[structopt(short = "v", long = "verbose")] verbose: bool,
    #[structopt(name = "INPUT", help = "Input file")] input: String,
    #[structopt(name = "OUTPUT", help = "Output file, stdout if not present")]
    output:
        Option<String>,
    #[structopt(subcommand)] cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "decode")]
    Decode {
        #[structopt(short = "i", long = "identify",
                    help = "don't decode, just identify the input FLIF")]
        identify: bool,
    },
    #[structopt(name = "encode")] Encode {},
}

fn main() {
    let args = Args::from_args();

    let result = match args.cmd {
        Command::Decode { identify } => decode(identify, &args.input, args.output),
        Command::Encode { .. } => encode(),
    };

    std::process::exit(match result {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    });
}

fn decode(identify: bool, input: &str, _output: Option<String>) -> Result<()> {
    let file = File::open(input)?;

    if identify {
        id_file(file)
    } else {
        Err(Error::Unimplemented(
            "decoding pixel data is not supported at this time",
        ))
    }
}

fn id_file<R: Read>(reader: R) -> Result<()> {
    let header = flif::components::header::Header::from_reader(reader)?;

    if header.interlaced {
        println!("interlaced");
    }
    if header.animated {
        println!("animated, frames: {}", header.num_frames);
    }
    println!("channels: {:?}", header.channels);
    println!("dimensions: {}W x {}H", header.width, header.height);

    Ok(())
}

fn encode() -> Result<()> {
    Err(Error::Unimplemented(
        "flif.rs does not currently support encoding",
    ))
}
