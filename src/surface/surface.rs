use std::marker::Sync;
use std::fmt::Debug;

use crate::basics::*;
use crate::matrix::{Mat3, AffineMat3};


pub trait Surface: Debug + Sync {
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<f32>;
    fn compute_normal(&self, point: &Point) -> Vec3;
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
}


impl<S: Surface> Surface for TransformedSurface<S> {
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<f32> {
        let ray_os = Ray {
            origin: &self.transformation_inv * &ray.origin,
            direction: (&self.transformation_inv * &ray.direction).normalize(),
        };

        if let Some(t) = self.surface.compute_hit(&ray_os, debug) {
            let hit_point = &self.transformation * &ray_os.compute_point(t);

            return Some(ray.compute_t(&hit_point));
        }

        None
    }

    fn compute_normal(&self, point: &Point) -> Vec3 {
        // TODO: just return it in compute_hit
        let point_os = &self.transformation_inv * point;
        let normal = self.surface.compute_normal(&point_os);

        (&self.transform_inv_t * &normal).normalize()
    }

    fn get_color(&self) -> Color { self.surface.get_color() }
    fn get_specular_strength(&self) -> f32 { self.surface.get_specular_strength() }
}
