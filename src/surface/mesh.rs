use std::sync::Arc;

use tobj::Model;

use crate::surface::surface::{Surface};
use crate::basics::*;
use crate::matrix::{Mat3, AffineMat3};
use crate::surface::MIN_RAY_T;

// #[derive(Debug, Clone)]
// pub struct Vertex {
//     // position: Point,
//     // color: Color,
//     // id: usize,
// }


#[derive(Debug, Clone)]
pub struct Triangle {
    vertices: (usize, usize, usize), // Vertex ids
    positions: Arc<Vec<Point>>,
}


impl Surface for Triangle {
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<f32> {
        None
    }

    fn compute_normal(&self, point: &Point) -> Vec3 {
        Vec3 {x: 0.0, y: -1.0, z: 0.0}
    }

    fn get_color(&self) -> Color { Color::zero() }
    fn get_specular_strength(&self) -> f32 { 0.5 }
}

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    triangles: Vec<Triangle>,
    positions: Arc<Vec<Point>>,
    normals: Option<Vec<Vec3>>,
}


impl TriangleMesh {
    pub fn from_obj(obj_file: &str) -> TriangleMesh {
        let (models, _) = tobj::load_obj(&obj_file, true).unwrap();
        let model = models[0].clone();
        let num_triangles = model.mesh.num_face_indices.len() as usize;
        // let mut triangle_mesh = TriangleMesh {triangles: vec![], positions: vec![], normals: None};
        let mut positions = vec![];

        for i in 0..num_triangles {
            let idx_1 = model.mesh.indices[i * 3 + 0] as usize;
            let idx_2 = model.mesh.indices[i * 3 + 1] as usize;
            let idx_3 = model.mesh.indices[i * 3 + 2] as usize;

            let v0 = Point::new(model.mesh.positions[idx_1 * 3 + 0], model.mesh.positions[idx_1 * 3 + 1], model.mesh.positions[idx_1 * 3 + 2]);
            let v1 = Point::new(model.mesh.positions[idx_2 * 3 + 0], model.mesh.positions[idx_2 * 3 + 1], model.mesh.positions[idx_2 * 3 + 2]);
            let v2 = Point::new(model.mesh.positions[idx_3 * 3 + 0], model.mesh.positions[idx_3 * 3 + 1], model.mesh.positions[idx_3 * 3 + 2]);

            // let normal_v0 = Vec3::new(model.mesh.normals[idx_1 * 3 + 0], model.mesh.normals[idx_1 * 3 + 1], model.mesh.normals[idx_1 * 3 + 2]);
            // let normal_v1 = Vec3::new(model.mesh.normals[idx_2 * 3 + 0], model.mesh.normals[idx_2 * 3 + 1], model.mesh.normals[idx_2 * 3 + 2]);
            // let normal_v2 = Vec3::new(model.mesh.normals[idx_3 * 3 + 0], model.mesh.normals[idx_3 * 3 + 1], model.mesh.normals[idx_3 * 3 + 2]);

            positions.push(v0);
            positions.push(v1);
            positions.push(v2);
        }

        let positions_arc = Arc::new(positions);
        let mut triangles = vec![];

        for i in 0..num_triangles {
            triangles.push(Triangle {
                vertices: (i * 3 + 0, i * 3 + 1, i * 3 + 2),
                positions: positions_arc.clone(),
            });
        }

        TriangleMesh {
            positions: positions_arc,
            triangles: triangles,
            normals: None,
        }
    }
}


impl Surface for TriangleMesh {
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<f32> {
        None
    }

    fn compute_normal(&self, point: &Point) -> Vec3 {
        Vec3 {x: 0.0, y: -1.0, z: 0.0}
    }

    fn get_color(&self) -> Color { Color::zero() }
    fn get_specular_strength(&self) -> f32 { 0.5 }
}
