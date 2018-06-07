extern crate flif;
extern crate png;
extern crate structopt;
#[macro_use] extern crate structopt_derive;

use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};

use flif::colors::ColorSpace;
use flif::{Result, Error};
use flif::{Decoder, FlifInfo};

use png::HasParameters;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "flif")]
struct Args {
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "decode")]
    Decode {
        #[structopt(short = "i", long = "identify",
                    help = "don't decode, just identify the input FLIF")]
        identify: bool,
        #[structopt(name = "INPUT", help = "Input file")]
        input: String,
        #[structopt(name = "OUTPUT", help = "Output file, stdout if not present")]
        output: Option<String>,
    },
    #[structopt(name = "encode")]
    Encode {},
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
    let reader = BufReader::new(File::open(input)?);
    let decoder = Decoder::new(reader)?;

    if identify {
        id_file(decoder.info());
    } else {
        let image = decoder.decode_image()?;

        if let Some(output) = output {
            let output_file = File::create(output)?;
            let w = &mut BufWriter::new(output_file);

            let info = image.info();

            let mut encoder = png::Encoder::new(
                w,
                info.header.width as u32,
                info.header.height as u32,
            );

            let color_type = match info.header.channels {
                ColorSpace::RGBA => png::ColorType::RGBA,
                ColorSpace::RGB => png::ColorType::RGB,
                ColorSpace::Monochrome => png::ColorType::Grayscale,
            };

            encoder.set(color_type).set(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();

            // Get the raw pixel array of the FLIF image
            let data = image.get_raw_pixels();
            // Save as PNG
            writer.write_image_data(&data).unwrap();
        } else {
            std::io::stdout().write_all(&image.get_raw_pixels())?;
        }
    }
    Ok(())
}

fn id_file(info: &FlifInfo) {
    if info.header.interlaced {
        println!("interlaced");
    }
    if info.header.num_frames != 1 {
        println!("animated, frames: {}", info.header.num_frames);
    }
    println!("channels: {:?}", info.header.channels);
    println!(
        "dimensions: {} x {}",
        info.header.width, info.header.height
    );

    let len = info.second_header.transformations.len();
    if len != 0 {
        println!("transformations:");

        for transformation in info.second_header.transformations[..len - 1].iter() {
            println!("├── {}", transformation);
        }
        println!("└── {}", info.second_header.transformations[len - 1]);
    }
}

fn encode() -> Result<()> {
    Err(Error::Unimplemented(
        "flif.rs does not currently support encoding",
    ))
}
