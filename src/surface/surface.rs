use std::marker::Sync;
use std::fmt::Debug;
// use std::cmp::Ordering;

use crate::basics::*;
use crate::matrix::{Mat3, AffineMat3};


#[derive(Debug, Clone)]
pub struct Hit {
    pub t: f32,
    pub normal: Vec3
}


impl Hit {
    pub fn inf() -> Hit {
        Hit {
            t: f32::INFINITY,
            normal: Vec3 {x: 0.0, y: 1.0, z: 0.0}
        }
    }
}


// macro_rules! impl_cmp_for_hit {
//     ($type_lhs:ty, $type_rhs:ty) => {
//         impl Ord for $type_lhs {
//             fn eq(&self, other: $type_rhs) -> Ordering {
//                 self.t.cmp(other.t)
//             }
//         }
//     };
// }
// impl_cmp_for_hit!(Hit, Hit);


pub trait Surface: Debug + Sync {
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<Hit>;
    // fn compute_normal(&self, point: &Point) -> Vec3;
    fn get_color(&self) -> Color;
    fn get_specular_strength(&self) -> f32;
}


#[derive(Debug, Clone)]
pub struct TransformedSurface<S> where S: Surface {
    transformation: AffineMat3,
    transformation_inv: AffineMat3,
    transform_inv_t: Mat3,
    surface: S,
}


impl<S: Surface> TransformedSurface<S> {
    pub fn new(transformation: AffineMat3, surface: S) -> TransformedSurface<S> {
        let transformation_inv = transformation.compute_inverse();

        TransformedSurface {
            transformation: transformation,
            transform_inv_t: transformation_inv.transform_mat.transpose(),
            transformation_inv: transformation_inv,
            surface: surface,
        }
    }

    fn transform_normal(&self, normal: &Vec3) -> Vec3 {
        // TODO: just return it in compute_hit
        // let point_object = &self.transformation_inv * point;
        // let normal = self.surface.transform_normal(&point_object);

        (&self.transform_inv_t * normal).normalize()
    }
}


impl<S: Surface> Surface for TransformedSurface<S> {
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<Hit> {
        let ray_object = Ray {
            origin: &self.transformation_inv * &ray.origin,
            direction: (&self.transformation_inv * &ray.direction).normalize(),
        };

        if let Some(hit) = self.surface.compute_hit(&ray_object, debug) {
            let hit_point = &self.transformation * &ray_object.compute_point(hit.t);
            let t_world = ray.compute_t(&hit_point);
            // let normal = self.compute_normal(hit_point);

            return Some(Hit {
                t: t_world,
                normal: self.transform_normal(&hit.normal)
            });
        }

        None
    }

    fn get_color(&self) -> Color { self.surface.get_color() }
    fn get_specular_strength(&self) -> f32 { self.surface.get_specular_strength() }
}
