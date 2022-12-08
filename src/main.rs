use std::{fs::OpenOptions, io::Write, process::ExitCode};

use clap::Parser;

use lithophane_creator::lithophane::LithophaneGenerator;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
	#[arg(short, long)]
	input: String,
	#[arg(short, long)]
	output: String,
	x_expression: String,
	y_expression: String,
	z_expression: String,
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

	let mut output_file = match OpenOptions::new().create_new(true).write(true).open(&cli.output) {
		Ok(f) => f,
		Err(e) => {
			eprintln!("Error opening output file \"{}\": {}", cli.output, e);
			return ExitCode::FAILURE;
		},
	};

	let x_expression = match cli.x_expression.parse::<meval::Expr>().and_then(|e| e.bind4("x", "y", "w", "h")) {
		Ok(e) => e,
		Err(e) => {
			eprintln!("Invalid x expression: {}", e);
			return ExitCode::FAILURE;
		},
	};
	let y_expression = match cli.y_expression.parse::<meval::Expr>().and_then(|e| e.bind4("x", "y", "w", "h")) {
		Ok(e) => e,
		Err(e) => {
			eprintln!("Invalid y expression: {}", e);
			return ExitCode::FAILURE;
		},
	};
	let z_expression = match cli.z_expression.parse::<meval::Expr>().and_then(|e| e.bind4("x", "y", "w", "h")) {
		Ok(e) => e,
		Err(e) => {
			eprintln!("Invalid z expression: {}", e);
			return ExitCode::FAILURE;
		},
	};

	fn meval_f32_wrapper(f: impl Fn(f64, f64, f64, f64) -> f64) -> impl Fn(f32, f32, f32, f32) -> f32 {
		move |x: f32, y: f32, w: f32, h: f32| -> f32 { f(x as f64, y as f64, w as f64, h as f64) as f32 }
	}

	let lithophane_generator = LithophaneGenerator::new(
		meval_f32_wrapper(x_expression),
		meval_f32_wrapper(y_expression),
		meval_f32_wrapper(z_expression),
		image.into_luma8(),
		0.5,
		3.0,
	);

	let lithophane = match lithophane_generator.generate_lithophane() {
		Ok(l) => l,
		Err(e) => {
			eprintln!("Error generating lithophane: {}", e);
			return ExitCode::FAILURE;
		},
	};

	if let Err(e) = output_file.write_all(&lithophane.as_binary()) {
		eprintln!("Error saving lithophane to \"{}\": {}", cli.output, e);
		return ExitCode::FAILURE;
	}

	ExitCode::SUCCESS
}
