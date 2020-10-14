use std::ptr;

use crate::camera::{Camera};
use crate::surface::{Surface};
use crate::basics::*;
use crate::matrix::{Mat3};


pub struct Scene {
    pub objects: Vec<Box<dyn Surface>>,
    pub camera: Camera,
    pub background_color: Color,
    pub lights: Vec<Light>,
    pub ambient_strength: f32,
    pub diffuse_strength: f32,
}


impl Scene {
    pub fn get_object_idx_at_pixel(&self, i: u32, j: u32) -> Option<usize> {
        let ray = self.camera.generate_ray(i, j);
        let mut closest_obj_idx = None;
        let mut min_t = f32::INFINITY;

        for (idx, object) in self.objects.iter().enumerate() {
            if let Some(t) = object.compute_hit(&ray, false) {
                if t < min_t {
                    closest_obj_idx = Some(idx);
                    min_t = t;
                }
            }
        }

        closest_obj_idx
    }

    pub fn compute_pixel(&self, i: u32, j: u32, debug: bool) -> Color {
        // let closest_obj = self.get_object_at_pixel(i, j);
        let ray_ws = self.camera.generate_ray(i, j);
        let mut closest_obj = None;
        let mut min_t_ws = f32::INFINITY;

        for object in self.objects.iter() {
            if let Some(t_ws) = object.compute_hit(&ray_ws, debug) {
                if t_ws < min_t_ws {
                    closest_obj = Some((object, t_ws));
                    min_t_ws = t_ws;
                }
            }
        }

        if closest_obj.is_none() {
            return self.background_color.clone();
        }

        let (obj, min_t_ws) = closest_obj.unwrap();
        let mut color = &obj.get_color() * self.ambient_strength;
        let hit_point_ws = ray_ws.compute_point(min_t_ws); // TODO: do not recompute the hit hit_point

        let normal_ws = obj.compute_normal(&hit_point_ws);

        for light_ws in self.lights.iter() {
            let distance_to_light = (&light_ws.location - &hit_point_ws).norm();
            let light_dir = (&light_ws.location - &hit_point_ws).normalize();
            let shadow_ray = Ray {
                origin: &hit_point_ws.clone() + &(&light_dir.clone() * 0.0001),
                direction: light_dir.clone(),
            };

            if self.objects.iter()
                // .filter(|o| !ptr::eq(*o, &*obj)) TODO: why did we need this?
                .any(|o| o.compute_hit(&shadow_ray, debug).filter(|t| t <= &distance_to_light).is_some()) {
                    continue;
            }

            let diffuse_cos = normal_ws.dot_product(&light_dir.normalize()).max(0.0);
            let diffuse_light_color = &light_ws.color * (diffuse_cos * self.diffuse_strength);

            // Specular light component
            let eye_dir = (&self.camera.origin - &hit_point_ws).normalize();
            let half_vector = (eye_dir + light_dir).normalize();
            let spec_strength = obj.get_specular_strength() * normal_ws.dot_product(&half_vector).max(0.0).powf(64.0);
            let spec_color = (&Color {r: 1.0, g: 1.0, b: 1.0}) * spec_strength;

            color = (&(&color + &diffuse_light_color) + &spec_color).clamp();
        }

        color
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::Sphere;

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
        assert_eq!(sphere.compute_hit(&ray_a, false), Some(4.0));
        assert!(approx_eq!(f32, sphere.compute_hit(&ray_b, false).unwrap(), 1.0));
    }
}
