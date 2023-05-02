use image::GrayImage;
use pk_stl::{
	geometry::{Triangle, Vec3},
	StlModel,
};
use thiserror::Error;

/// Create a lithophane using three functions to translate x and y coordinates from an image into x,y,z coordinates for a mesh
pub fn generate_lithophane<F: Fn(f32, f32, f32, f32) -> f32>(
	x_fn: F,
	y_fn: F,
	z_fn: F,
	image: GrayImage,
	white_depth: f32,
	black_depth: f32,
) -> Result<StlModel, InvalidPointsError> {
	let point_cloud = generate_point_cloud(x_fn, y_fn, z_fn, image.width(), image.height(), 1)?;
	let mesh = generate_lithophane_mesh(point_cloud, image, white_depth, black_depth)?;
	Ok(StlModel {
		header: String::new(),
		triangles: mesh,
	})
}

/// Create a flat preview mesh using three functions to translate x and y coordinates from an image into x,y,z coordinates for the mesh
/// The step argument allows stepping by that many vertices at a time, generating a lower resolution preview in a shorter amount of time
pub fn generate_preview<F: Fn(f32, f32, f32, f32) -> f32>(
	x_fn: F,
	y_fn: F,
	z_fn: F,
	width: u32,
	height: u32,
	step: u32,
) -> Result<StlModel, InvalidPointsError> {
	let point_cloud = generate_point_cloud(x_fn, y_fn, z_fn, width, height, step)?;

	let width_usize = point_cloud.width as usize;
	let height_usize = point_cloud.height as usize;

	let num_triangles = (width_usize - 1) * (height_usize - 1) * 2;
	let mut triangles = Vec::with_capacity(num_triangles);

	// Remember that the image origin is top left, so y_i = 0, x_i = 0 is the top left of the image

	for y_i in 0..point_cloud.height as usize - 1 {
		for x_i in 0..point_cloud.width as usize - 1 {
			triangles.push(three_points_to_triangle([
				point_cloud.vertices[y_i * width_usize + x_i],
				point_cloud.vertices[(y_i + 1) * width_usize + x_i],
				point_cloud.vertices[(y_i + 1) * width_usize + x_i + 1],
			])?);
			triangles.push(three_points_to_triangle([
				point_cloud.vertices[y_i * width_usize + x_i],
				point_cloud.vertices[(y_i + 1) * width_usize + x_i + 1],
				point_cloud.vertices[y_i * width_usize + x_i + 1],
			])?);
		}
	}
	Ok(StlModel {
		header: String::new(),
		triangles,
	})
}

struct PointCloud {
	pub vertices: Vec<Vec3>,
	/// The normals of the vertices, in respect to their upper, lower, left, and right points
	pub vertex_normals: Vec<Vec3>,
	pub width: u32,
	pub height: u32,
}

