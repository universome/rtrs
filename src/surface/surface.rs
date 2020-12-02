use std::marker::Sync;
use std::fmt::Debug;
// use std::cmp::Ordering;

use crate::basics::*;
use crate::matrix::{Mat3, AffineMat3};


#[derive(Debug, Clone)]
pub struct Hit {
    pub t: f32,
    pub normal: Vec3,
}


impl Hit {
    pub fn new(t: f32, normal: Vec3) -> Self {
        Hit {t, normal}
    }

    pub fn inf() -> Self {
        Hit::new(f32::INFINITY, Vec3 {x: 0.0, y: 1.0, z: 0.0})
    }
}

#[derive(Debug, Clone)]
pub struct VisualData {
    pub color: Color,
    pub specular_strength: f32,
    pub reflection_strength: f32,
    pub reflection_glossiness: f32,
}


impl VisualData {
    pub fn from_color(color: &Color) -> Self {
        VisualData {
            color: color.clone(),
            specular_strength: 0.0,
            reflection_strength: 0.0,
            reflection_glossiness: 0.0,
        }
    }

    pub fn zero() -> Self {
        VisualData::from_color(&Color::zero())
    }

    pub fn grey() -> Self {
        VisualData::from_color(&Color {r: 0.74, g: 0.76, b: 0.78})
    }
}


pub trait Surface: Debug + Sync {
    fn compute_hit(&self, ray: &Ray, debug: bool) -> Option<Hit>;
    fn get_visual_data(&self) -> VisualData;
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

            return Some(Hit::new(t_world, self.transform_normal(&hit.normal)));
        }

        None
    }

    fn get_visual_data(&self) -> VisualData { self.surface.get_visual_data() }
}
