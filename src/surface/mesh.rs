use std::sync::Arc;

use tobj::Model;

use crate::surface::surface::{Surface, Hit};
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
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<Hit> {
        let v0 = &self.positions[self.vertices.0];
        let v1 = &self.positions[self.vertices.1];
        let v2 = &self.positions[self.vertices.2];
        let edge_01 = v1 - v0;
        let edge_02 = v2 - v0;
        let face_normal = &edge_01.cross_product(&edge_02).normalize();
        let t_denom = face_normal.dot_product(&ray.direction);

        if t_denom.abs() < 0.000001 {
            // The ray and the triangle are parallel
            // println!("T denom: {:?}", t_denom);
            return None;
        }

        let plane_bias = -face_normal.dot_product(&v0.into());
        let t = -(face_normal.dot_product(&(&ray.origin).into()) + plane_bias) / t_denom;

        if t < MIN_RAY_T {
            // The triangle is either behind or too close
            // println!("TOO SMALL {:?}", t);
            return None;
        }

        let hit_point = &ray.compute_point(t);

        if is_on_the_right(hit_point, v0, v1, face_normal) ||
           is_on_the_right(hit_point, v1, v2, face_normal) ||
           is_on_the_right(hit_point, v2, v0, face_normal) {
            // println!("Is on the right!");
            return None;
        }

        Some(Hit {
            t: t,
            normal: face_normal.clone()
        })
    }

    // fn compute_normal(&self, point: &Point) -> Vec3 {
    //     Vec3 {x: 0.0, y: -1.0, z: 0.0}
    // }

    fn get_color(&self) -> Color { Color {r: 0.3, g: 0.3, b: 0.3} }
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

        // println!("Num triangles: {}", num_triangles);

        // We are going to convert a flattened array of [x1, y1, z1, x2, y2, z2, ...]
        // into an array of points [(x1, y1, z1), (x2, y2, z2), ...]
        for i in 0..(model.mesh.positions.len() / 3) {
            positions.push(Point::new(
                model.mesh.positions[i * 3 + 0],
                model.mesh.positions[i * 3 + 1],
                model.mesh.positions[i * 3 + 2]
            ));
        }

        let positions_arc = Arc::new(positions);
        let mut triangles = vec![];

        for i in 0..num_triangles {
            triangles.push(Triangle {
                vertices: (
                    model.mesh.indices[i * 3 + 0] as usize,
                    model.mesh.indices[i * 3 + 1] as usize,
                    model.mesh.indices[i * 3 + 2] as usize
                ),
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
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<Hit> {
        let mut closest_hit = Hit::inf();

        for triangle in self.triangles.iter() {
            if let Some(hit) = triangle.compute_hit(ray, debug) {
                closest_hit = if hit.t < closest_hit.t {hit} else {closest_hit};
            }
        }

        if closest_hit.t < f32::INFINITY {
            Some(closest_hit)
        } else {
            None
        }
    }

    // fn compute_normal(&self, point: &Point) -> Vec3 {
    //     Vec3 {x: 0.0, y: -1.0, z: 0.0}
    // }

    fn get_color(&self) -> Color { Color {r: 0.3, g: 0.3, b: 0.3} }
    fn get_specular_strength(&self) -> f32 { 0.5 }
}

#[inline]
fn is_on_the_right(hit_point: &Point, from: &Point, to: &Point, normal: &Vec3) -> bool {
    // Checks if the intersection point is on the left of the line
    // which goes from `from` to `to` points with the given `normal` normal
    let normal_for_intersection = (&(to - from)).cross_product(&(hit_point - from));

    normal.dot_product(&normal_for_intersection) < 0.0
}


#[cfg(test)]
mod mesh_tests {
    use super::*;

    #[test]
    fn test_ray_triangle_intersection() {
        let ray = Ray {
            origin: Point {x: 0.0, y: 0.0, z: 0.0},
            direction: Vec3 {x: 0.0, y: 0.0, z: 1.0},
        };
        let vertex_positions = Arc::new(vec![
            Point {x: 0.0, y: 0.0, z: 1.0},
            Point {x: 1.0, y: 0.0, z: 1.0},
            Point {x: 0.0, y: 1.0, z: 1.0},
        ]);
        let triangle_a = Triangle {
            vertices: (0, 1, 2),
            positions: vertex_positions.clone(),
        };
        // let triangle_b = Triangle {
        //     vertices: (0, 2, 1),
        //     positions: vertex_positions.clone(),
        // };
        let t_a = triangle_a.compute_hit(&ray, false).unwrap_or(f32::INFINITY);
        // let t_b = triangle_b.compute_hit(&ray, false);

        assert!(approx_eq!(f32, t_a, 1.0));
        // assert!(t_b.is_none());
    }

    #[test]
    fn test_ray_mesh_intersection() {
        // let mesh = TriangleMesh::from_obj("resources/square.obj");
        let mesh = TriangleMesh::from_obj("resources/cube.obj");
        let ray = Ray {
            origin: Point {x: 0.0, y: 0.0, z: -1.0},
            direction: Vec3 {x: 0.0, y: 0.0, z: 1.0},
        };

        let t = mesh.compute_hit(&ray, false).unwrap_or(f32::INFINITY);
        assert!(approx_eq!(f32, t, 1.0));
    }
}
