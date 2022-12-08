use std::io::Cursor;

use image::ImageError;
use lithophane::{InvalidPointsError, LithophaneGenerator};
use thiserror::Error;
use wasm_bindgen::{prelude::wasm_bindgen, JsError};

pub mod lithophane;

#[wasm_bindgen]
pub fn generate_lithophane(
	x_expression: &str,
	y_expression: &str,
	z_expression: &str,
	image: Vec<u8>,
	white_depth: f32,
	black_depth: f32,
) -> Result<Vec<u8>, JsError> {
	let image = image::io::Reader::new(Cursor::new(image)).with_guessed_format().map_err(|e| ImageError::IoError(e))?.decode()?;

	let x_expression =
		x_expression.parse::<meval::Expr>().and_then(|e| e.bind4("x", "y", "w", "h")).map_err(|e| Error::MevalError("x".to_string(), e))?;
	let y_expression =
		y_expression.parse::<meval::Expr>().and_then(|e| e.bind4("x", "y", "w", "h")).map_err(|e| Error::MevalError("y".to_string(), e))?;
	let z_expression =
		z_expression.parse::<meval::Expr>().and_then(|e| e.bind4("x", "y", "w", "h")).map_err(|e| Error::MevalError("z".to_string(), e))?;

	fn meval_f32_wrapper(f: impl Fn(f64, f64, f64, f64) -> f64) -> impl Fn(f32, f32, f32, f32) -> f32 {
		move |x: f32, y: f32, w: f32, h: f32| -> f32 { f(x as f64, y as f64, w as f64, h as f64) as f32 }
	}

	let lithophane_generator = LithophaneGenerator::new(
		meval_f32_wrapper(x_expression),
		meval_f32_wrapper(y_expression),
		meval_f32_wrapper(z_expression),
		image.into_luma8(),
		white_depth,
		black_depth,
	);

	Ok(lithophane_generator.generate_lithophane()?.as_binary())
}

#[derive(Error, Debug)]
pub enum Error {
	#[error("invalid {0} expression: {1}")]
	MevalError(String, meval::Error),
	#[error(transparent)]
	InvalidPointsError(#[from] InvalidPointsError),
	#[error("error with image: {0}")]
	ImageError(#[from] ImageError),
}
