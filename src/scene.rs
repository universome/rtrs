use std::ptr;

use crate::surface::*;
use crate::basics::*;
use crate::matrix::*;


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
            let point = ray.compute_point(min_t);
            let normal = obj.compute_normal(&point);

            for light in self.lights.iter() {
                let distance_to_light = (&light.location - &point).norm();
                let light_dir = (&light.location - &point).normalize();
                let shadow_ray = Ray {
                    origin: point.clone(),
                    direction: light_dir.clone(),
                };

                if self.objects.iter()
                    .filter(|o| !ptr::eq(*o, obj))
                    .any(|o| o.compute_hit(&shadow_ray).filter(|t| t <= &distance_to_light).is_some()) {
                        continue;
                }

                let diffuse_cos = normal.dot_product(&light_dir.normalize());
                let diffuse_light_color = &light.color * (diffuse_cos * self.diffuse_strength);

                // Specular light component
                let eye_dir = (&self.camera.origin - &point).normalize();
                let half_vector = (eye_dir + light_dir).normalize();
                let spec_strength = obj.get_specular_strength() * normal.dot_product(&half_vector).max(0.0).powf(64.0);
                let spec_color = (&Color {r: 1.0, g: 1.0, b: 1.0}) * spec_strength;

                color = (&(&color + &diffuse_light_color) + &spec_color).clamp();
            }

            color
        } else {
            self.background_color.clone()
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum ProjectionType {Parallel, Perspective}

#[derive(Debug, Clone)]
pub struct Camera {
    origin: Point,
    direction: Vec3,
    up: Vec3,
    right: Vec3,
    projection_type: ProjectionType
}

impl Camera {
    pub fn from_z_position(z: f32, projection_type: ProjectionType) -> Camera {
        Camera {
            origin: Point {x: 0.0, y: 0.0, z: z},
            direction: Vec3 {x: 0.0, y: 0.0, z: -1.0},
            up: Vec3 {x: 0.0, y: 1.0, z: 0.0},
            right: Vec3 {x: 1.0, y: 0.0, z: 0.0},
            projection_type: projection_type
        }
    }

    pub fn generate_ray(&self, u: f32, v: f32, viewing_plane: &ViewingPlane) -> Ray {
        let d = viewing_plane.z - self.origin.z;

        match self.projection_type {
            ProjectionType::Perspective => Ray {
                // TODO: actually, we do not need to clone anything here, right?
                origin: self.origin.clone(),
                direction: &self.direction * (-d) + &self.right * u + &self.up * v
            },
            ProjectionType::Parallel => Ray {
                origin: (&self.origin + &(&self.right * u) + &self.up * v).into(),
                direction: &self.direction * (-1.0),
            }
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere() {
        let sphere = Sphere {
            center: Point {x: 0.0, y: 0.0, z: 0.0},
            radius: 1.0,
            color: Color {r: 1.0, g: 0.0, b: 0.0},
            specular_strength: 0.0
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
            // direction: (&Vec3 { x: 0.0, y: 1.0, z: 1.0 }).normalize()
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