/// Generates a point cloud from a set of equations
fn generate_point_cloud<F: Fn(f32, f32, f32, f32) -> f32>(
	x_fn: F,
	y_fn: F,
	z_fn: F,
	width: u32,
	height: u32,
	step: u32,
) -> Result<PointCloud, InvalidPointsError> {
	// Generate vertices with an extra border that will be used to calculate normals
	let mut vertices = Vec::with_capacity((width as usize + 2) * (height as usize + 2));

	let width_f32 = width as f32;
	let height_f32 = height as f32;

	// TODO check parsed equations to optimize case where equation doesn't reference x or y. For instance, if the x_fn doesn't reference y at all,
	// then it only needs to be run for one row, then the results can be copied for each value of y_i. This would require moving the meval stuff
	// into the lithophane generator.

	/// Create a Vec<i64> from -step to length-1 inclusive, stepping by step, but with an extra element at the end to reach exactly length if
	/// necessary, and with another element after that with the same difference (eg length=15 step=4 results in -4,0,4,8,12,14,16).
	fn step_iter_with_size(length: u32, step: u32) -> Vec<i64> {
		let mut v = Vec::with_capacity(((length - 1 + step - 1) / step + 3) as usize);

		let length_i64 = length as i64;
		let step_i64 = step as i64;

		v.extend((-step_i64..length_i64).step_by(step as usize));

		if (length - 1) % step != 0 {
			v.push(length_i64 - 1);
		}
		v.push((length_i64 - 1) * 2 - v[v.len() - 2]);

		v
	}

	let width_range = step_iter_with_size(width, step);
	let ewc = width_range.len(); // Extended width count
	let height_range = step_iter_with_size(height, step);
	let ehc = height_range.len(); // Extended height count

	for y_i in height_range.iter().copied() {
		for x_i in width_range.iter().copied() {
			vertices.push(Vec3 {
				x: (x_fn)(x_i as f32, y_i as f32, width_f32, height_f32),
				y: (y_fn)(x_i as f32, y_i as f32, width_f32, height_f32),
				z: (z_fn)(x_i as f32, y_i as f32, width_f32, height_f32),
			});
		}
	}

	let wc = ewc - 2; // Actual width count
	let hc = ehc - 2; // Actual height count
	let mut normals = Vec::with_capacity(wc * hc);

	for y_i in 0..hc {
		for x_i in 0..wc {
			let v = vertices[(y_i + 1) * ewc + 1 + x_i];
			// lower and right vectors
			let norm1 = normalize_to_unit_vector(cross_product(
				vertices[(y_i + 2) * ewc + 1 + x_i] - v,
				vertices[(y_i + 1) * ewc + 2 + x_i] - v,
			))?;
			// upper and left vectors
			let norm2 = normalize_to_unit_vector(cross_product(vertices[y_i * ewc + 1 + x_i] - v, vertices[(y_i + 1) * ewc + x_i] - v))?;

			normals.push(normalize_to_unit_vector(norm1 + norm2)?);
		}
	}

	Ok(PointCloud {
		vertices: vertices
			.into_iter()
			.enumerate()
			.filter(|&(i, _)| {
				i >= ewc // exclude extra bottom row
				&& i < ewc * (ehc - 1) // exclude extra top row
				&& i % ewc != 0 // exclude extra left row
				&& i % ewc != ewc - 1 // exclude extra right row
			})
			.map(|(_, v)| v)
			.collect::<Vec<_>>(),
		vertex_normals: normals,
		width: wc as u32,
		height: hc as u32,
	})
}

