use std::fmt::Debug;
use crate::basics::*;


static MIN_RAY_T: f32 = 0.0001;


pub trait Surface: Debug {
    fn compute_hit(&self, ray: &Ray) -> Option<f32>;
    fn compute_normal(&self, point: &Point) -> Vec3;
    fn get_color(&self) -> Color;
    fn get_specular_strength(&self) -> f32;
}


#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Point,
    pub radius: f32,
    pub color: Color,
    pub specular_strength: f32,
}

impl Surface for Sphere {
    fn compute_hit(&self, ray: &Ray) -> Option<f32> {
        // debug_assert!(is_unit_length(ray.direction));
        let orig_to_c = &ray.origin - &self.center;
        let d_lhs = ray.direction.dot_product(&orig_to_c).powi(2);
        let d_rhs = ray.direction.norm_squared() * (orig_to_c.norm_squared() - self.radius.powi(2));
        let discriminant = d_lhs - d_rhs;

        if discriminant < 0.0 {
            return None;
        }

        let t;
        let num_lhs = -ray.direction.dot_product(&orig_to_c);
        let denom = ray.direction.norm_squared();

        if discriminant == 0.0 {
            t = num_lhs / denom;
        } else {
            let discr_sqrt = discriminant.sqrt();
            t = match num_lhs < discr_sqrt {
                true => (num_lhs + discr_sqrt) / denom,
                false => (num_lhs - discr_sqrt) / denom,
            };
        }

        if t >= MIN_RAY_T { Some(t) } else { None }
    }

    fn compute_normal(&self, point: &Point) -> Vec3 {
        &(point - &self.center) * (1. / self.radius)
    }

    fn get_color(&self) -> Color {
        self.color.clone()
    }

    fn get_specular_strength(&self) -> f32 { self.specular_strength }
}


#[derive(Debug, Clone)]
pub struct Plane {
    pub bias: Point,
    pub normal: Vec3,
    pub color: Color,
}

impl Plane {
    pub fn from_y(y: f32, color: Color) -> Plane {
        // Creates a horizontal plane
        Plane {
            bias: Point {x: 0.0, z: 0.0, y: y},
            normal: Vec3 {x: 0.0, y: 1.0, z: 0.0},
            color: color
        }
    }
}

impl Surface for Plane {
    fn compute_hit(&self, ray: &Ray) -> Option<f32> {
        let denom = self.normal.dot_product(&ray.direction);

        if denom == 0.0 {
            return None;
        }

        let num = (&self.bias - &ray.origin).dot_product(&self.normal);
        let t = num / denom;

        if t >= MIN_RAY_T {
            Some(t)
        } else {
            None
        }
    }

    fn compute_normal(&self, _point: &Point) -> Vec3 {
        self.normal.clone()
    }

    fn get_color(&self) -> Color {
        self.color.clone()
    }

    fn get_specular_strength(&self) -> f32 { 0.0 }
}
