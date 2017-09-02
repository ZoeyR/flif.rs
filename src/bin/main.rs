extern crate structopt;
#[macro_use]
extern crate structopt_derive;

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

    match args.cmd {
        Command::Decode { .. } => {}
        Command::Encode { .. } => eprintln!("encoding is not implemented at the moment"),
    }
}
