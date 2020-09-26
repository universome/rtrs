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
        let orig_to_c = &self.center - &ray.origin;
        let roots = find_square_roots(
            ray.direction.norm_squared(),
            -2.0 * ray.direction.dot_product(&orig_to_c),
            orig_to_c.norm_squared() - self.radius * self.radius,
        )?;

        select_smallest_positive_root(roots)
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


#[derive(Debug, Clone)]
pub struct Ellipsoid {
    pub center: Point,
    pub color: Color,
    pub specular_strength: f32,
    pub scale: DiagMat3,
}

impl Surface for Ellipsoid {
    fn compute_hit(&self, ray: &Ray) -> Option<f32> {
        let scale_inv = self.scale.compute_inverse();
        let orig_to_c_scaled = &scale_inv * &(&self.center - &ray.origin);
        let ray_dir_scaled = &scale_inv * &ray.direction;
        let roots = find_square_roots(
            ray_dir_scaled.norm_squared(),
            -2.0 * ray_dir_scaled.dot_product(&orig_to_c_scaled),
            orig_to_c_scaled.norm_squared() - 1.0,
        )?;

        select_smallest_positive_root(roots)
    }

    fn compute_normal(&self, point: &Point) -> Vec3 {
        // &(point - &self.center) * (1. / self.radius)
        Vec3 {
            x: 2.0 * point.x / (self.scale.a * self.scale.a),
            y: 2.0 * point.y / (self.scale.b * self.scale.b),
            z: 2.0 * point.z / (self.scale.c * self.scale.c),
        }
    }

    fn get_color(&self) -> Color { self.color.clone() }
    fn get_specular_strength(&self) -> f32 { self.specular_strength }
}


#[inline]
fn find_square_roots(a: f32, b: f32, c: f32) -> Option<(f32, Option<f32>)> {
    // Finds roots of a quadratic equation
    let discr = b * b - 4.0 * a * c;

    if discr < 0.0 {
        return None;
    }

    if discr == 0.0 {
        Some((-b / (2.0 * a), None))
    } else {
        let discr_sqrt = discr.sqrt();

        Some((
            (-b - discr_sqrt) / (2.0 * a),
            Some((-b + discr_sqrt) / (2.0 * a)),
        ))
    }
}

#[inline]
fn select_smallest_positive_root(roots: (f32, Option<f32>)) -> Option<f32> {
    if roots.1.is_none() && roots.0 >= MIN_RAY_T {
        return Some(roots.0);
    }

    let (t0, t1) = (roots.0, roots.1.unwrap());

    if t0 < MIN_RAY_T {
        if t1 < MIN_RAY_T {
            None
        } else {
            Some(t1)
        }
    } else {
        if t1 < MIN_RAY_T {
            Some(t0)
        } else {
            Some(t0.min(t1))
        }
    }
}
