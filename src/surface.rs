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
        compute_plane_hit(&self.bias, &self.normal, ray)
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
        (Vec3 {
            x: 2.0 * (point.x - self.center.x) / (self.scale.a * self.scale.a),
            y: 2.0 * (point.y - self.center.y) / (self.scale.b * self.scale.b),
            z: 2.0 * (point.z - self.center.z) / (self.scale.c * self.scale.c),
        }).normalize()
    }

    fn get_color(&self) -> Color { self.color.clone() }
    fn get_specular_strength(&self) -> f32 { self.specular_strength }
}


#[derive(Debug, Clone)]
pub struct Cone {
    pub apex: Point,
    pub height: f32,
    pub half_angle: f32,
    pub color: Color,
    pub specular_strength: f32,
}


impl Cone {
    fn compute_cone_hit(&self, ray: &Ray) -> Option<f32> {
        let top_to_bot_vector = Vec3 {x: 0.0, y: -1.0, z: 0.0 };
        let cos2_alpha = self.half_angle.cos().powi(2);
        let apex_to_orig = (&ray.origin - &self.apex).normalize();
        let roots = find_square_roots(
            ray.direction.normalize().dot_product(&top_to_bot_vector).powi(2) - cos2_alpha,
            2.0 * ((ray.direction.normalize().dot_product(&top_to_bot_vector) * apex_to_orig.dot_product(&top_to_bot_vector)) - ray.direction.normalize().dot_product(&apex_to_orig) * cos2_alpha),
            apex_to_orig.dot_product(&top_to_bot_vector).powi(2) - apex_to_orig.norm_squared() * cos2_alpha,
        )?;

        if let Some(t) = select_smallest_positive_root(roots) {
            let hit_point = ray.compute_point(t);
            let bottom_center = Point {x: self.apex.x, y: self.apex.y - self.height, z: self.apex.z};

            if (&hit_point - &self.apex).dot_product(&top_to_bot_vector) >= 0.0 {
                if hit_point.y >= bottom_center.y {
                    Some(t)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn compute_slab_hit(&self, ray: &Ray) -> Option<f32> {
        let bottom_center = Point {x: self.apex.x, y: self.apex.y - self.height, z: self.apex.z};
        let slab_normal = Vec3 {x: 0.0, y: -1.0, z: 0.0};
        let radius = self.height * self.half_angle.tanh();
        let plane_hit = compute_plane_hit(
            &bottom_center, &slab_normal, ray)?;

        let hit_point = ray.compute_point(plane_hit);

        if (&hit_point - &bottom_center).norm_squared() < radius.powi(2) {
            Some(plane_hit)
        } else {
            None
        }
    }

    fn lies_on_slab(&self, point: &Point) -> bool {
        let bottom_center = Point {x: self.apex.x, y: self.apex.y - self.height, z: self.apex.z};
        let radius = self.height * self.half_angle.tanh();
        let slab_normal = Vec3 {x: 0.0, y: -1.0, z: 0.0};
        let lies_on_slab_plane = slab_normal.dot_product(&(point - &bottom_center)) == 0.0;
        let is_close = (point - &bottom_center).norm_squared() < radius.powi(2);

        lies_on_slab_plane && is_close
    }
}


impl Surface for Cone {
    fn compute_hit(&self, ray: &Ray) -> Option<f32> {
        let cone_hit = self.compute_cone_hit(ray);
        let slab_hit = self.compute_slab_hit(ray);

        if slab_hit.is_some() {
            if cone_hit.is_some() {
                Some(slab_hit.unwrap().min(cone_hit.unwrap()))
            } else {
                slab_hit
            }
        } else {
            cone_hit
        }
    }

    fn compute_normal(&self, point: &Point) -> Vec3 {
        if self.lies_on_slab(point) {
            Vec3 {x: 0.0, y: -1.0, z: 0.0}
        } else {
            let cos2_alpha = self.half_angle.cos().powi(2);

            (Vec3 {
                x: 2.0 * point.x,
                y: -2.0 * point.y * cos2_alpha,
                z: 2.0 * point.z,
            }).normalize()
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

#[inline]
fn compute_plane_hit(bias: &Point, normal: &Vec3, ray: &Ray) -> Option<f32> {
    let denom = normal.dot_product(&ray.direction);

    if denom == 0.0 {
        return None;
    }

    let num = (bias - &ray.origin).dot_product(&normal);
    let t = num / denom;

    if t >= MIN_RAY_T {
        Some(t)
    } else {
        None
    }
}
