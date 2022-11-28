use std::{process::{self, ExitCode}, fs::{self, OpenOptions}, io::Write};

use clap::Parser;
use image;

use crate::lithophane::{rectangle::RectangularLithophaneGenerator, LithophaneGenerator};

mod lithophane;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: String,
    #[arg(short = 'l', long, default_value = "0.1")]
    edge_length: f32,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let image = match image::open(&cli.input) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Error opening image file \"{}\": {}", cli.input, e);
            return ExitCode::FAILURE;
        },
    };

    let lithophane_generator = RectangularLithophaneGenerator {
        edge_length: cli.edge_length,
        white_height: 0.5,
        black_height: 3.0,
        image: image.into_luma8(),
    };

    let lithophane = lithophane_generator.generate_lithophane();

    let mut output_file = match OpenOptions::new().create_new(true).write(true).open(&cli.output) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening output file \"{}\": {}", cli.output, e);
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = output_file.write_all(&lithophane.as_binary()) {
        eprintln!("Error saving lithophane to \"{}\": {}", cli.output, e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
