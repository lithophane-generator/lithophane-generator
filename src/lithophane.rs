use pk_stl::{StlModel, geometry::{Vec3, Triangle}};

pub mod rectangle;

pub trait LithophaneGenerator {
    fn generate_lithophane(&self) -> StlModel;
}

// TODO abstract lithophane generator that takes an equation for a 3d surface with normals and outputs a lithophane based on that surface
// Probably won't bother to verify that there aren't any curves that are too tight which would result in self-intersection, if someone uses
// a surface that has too tight of a curve then they'll just get a bad model 🤷
// Use https://github.com/rekka/meval-rs for equation parsing?

/// Turns four points of a square into two triangles. The points should be ordered according to the right hand rule. The triangles will share the edge
/// from the second point to the fourth point.
fn three_points_to_triangle(points: [Vec3; 3]) -> Triangle {
    if points[0] == points[1] || points[0] == points[2] || points[1] == points[2] {
        panic!("duplicate point");
    }
    Triangle {
        normal: normalize_to_unit_vector(cross_product(points[1] - points[0], points[2] - points[0])),
        vertices: [
            points[0],
            points[1],
            points[2],
        ],
    }
}

/// Turns four points of a square into two triangles. The points should be ordered according to the right hand rule. The triangles will share the edge
/// from the second point to the fourth point.
fn four_points_to_triangles(points: [Vec3; 4]) -> [Triangle; 2] {
    if points[0] == points[1] || points[0] == points[2] || points[0] == points[3] || points[1] == points[2] || points[1] == points[3] || points[2] == points[3] {
        panic!("duplicate point");
    }
    [
        three_points_to_triangle([points[0], points[1], points[3]]),
        three_points_to_triangle([points[1], points[2], points[3]]),
    ]
}

fn cross_product(a: Vec3, b: Vec3) -> Vec3 {
    [
        a.y * b.z - b.y * a.z,
        a.z * b.x - b.z * a.x,
        a.x * b.y - b.x * a.y,
    ].into()
}

fn normalize_to_unit_vector(v: Vec3) -> Vec3 {
    let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    [
        v.x / length,
        v.y / length,
        v.z / length,
    ].into()
}
