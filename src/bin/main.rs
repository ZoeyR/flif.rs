extern crate flif;
extern crate png;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::fs::File;
use std::io::{Read, Write};
use std::io::BufWriter;

use flif::components::header::Channels;
use flif::error::*;
use flif::Decoder;

use png::HasParameters;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "flif")]
struct Args {
    #[structopt(short = "v", long = "verbose")] verbose: bool,
    #[structopt(subcommand)] cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "decode")]
    Decode {
        #[structopt(short = "i", long = "identify",
                    help = "don't decode, just identify the input FLIF")]
        identify: bool,
        #[structopt(name = "INPUT", help = "Input file")] input: String,
        #[structopt(name = "OUTPUT", help = "Output file, stdout if not present")]
        output:
            Option<String>,
    },
    #[structopt(name = "encode")] Encode {},
}

fn main() {
    let args = Args::from_args();

    let result = match args.cmd {
        Command::Decode {
            identify,
            input,
            output,
        } => decode(identify, &input, output),
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

fn decode(identify: bool, input: &str, output: Option<String>) -> Result<()> {
    let file = File::open(input)?;
    let mut decoder = Decoder::new(file);

    if identify {
        id_file(decoder)
    } else {
        let flif = decoder.decode()?;

        if let Some(output) = output {
            let output_file = File::create(output)?;
            let ref mut w = BufWriter::new(output_file);

            let mut encoder = png::Encoder::new(w, flif.info.header.width as u32, flif.info.header.height as u32);

            let color_type = match flif.info.header.channels {
                Channels::RGBA => png::ColorType::RGBA,
                Channels::RGB => png::ColorType::RGB,
                _ => panic!("unsupported color type"),
            };

            encoder.set(color_type).set(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();

            let data = flif.get_raw_pixels(); // Get the raw pixel array of the FLIF image
            writer.write_image_data(&data).unwrap(); // Save
            Ok(())
        } else {
            std::io::stdout().write(&flif.get_raw_pixels())?;
            Ok(())
        }
        
    }
}

fn id_file<R: Read>(mut decoder: Decoder<R>) -> Result<()> {
    let info = decoder.identify()?;

    if info.header.interlaced {
        println!("interlaced");
    }
    if info.header.animated {
        println!("animated, frames: {}", info.header.num_frames);
    }
    println!("channels: {:?}", info.header.channels);
    println!("dimensions: {}W x {}H", info.header.width, info.header.height);

    Ok(())
}

fn encode() -> Result<()> {
    Err(Error::Unimplemented(
        "flif.rs does not currently support encoding",
    ))
}
