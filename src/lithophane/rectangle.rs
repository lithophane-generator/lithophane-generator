use image::GrayImage;
use pk_stl::{StlModel, geometry::Triangle};

use crate::lithophane::three_points_to_triangle;

use super::{LithophaneGenerator, four_points_to_triangles};

pub struct RectangularLithophaneGenerator {
    pub edge_length: f32,
    pub white_height: f32,
    pub black_height: f32,
    pub image: GrayImage,
}

impl LithophaneGenerator for RectangularLithophaneGenerator {
    fn generate_lithophane(&self) -> StlModel {
        let image_width = self.image.width();
        let image_height = self.image.height();

        if image_width < 2 || image_height < 2 {
            return StlModel {
                header: String::new(),
                triangles: vec![],
            };
        }

        // Triangles for pixels
        let mut num_triangles = (image_width - 1) * (image_height - 1) * 2;
        // Triangles for back
        num_triangles += 2;
        // Triangles for sides
        num_triangles += 2 * image_width + 2 * image_height;
        // Triangles to enclose pixels to sides
        num_triangles += 4 * (image_width - 1) + 4 * (image_height - 1);

        let mut triangles = Vec::with_capacity(num_triangles as usize);

        let get_height = |gray_value: u8| -> f32 {
            self.white_height + (255 - gray_value) as f32 / 255f32 * (self.black_height - self.white_height)
        };

        // Generate triangles for image, from top left to bottom right
        // The lithophane will be in the +,+,+ octant with its back on the YZ plane, so the X axis of the image corresponds to the Y axis of the STL,
        // the Y axis of the image corresponds to the Z axis of the STL, and the X axis of the STL is used for luma.
        for z in 0..image_height - 1 {
            // Since image is top to bottom, we calculate a vertical offset to keep the generated triangles above the XY plane
            // The last row of pixels should be located on the XY plane
            let z_offset = (image_height - 1 - z) as f32 * self.edge_length;

            for y in 0..image_width - 1 {
                let y_offset = y as f32 * self.edge_length;

                // Two triangles to add, triangle for current, lower, and lower right pixels, and triangle for current, lower right, and right pixels
                let px_current_height = get_height(self.image.get_pixel(y, z).0[0]);
                let px_lower_height = get_height(self.image.get_pixel(y, z + 1).0[0]);
                let px_lower_right_height = get_height(self.image.get_pixel(y + 1, z + 1).0[0]);
                let px_right_height = get_height(self.image.get_pixel(y + 1, z).0[0]);

                triangles.extend_from_slice(&four_points_to_triangles([
                    [px_lower_height, y_offset, z_offset - self.edge_length].into(),
                    [px_lower_right_height, y_offset + self.edge_length, z_offset - self.edge_length].into(),
                    [px_right_height, y_offset + self.edge_length, z_offset].into(),
                    [px_current_height, y_offset, z_offset].into(),
                ]));
            }
        }

        let back_width = (image_width - 1) as f32 * self.edge_length;
        let back_height = (image_height - 1) as f32 * self.edge_length;

        // Triangles for back
        triangles.extend_from_slice(&four_points_to_triangles([
            [0f32, 0f32, 0f32].into(),
            [0f32, 0f32, back_height].into(),
            [0f32, back_width, back_height].into(),
            [0f32, back_width, 0f32].into(),
        ]));

        // Triangles for bottom
        triangles.push(three_points_to_triangle([
            [0f32, 0f32, 0f32].into(),
            [0f32, back_width, 0f32].into(),
            [self.white_height, back_width, 0f32].into(),
        ]));
        for y in 0..image_width - 1 {
            let y_offset = y as f32 * self.edge_length;
            let y_next_offset = (y + 1) as f32 * self.edge_length;
            triangles.push(three_points_to_triangle([
                [0f32, 0f32, 0f32].into(),
                [self.white_height, y_next_offset, 0f32].into(),
                [self.white_height, y_offset, 0f32].into(),
            ]));
        }

        // Triangles for top
        triangles.push(three_points_to_triangle([
            [0f32, 0f32, back_height].into(),
            [self.white_height, back_width, back_height].into(),
            [0f32, back_width, back_height].into(),
        ]));
        for y in 0..image_width - 1 {
            let y_offset = y as f32 * self.edge_length;
            let y_next_offset = (y + 1) as f32 * self.edge_length;
            triangles.push(three_points_to_triangle([
                [0f32, 0f32, back_height].into(),
                [self.white_height, y_offset, back_height].into(),
                [self.white_height, y_next_offset, back_height].into(),
            ]));
        }

        // Triangles for left side
        triangles.push(three_points_to_triangle([
            [0f32, 0f32, 0f32].into(),
            [self.white_height, 0f32, back_height].into(),
            [0f32, 0f32, back_height].into(),
        ]));
        for z in 0..image_height - 1 {
            let z_offset = (image_height - 1 - z) as f32 * self.edge_length;
            let z_next_offset = (image_height - 2 - z) as f32 * self.edge_length;
            triangles.push(three_points_to_triangle([
                [0f32, 0f32, 0f32].into(),
                [self.white_height, 0f32, z_next_offset].into(),
                [self.white_height, 0f32, z_offset].into(),
            ]));
        }

        // Triangles for right side
        triangles.push(three_points_to_triangle([
            [0f32, back_width, 0f32].into(),
            [0f32, back_width, back_height].into(),
            [self.white_height, back_width, back_height].into(),
        ]));
        for z in 0..image_height - 1 {
            let z_offset = (image_height - 1 - z) as f32 * self.edge_length;
            let z_next_offset = (image_height - 2 - z) as f32 * self.edge_length;
            triangles.push(three_points_to_triangle([
                [0f32, back_width, 0f32].into(),
                [self.white_height, back_width, z_offset].into(),
                [self.white_height, back_width, z_next_offset].into(),
            ]));
        }

        // Triangles to connect image to the bottom
        for y in 0..image_width - 1 {
            let y_offset = y as f32 * self.edge_length;

            let px_current_height = get_height(self.image.get_pixel(y, image_height - 1).0[0]);
            let px_right_height = get_height(self.image.get_pixel(y + 1, image_height - 1).0[0]);

            if px_current_height != self.white_height {
                triangles.push(three_points_to_triangle([
                    [self.white_height, y_offset, 0f32].into(),
                    [self.white_height, y_offset + self.edge_length, 0f32].into(),
                    [px_current_height, y_offset, 0f32].into(),
                ]));
            }

            if px_right_height != self.white_height {
                triangles.push(three_points_to_triangle([
                    [self.white_height, y_offset + self.edge_length, 0f32].into(),
                    [px_right_height, y_offset + self.edge_length, 0f32].into(),
                    [px_current_height, y_offset, 0f32].into(),
                ]));
            }
        }

        // Triangles to connect image to the top
        for y in 0..image_width - 1 {
            let y_offset = y as f32 * self.edge_length;

            let px_current_height = get_height(self.image.get_pixel(y, 0).0[0]);
            let px_right_height = get_height(self.image.get_pixel(y + 1, 0).0[0]);

            if px_current_height != self.white_height {
                triangles.push(three_points_to_triangle([
                    [self.white_height, y_offset, back_height].into(),
                    [px_current_height, y_offset, back_height].into(),
                    [self.white_height, y_offset + self.edge_length, back_height].into(),
                ]));
            }

            if px_right_height != self.white_height {
                triangles.push(three_points_to_triangle([
                    [px_current_height, y_offset, back_height].into(),
                    [px_right_height, y_offset + self.edge_length, back_height].into(),
                    [self.white_height, y_offset + self.edge_length, back_height].into(),
                ]));
            }
        }

        // Triangles to connect image to the left side
        for z in 0..image_height - 1 {
            let z_offset = (image_height - 1 - z) as f32 * self.edge_length;

            let px_current_height = get_height(self.image.get_pixel(0, z).0[0]);
            let px_lower_height = get_height(self.image.get_pixel(0, z + 1).0[0]);

            if px_current_height != self.white_height {
                triangles.push(three_points_to_triangle([
                    [self.white_height, 0f32, z_offset].into(),
                    [self.white_height, 0f32, z_offset - self.edge_length].into(),
                    [px_current_height, 0f32, z_offset].into(),
                ]));
            }

            if px_lower_height != self.white_height {
                triangles.push(three_points_to_triangle([
                    [self.white_height, 0f32, z_offset - self.edge_length].into(),
                    [px_lower_height, 0f32, z_offset - self.edge_length].into(),
                    [px_current_height, 0f32, z_offset].into(),
                ]));
            }
        }

        // Triangles to connect image to the right side
        for z in 0..image_height - 1 {
            let z_offset = (image_height - 1 - z) as f32 * self.edge_length;

            let px_current_height = get_height(self.image.get_pixel(image_width - 1, z).0[0]);
            let px_lower_height = get_height(self.image.get_pixel(image_width - 1, z + 1).0[0]);

            if px_current_height != self.white_height {
                triangles.push(three_points_to_triangle([
                    [self.white_height, back_width, z_offset - self.edge_length].into(),
                    [self.white_height, back_width, z_offset].into(),
                    [px_current_height, back_width, z_offset].into(),
                ]));
            }

            if px_lower_height != self.white_height {
                triangles.push(three_points_to_triangle([
                    [self.white_height, back_width, z_offset - self.edge_length].into(),
                    [px_current_height, back_width, z_offset].into(),
                    [px_lower_height, back_width, z_offset - self.edge_length].into(),
                ]));
            }
        }

        StlModel {
            header: String::new(),
            triangles,
        }
    }
}