fn generate_lithophane_mesh(
	point_cloud: PointCloud,
	image: GrayImage,
	white_depth: f32,
	black_depth: f32,
) -> Result<Vec<Triangle>, InvalidPointsError> {
	let width = point_cloud.width as usize;
	let height = point_cloud.height as usize;

	// Triangles for backing mesh and connecting pixels
	let mut num_triangles = (width - 1) * (height - 1) * 4;
	// Triangles to enclose pixels to mesh
	num_triangles += 4 * (width - 1) + 4 * (height - 1);

	let mut triangles = Vec::with_capacity(num_triangles);

	// Remember that the image origin is top left, so y_i = 0, x_i = 0 is the top left of the image

	// Generate triangles for backing mesh
	for y_i in 0..height - 1 {
		for x_i in 0..width - 1 {
			triangles.push(three_points_to_triangle([
				point_cloud.vertices[y_i * width + x_i],
				point_cloud.vertices[(y_i + 1) * width + x_i + 1],
				point_cloud.vertices[(y_i + 1) * width + x_i],
			])?);
			triangles.push(three_points_to_triangle([
				point_cloud.vertices[y_i * width + x_i],
				point_cloud.vertices[y_i * width + x_i + 1],
				point_cloud.vertices[(y_i + 1) * width + x_i + 1],
			])?);
		}
	}

	// Calculate vertices for pixels
	let get_px_depth = |gray_value: u8| -> f32 { white_depth + (255 - gray_value) as f32 / 255.0 * (black_depth - white_depth) };
	let mut px_vertices = Vec::with_capacity(width * height);
	for i in 0..width * height {
		let depth = get_px_depth(image.get_pixel(i as u32 % image.width(), i as u32 / image.width()).0[0]);
		px_vertices.push(point_cloud.vertices[i] + point_cloud.vertex_normals[i] * depth);
	}

	// Generate triangles for pixels
	for y_i in 0..height - 1 {
		for x_i in 0..width - 1 {
			triangles.push(three_points_to_triangle([
				px_vertices[y_i * width + x_i],
				px_vertices[(y_i + 1) * width + x_i],
				px_vertices[(y_i + 1) * width + x_i + 1],
			])?);
			triangles.push(three_points_to_triangle([
				px_vertices[y_i * width + x_i],
				px_vertices[(y_i + 1) * width + x_i + 1],
				px_vertices[y_i * width + x_i + 1],
			])?);
		}
	}

	// Generate triangles to connect the top of the image to backing mesh
	for x_i in 0..width - 1 {
		triangles.push(three_points_to_triangle([
			point_cloud.vertices[x_i],
			px_vertices[x_i],
			px_vertices[x_i + 1],
		])?);
		triangles.push(three_points_to_triangle([
			point_cloud.vertices[x_i],
			px_vertices[x_i + 1],
			point_cloud.vertices[x_i + 1],
		])?);
	}

	// Generate triangles to connect the bottom of the image to backing mesh
	for x_i in (height - 1) * width..height * width - 1 {
		triangles.push(three_points_to_triangle([
			point_cloud.vertices[x_i],
			px_vertices[x_i + 1],
			px_vertices[x_i],
		])?);
		triangles.push(three_points_to_triangle([
			point_cloud.vertices[x_i],
			point_cloud.vertices[x_i + 1],
			px_vertices[x_i + 1],
		])?);
	}

	// Generate triangles to connect the left side of the image to backing mesh
	for y_i in 0..height - 1 {
		let current_index = y_i * width;
		let lower_index = (y_i + 1) * width;
		triangles.push(three_points_to_triangle([
			point_cloud.vertices[current_index],
			point_cloud.vertices[lower_index],
			px_vertices[lower_index],
		])?);
		triangles.push(three_points_to_triangle([
			point_cloud.vertices[current_index],
			px_vertices[lower_index],
			px_vertices[current_index],
		])?);
	}

	// Generate triangles to connect the right side of the image to backing mesh
	for y_i in 0..height - 1 {
		let current_index = (y_i + 1) * width - 1;
		let lower_index = (y_i + 2) * width - 1;
		triangles.push(three_points_to_triangle([
			point_cloud.vertices[current_index],
			px_vertices[lower_index],
			point_cloud.vertices[lower_index],
		])?);
		triangles.push(three_points_to_triangle([
			point_cloud.vertices[current_index],
			px_vertices[current_index],
			px_vertices[lower_index],
		])?);
	}

	Ok(triangles)
}

#[derive(Error, Debug)]
#[error("all three points for this triangle are in the same line")]
pub struct InvalidPointsError {}

/// Turn three points into a triangle, calculating the normal by counterclockwise ordering.
fn three_points_to_triangle(points: [Vec3; 3]) -> Result<Triangle, InvalidPointsError> {
	Ok(Triangle {
		normal: normalize_to_unit_vector(cross_product(points[1] - points[0], points[2] - points[0]))?,
		vertices: [points[0], points[1], points[2]],
	})
}

fn cross_product(a: Vec3, b: Vec3) -> Vec3 {
	[a.y * b.z - b.y * a.z, a.z * b.x - b.z * a.x, a.x * b.y - b.x * a.y].into()
}

/// Will return Err if the vector has no length
fn normalize_to_unit_vector(v: Vec3) -> Result<Vec3, InvalidPointsError> {
	let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
	if length == 0.0 {
		return Err(InvalidPointsError {});
	}

	Ok([v.x / length, v.y / length, v.z / length].into())
}
