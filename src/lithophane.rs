use image::GrayImage;
use pk_stl::{
	geometry::{Triangle, Vec3},
	StlModel,
};
use thiserror::Error;

pub struct LithophaneGenerator<F: Fn(f32, f32, f32, f32) -> f32> {
	x_fn: F,
	y_fn: F,
	z_fn: F,
	image: GrayImage,
	white_depth: f32,
	black_depth: f32,
}

struct PointCloud {
	pub vertices: Vec<Vec3>,
	/// The normals of the vertices, in respect to their upper, lower, left, and right points
	pub vertex_normals: Vec<Vec3>,
}

impl<F: Fn(f32, f32, f32, f32) -> f32> LithophaneGenerator<F> {
	/// Create a LithophaneGenerator that uses three functions to translate x and y coordinates from an image into x,y,z coordinates for a mesh
	pub fn new(x_fn: F, y_fn: F, z_fn: F, image: GrayImage, white_depth: f32, black_depth: f32) -> Self {
		Self {
			x_fn,
			y_fn,
			z_fn,
			image,
			white_depth,
			black_depth,
		}
	}

	pub fn generate_lithophane(&self) -> Result<StlModel, InvalidPointsError> {
		let vertices = self.calculate_vertices();
		let point_cloud = self.generate_point_cloud(vertices)?;
		let mesh = self.generate_mesh(point_cloud)?;
		Ok(StlModel {
			header: String::new(),
			triangles: mesh,
		})
	}

	/// Calculate vertices using the callbacks. Includes an extra 1 pixel border around the image for use in calculating vertex normals.
	fn calculate_vertices(&self) -> Vec<Vec3> {
		let mut vertices = Vec::with_capacity((self.image.width() as usize + 2) * (self.image.height() as usize + 2));

		let image_width_f32 = self.image.width() as f32;
		let image_height_f32 = self.image.height() as f32;

		// TODO check parsed equations to optimize case where equation doesn't reference x or y. For instance, if the x_fn doesn't reference y at all,
		// then it only needs to be run for one row, then the results can be copied for each value of y_i. This would require moving the meval stuff
		// into the lithophane generator.

		for y_i in -1..(self.image.height() as i64) + 1 {
			for x_i in -1..(self.image.width() as i64) + 1 {
				vertices.push(Vec3 {
					x: (self.x_fn)(x_i as f32, y_i as f32, image_width_f32, image_height_f32),
					y: (self.y_fn)(x_i as f32, y_i as f32, image_width_f32, image_height_f32),
					z: (self.z_fn)(x_i as f32, y_i as f32, image_width_f32, image_height_f32),
				});
			}
		}

		vertices
	}

	fn generate_point_cloud(&self, vertices: Vec<Vec3>) -> Result<PointCloud, InvalidPointsError> {
		let image_width = self.image.width() as usize;
		let image_height = self.image.height() as usize;

		let mut normals = Vec::with_capacity(image_width * image_height);

		let vw = image_width + 2;

		for y_i in 0..image_height {
			for x_i in 0..image_width {
				let v = vertices[(y_i + 1) * vw + 1 + x_i];
				// lower and right vectors
				let norm1 = normalize_to_unit_vector(cross_product(
					vertices[(y_i + 2) * vw + 1 + x_i] - v,
					vertices[(y_i + 1) * vw + 2 + x_i] - v,
				))?;
				// upper and left vectors
				let norm2 = normalize_to_unit_vector(cross_product(vertices[y_i * vw + 1 + x_i] - v, vertices[(y_i + 1) * vw + x_i] - v))?;

				normals.push(normalize_to_unit_vector(norm1 + norm2)?);
			}
		}

		Ok(PointCloud {
			vertices: vertices
				.into_iter()
				.enumerate()
				.filter(|&(i, _)| {
					let image_width = self.image.width() as usize;
					let image_height = self.image.height() as usize;

					i >= image_width + 2 // exclude extra bottom row
                    && i < (image_width + 2) * (image_height + 1) // exclude extra top row
                    && i % (image_width + 2) != 0 // exclude extra left row
                    && i % (image_width + 2) != image_width + 1 // exclude extra right row
				})
				.map(|(_, v)| v)
				.collect::<Vec<_>>(),
			vertex_normals: normals,
		})
	}

	fn generate_mesh(&self, point_cloud: PointCloud) -> Result<Vec<Triangle>, InvalidPointsError> {
		let image_width = self.image.width() as usize;
		let image_height = self.image.height() as usize;

		// Triangles for backing mesh and connecting pixels
		let mut num_triangles = (image_width - 1) * (image_height - 1) * 4;
		// Triangles to enclose pixels to mesh
		num_triangles += 4 * (image_width - 1) + 4 * (image_height - 1);

		let mut triangles = Vec::with_capacity(num_triangles as usize);

		// Generate triangles for backing mesh
		for y_i in 0..image_height - 1 {
			for x_i in 0..image_width - 1 {
				triangles.push(three_points_to_triangle([
					point_cloud.vertices[y_i * image_width + x_i],
					point_cloud.vertices[(y_i + 1) * image_width + x_i],
					point_cloud.vertices[(y_i + 1) * image_width + x_i + 1],
				])?);
				triangles.push(three_points_to_triangle([
					point_cloud.vertices[y_i * image_width + x_i],
					point_cloud.vertices[(y_i + 1) * image_width + x_i + 1],
					point_cloud.vertices[y_i * image_width + x_i + 1],
				])?);
			}
		}

		// Calculate vertices for pixels
		let get_px_depth = |gray_value: u8| -> f32 { self.white_depth + (255 - gray_value) as f32 / 255.0 * (self.black_depth - self.white_depth) };
		let mut px_vertices = Vec::with_capacity(image_width * image_height);
		for i in 0..self.image.width() * self.image.height() {
			let depth = get_px_depth(self.image.get_pixel(i % self.image.width(), i / self.image.width()).0[0]);
			px_vertices.push(point_cloud.vertices[i as usize] + point_cloud.vertex_normals[i as usize] * depth);
		}

		// Generate triangles for pixels
		for y_i in 0..image_height - 1 {
			for x_i in 0..image_width - 1 {
				triangles.push(three_points_to_triangle([
					px_vertices[y_i * image_width + x_i],
					px_vertices[(y_i + 1) * image_width + x_i],
					px_vertices[(y_i + 1) * image_width + x_i + 1],
				])?);
				triangles.push(three_points_to_triangle([
					px_vertices[y_i * image_width + x_i],
					px_vertices[(y_i + 1) * image_width + x_i + 1],
					px_vertices[y_i * image_width + x_i + 1],
				])?);
			}
		}

		// Generate triangles to connect the top of the image to backing mesh
		for x_i in 0..image_width - 1 {
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
		for x_i in (image_height - 1) * image_width..image_height * image_width - 1 {
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
		for y_i in 0..image_height - 1 {
			let current_index = y_i * image_width;
			let lower_index = (y_i + 1) * image_width;
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
		for y_i in 0..image_height - 1 {
			let current_index = (y_i + 1) * image_width - 1;
			let lower_index = (y_i + 2) * image_width - 1;
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

/// Will return None if the vector has no length
fn normalize_to_unit_vector(v: Vec3) -> Result<Vec3, InvalidPointsError> {
	let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
	if length == 0.0 {
		return Err(InvalidPointsError {});
	}

	Ok([v.x / length, v.y / length, v.z / length].into())
}
