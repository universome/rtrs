use std::ops;
use std::fmt::Debug;
use derive_more::{Add, Sub};

#[derive(Debug, Clone, Add)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl ops::Mul<f32> for &Color {
    type Output = Color;

    fn mul(self, strength: f32) -> Color {
        Color {
            r: self.r * strength,
            g: self.g * strength,
            b: self.b * strength,
        }
    }
}

pub struct Scene<'a> {
    pub objects: Vec<&'a dyn Surface>,
    pub camera: Camera,
    pub viewing_plane: ViewingPlane,
    pub background_color: Color,
    pub lights: Vec<Light>,
    pub ambient_strength: f32,
    pub diffuse_strength: f32,
}

impl Scene<'_> {
    pub fn compute_pixel(&self, i: u32, j: u32) -> Color {
        let (u, v) = self.viewing_plane.generate_uv_coords(i, j);
        let ray = self.camera.generate_ray(u, v, &self.viewing_plane);
        let mut closest_obj = None;
        let mut min_t = f32::INFINITY;

        for object in self.objects.iter() {
            if let Some(t) = object.compute_hit(&ray) {
                if t < min_t {
                    closest_obj = Some(object);
                    min_t = t;
                }
            }
        }

        if closest_obj.is_some() {
            let obj = closest_obj.unwrap();
            let mut color = &obj.get_color() * self.ambient_strength;
            let ray_hit_point = ray.compute_point(min_t);
            let normal = obj.compute_normal(&ray_hit_point);

            for light in self.lights.iter() {
                let light_dir = (&light.location - &ray_hit_point).normalize();
                let strength = normal.dot_product(&light_dir).max(0.0);

                color = color + &light.color * strength;
            }

            color
        } else {
            self.background_color.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Light {
    pub location: Point,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct Camera {
    origin: Point,
    direction: Vec3,
    up: Vec3,
    right: Vec3,
}

impl Camera {
    pub fn from_z_position(z: f32) -> Camera {
        Camera {
            origin: Point {x: 0.0, y: 0.0, z: z},
            direction: Vec3 {x: 0.0, y: 0.0, z: -1.0},
            up: Vec3 {x: 0.0, y: 1.0, z: 0.0},
            right: Vec3 {x: 1.0, y: 0.0, z: 0.0},
        }
    }

    pub fn generate_ray(&self, u: f32, v: f32, viewing_plane: &ViewingPlane) -> Ray {
        let d = viewing_plane.z - self.origin.z;
        Ray {
            // TODO: actually, we do not need to clone anything here, right?
            origin: self.origin.clone(),
            direction: &self.direction * (-d) + &self.right * u + &self.up * v
        }
    }
}

#[derive(Debug, Clone)]
pub struct ViewingPlane {
    pub z: f32,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub width: u32,
    pub height: u32,
}

impl ViewingPlane {
    pub fn generate_uv_coords(&self, i: u32, j: u32) -> (f32, f32) {
        let x_dist = self.x_max - self.x_min;
        let y_dist = self.y_max - self.y_min;
        let u = self.x_min + x_dist * (i as f32 + 0.5) / (self.width as f32);
        let v = self.y_min + y_dist * (j as f32 + 0.5) / (self.height as f32);

        (u, v)
    }
}

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vec3
}

impl Ray {
    pub fn compute_point(&self, t: f32) -> Point {
        // Computes point on the ray given t value
        (&self.origin + &(&self.direction * t)).into()
    }
}

#[derive(Debug, Clone, Add, Sub, PartialEq)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    pub fn normalize(&self) -> Self {
        let norm = self.norm();

        Vec3 {x: self.x / norm, y: self.y / norm, z: self.z / norm }
    }

    pub fn norm(&self) -> f32 {
        self.norm_squared().sqrt()
    }

    pub fn norm_squared(&self) -> f32 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }

    pub fn dot_product(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross_product(&self, other: &Self) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}


impl ops::Mul<f32> for &Vec3 {
    type Output = Vec3;

    fn mul(self, scale: f32) -> Vec3 {
        Vec3 {x: self.x * scale, y: self.y * scale, z: self.z * scale}
    }
}


#[derive(Debug, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl ops::Sub<&Point> for &Point {
    type Output = Vec3;

    fn sub(self, other: &Point) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl ops::Add<&Vec3> for &Point {
    type Output = Vec3;

    fn add(self, direction: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x + direction.x,
            y: self.y + direction.y,
            z: self.z + direction.z,
        }
    }
}

impl From<Vec3> for Point {
    fn from(v: Vec3) -> Self {
        Point {x: v.x, y: v.y, z: v.z}
    }
}


pub trait Surface: Debug {
    fn compute_hit(&self, ray: &Ray) -> Option<f32>;
    fn compute_normal(&self, point: &Point) -> Vec3;
    fn get_color(&self) -> Color;
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Point,
    pub radius: f32,
    pub color: Color,
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

        let num_lhs = -ray.direction.dot_product(&orig_to_c);
        let denom = ray.direction.norm_squared();

        if discriminant == 0.0 {
            Some(num_lhs / denom)
        } else {
            Some((num_lhs - discriminant.sqrt()) / denom)
        }
    }

    fn compute_normal(&self, point: &Point) -> Vec3 {
        &(point - &self.center) * (1. / self.radius)
    }

    fn get_color(&self) -> Color {
        self.color.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere() {
        let sphere = Sphere {
            center: Point {x: 0.0, y: 0.0, z: 0.0},
            radius: 1.0,
            color: Color {r: 1.0, g: 0.0, b: 0.0},
        };
        let point_a = Point {x: 0.0, y: 1.0, z: 0.0};
        let point_b = Point {x: 0.0, y: 0.0, z: -1.0};

        assert_eq!(
            sphere.compute_normal(&point_a).normalize(),
            Vec3 { x: 0.0, y: 1.0, z: 0.0}
        );

        assert_eq!(
            sphere.compute_normal(&point_b).normalize(),
            Vec3 { x: 0.0, y: 0.0, z: -1.0}
        );

        let ray_a = Ray {
            origin: Point {x: 0.0, y: 0.0, z: -5.0},
            direction: Vec3 { x: 0.0, y: 0.0, z: 1.0 }
        };
        let ray_b = Ray {
            origin: Point {x: 0.0, y: 0.0, z: -(2.0_f32.sqrt())},
            direction: Vec3 { x: 0.0, y: 1.0 / 2.0_f32.sqrt(), z: 1.0 / 2.0_f32.sqrt() }
        };
        assert_eq!(sphere.compute_hit(&ray_a), Some(4.0));
        assert!(approx_eq!(f32, sphere.compute_hit(&ray_b).unwrap(), 1.0));
    }

    #[test]
    fn test_viewing_plane() {
        let vp = ViewingPlane {
            z: 0.0,
            x_min: -2.0,
            x_max: 2.0,
            y_min: -1.5,
            y_max: 1.5,
            width: 640,
            height: 480,
        };

        assert_eq!(vp.generate_uv_coords(0, 0), (-1.996875, -1.496875));
        assert_eq!(vp.generate_uv_coords(320, 240), (0.0031249523, 0.0031249523));
        assert_eq!(vp.generate_uv_coords(640, 480), (2.0031252, 1.503125));
    }
}
