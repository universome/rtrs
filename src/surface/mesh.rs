use std::sync::Arc;

use tobj::Model;

use crate::surface::surface::{Surface, Hit};
use crate::surface::quadrics::{Sphere};
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
    indices: (usize, usize, usize), // Vertex ids
    positions: Arc<Vec<Point>>,
    calculated_normals: Arc<Vec<Vec3>>,
    normals: Arc<Vec<Vec3>>
}


impl Triangle {
    pub fn compute_normal(&self) -> Vec3 {
        let v0 = &self.positions[self.indices.0];
        let v1 = &self.positions[self.indices.1];
        let v2 = &self.positions[self.indices.2];
        let edge_01 = v1 - v0;
        let edge_02 = v2 - v0;

        edge_01.cross_product(&edge_02).normalize()
    }

    pub fn compute_center(&self) -> Point {
        (&(Vec3::from(&self.positions[self.indices.0])
        + Vec3::from(&self.positions[self.indices.1])
        + Vec3::from(&self.positions[self.indices.0])) * (1.0 / 3.0)).into()
    }
}


impl Surface for Triangle {
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<Hit> {
        let v0 = &self.positions[self.indices.0];
        let v1 = &self.positions[self.indices.1];
        let v2 = &self.positions[self.indices.2];
        let face_normal = &self.compute_normal();
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

        let area = face_normal.norm() / 2.0;
        let area_v0 = (v1 - v0).cross_product(&(hit_point - v0)).norm() / 2.0;
        let area_v1 = (v2 - v1).cross_product(&(hit_point - v1)).norm() / 2.0;
        let area_v2 = (v0 - v2).cross_product(&(hit_point - v2)).norm() / 2.0;
        let bar_coords = (area_v0 / area, area_v1 / area, area_v2 / area);

        let normal = (if self.normals.len() > 0 {
            &self.normals[self.indices.0] * bar_coords.0 +
            &self.normals[self.indices.1] * bar_coords.1 +
            &self.normals[self.indices.2] * bar_coords.2
        } else {
            &self.calculated_normals[self.indices.0] * bar_coords.0 +
            &self.calculated_normals[self.indices.1] * bar_coords.1 +
            &self.calculated_normals[self.indices.2] * bar_coords.2
        }).normalize();

        Some(Hit {t, normal})
        // Some(Hit {t: t, normal: face_normal.clone()})
    }

    // fn compute_normal(&self, point: &Point) -> Vec3 {
    //     Vec3 {x: 0.0, y: -1.0, z: 0.0}
    // }

    fn get_color(&self) -> Color { Color {r: 0.3, g: 0.3, b: 0.3} }
    fn get_specular_strength(&self) -> f32 { 0.5 }
}

#[derive(Debug, Clone)]
// pub struct TriangleMesh {
pub struct TriangleMesh {
    triangles: Vec<Triangle>,
    positions: Arc<Vec<Point>>,
    calculated_normals: Arc<Vec<Vec3>>,
    normals: Arc<Vec<Vec3>>,
    bvh: Option<BoundingVolumeHierarchy>,
}


// impl TriangleMesh {
impl TriangleMesh {
    pub fn from_obj(obj_file: &str) -> Self {
        let (models, _) = tobj::load_obj(&obj_file, true).unwrap();
        let model = models[0].clone();
        let num_triangles = model.mesh.num_face_indices.len() as usize;
        let mut positions = vec![];
        let mut normals = vec![];

        // We are going to convert a flattened array of [x1, y1, z1, x2, y2, z2, ...]
        // into an array of points [(x1, y1, z1), (x2, y2, z2), ...]
        for i in 0..(model.mesh.positions.len() / 3) {
            positions.push(Point::new(
                model.mesh.positions[i * 3 + 0],
                model.mesh.positions[i * 3 + 1],
                model.mesh.positions[i * 3 + 2]
            ));

            if model.mesh.normals.len() > 0 {
                normals.push(Vec3::new(
                    model.mesh.normals[i * 3 + 0],
                    model.mesh.normals[i * 3 + 1],
                    model.mesh.normals[i * 3 + 2]
                ));
            }
        }

        let positions_arc = Arc::new(positions);
        let normals_arc = Arc::new(normals);
        let mut triangles = vec![];

        for i in 0..num_triangles {
            triangles.push(Triangle {
                indices: (
                    model.mesh.indices[i * 3 + 0] as usize,
                    model.mesh.indices[i * 3 + 1] as usize,
                    model.mesh.indices[i * 3 + 2] as usize
                ),
                positions: positions_arc.clone(),
                calculated_normals: Arc::new(vec![]),
                normals: normals_arc.clone(),
            });
        }

        // We are going fill it the following way
        // We iterate over each triangle and each triangle
        let mut all_normals = vec![vec![]; positions_arc.len()];
        for triangle_idx in 0..triangles.len() {
            let normal = triangles[triangle_idx].compute_normal();
            all_normals[triangles[triangle_idx].indices.0].push(normal.clone());
            all_normals[triangles[triangle_idx].indices.1].push(normal.clone());
            all_normals[triangles[triangle_idx].indices.2].push(normal.clone());
        }
        let calculated_normals = all_normals.iter().map(|normals| {
            normals.iter().fold(Vec3::zero(), |v1, v2| (&v1 + v2)).normalize()
        }).collect::<Vec<Vec3>>();
        let calculated_normals_arc = Arc::new(calculated_normals);

        for triangle_idx in 0..triangles.len() {
            triangles[triangle_idx].calculated_normals = calculated_normals_arc.clone();
        }

        TriangleMesh {
            bvh: Some(BoundingVolumeHierarchy::from_triangles_list(triangles.clone())),
            positions: positions_arc,
            calculated_normals: calculated_normals_arc,
            triangles: triangles,
            normals: normals_arc,
        }
    }

    fn compute_slow_hit(&self, ray: &Ray, debug: bool) -> Option<Hit> {
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
}


// impl Surface for TriangleMesh {
impl Surface for TriangleMesh {
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<Hit> {
        if self.bvh.is_some() {
            self.bvh.as_ref().unwrap().compute_hit(ray, debug)
        } else {
            self.compute_slow_hit(ray, debug)
        }
    }

    // fn compute_normal(&self, point: &Point) -> Vec3 {
    //     Vec3 {x: 0.0, y: -1.0, z: 0.0}
    // }

    fn get_color(&self) -> Color { Color {r: 0.3, g: 0.3, b: 0.3} }
    fn get_specular_strength(&self) -> f32 { 0.5 }
}

#[derive(Debug, Clone)]
struct BoundingVolumeHierarchy {
    triangle_left: Option<Triangle>,
    triangle_right: Option<Triangle>,
    bvh_left: Option<Box<BoundingVolumeHierarchy>>,
    bvh_right: Option<Box<BoundingVolumeHierarchy>>,
    bbox: Sphere,
}


impl BoundingVolumeHierarchy {
    pub fn from_triangles_list(triangles: Vec<Triangle>) -> Self {
        assert!(triangles.len() > 0);

        if triangles.len() == 1 {
            return BoundingVolumeHierarchy {
                bbox: BoundingVolumeHierarchy::compute_sphere_from_triangles(&triangles),
                triangle_left: Some(triangles[0].clone()),
                triangle_right: None,
                bvh_left: None,
                bvh_right: None,
            };
        }

        if triangles.len() == 2 {
            return BoundingVolumeHierarchy {
                bbox: BoundingVolumeHierarchy::compute_sphere_from_triangles(&triangles),
                triangle_left: Some(triangles[0].clone()),
                triangle_right: Some(triangles[1].clone()),
                bvh_left: None,
                bvh_right: None,
            };
        }

        let sphere = BoundingVolumeHierarchy::compute_sphere_from_triangles(&triangles);
        let mut triangles = triangles;
        triangles.sort_by(|t1, t2| t1.compute_center().x.partial_cmp(&t2.compute_center().x).unwrap());
        let (triangles_left, triangles_right) = triangles.split_at(triangles.len() / 2);

        let triangle_left = if triangles_left.len() == 1 {Some(triangles_left[0].clone())} else {None};
        let bvh_left = if triangles_left.len() == 1 {None} else {
            Some(Box::new(BoundingVolumeHierarchy::from_triangles_list(triangles_left.to_vec())))};
        let triangle_right = if triangles_right.len() == 1 {Some(triangles_right[0].clone())} else {None};
        let bvh_right = if triangles_right.len() == 1 {None} else {
            Some(Box::new(BoundingVolumeHierarchy::from_triangles_list(triangles_right.to_vec())))};

        BoundingVolumeHierarchy  {
            triangle_left: triangle_left,
            triangle_right: triangle_right,
            bvh_left: bvh_left,
            bvh_right: bvh_right,
            bbox: sphere,
        }
    }

    pub fn compute_sphere_from_triangles(triangles: &Vec<Triangle>) -> Sphere {
        let mut center = Point::zero();
        for i in 0..triangles.len() {
            let triangle_center = triangles[i].compute_center();
            center.x += triangle_center.x * (1.0 / triangles.len() as f32);
            center.y += triangle_center.y * (1.0 / triangles.len() as f32);
            center.z += triangle_center.z * (1.0 / triangles.len() as f32);
        }

        let max_distance = triangles.iter().map(|t| -> f32 {
            0.0_f32
                .max((&t.positions[t.indices.0] - &center).norm())
                .max((&t.positions[t.indices.1] - &center).norm())
                .max((&t.positions[t.indices.2] - &center).norm())
        }).fold(0.0, |x: f32, y: f32| x.max(y));

        Sphere::from_position(max_distance + 0.001, center)
    }
}


impl Surface for BoundingVolumeHierarchy {
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<Hit> {
        self.bbox.compute_hit(ray, debug)?;

        let left_hit = if self.triangle_left.is_some() {
            self.triangle_left.as_ref().unwrap().compute_hit(ray, debug)
        } else if self.bvh_left.is_some() {
            (*self.bvh_left.as_ref().unwrap()).compute_hit(ray, debug)
        } else {
            None
        };
        let right_hit = if self.triangle_right.is_some() {
            self.triangle_right.as_ref().unwrap().compute_hit(ray, debug)
        } else if self.bvh_right.is_some() {
            (*self.bvh_right.as_ref().unwrap()).compute_hit(ray, debug)
        } else {
            None
        };

        if left_hit.is_some() {
            if right_hit.is_some() {
                let left_hit = left_hit.unwrap();
                let right_hit = right_hit.unwrap();

                Some(if left_hit.t < right_hit.t { left_hit } else { right_hit })
            } else {
                left_hit
            }
        } else {
            right_hit
        }
    }
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
            indices: (0, 1, 2),
            positions: vertex_positions.clone(),
            normals: Arc::new(vec![]),
            calculated_normals: Arc::new(vec![Vec3::zero(), Vec3::zero(), Vec3::zero()]),
        };
        // let triangle_b = Triangle {
        //     indices: (0, 2, 1),
        //     positions: vertex_positions.clone(),
        // };
        // println!("{:?}", triangle_a.compute_hit(&ray, false));
        let t_a = triangle_a.compute_hit(&ray, false).unwrap().t;
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

        let t = mesh.compute_hit(&ray, false).unwrap().t;
        assert!(approx_eq!(f32, t, 1.0));
    }
}
