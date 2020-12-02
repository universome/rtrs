use rand::Rng;
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;

use crate::camera::{Camera};
use crate::surface::surface::{Surface, Hit, VisualData};
use crate::basics::*;


static NUM_DIST_RT_SAMPLES: i32 = 5;
static NUM_GLOSSY_REFL_RAYS: i32 = 10;


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
        let ray = self.camera.generate_ray(i as f32, j as f32);
        let mut closest_obj_idx = None;
        let mut min_t = f32::INFINITY;

        for (idx, object) in self.objects.iter().enumerate() {
            if let Some(hit) = object.compute_hit(&ray, false) {
                if hit.t < min_t {
                    closest_obj_idx = Some(idx);
                    min_t = hit.t;
                }
            }
        }

        closest_obj_idx
    }

    pub fn compute_ray_color(&self, ray_camera: &Ray, light_shift: Option<(f32, f32)>, rng: &mut ThreadRng, depth: u32, _debug: bool) -> Color {
        let mut hit = Hit::inf();
        let mut vis = VisualData::zero();

        for object in self.objects.iter() {
            if let Some(another_hit) = object.compute_hit(ray_camera, _debug) {
                if another_hit.t < hit.t {
                    hit = another_hit;
                    vis = object.get_visual_data();
                }
            }
        }

        if hit.t == f32::INFINITY {
            return self.background_color.clone();
        }

        let mut color = &vis.color * self.ambient_strength;
        // let mut color = Color {r: 0.5, g: 0.5, b: 0.5};
        let hit_point_camera = ray_camera.compute_point(hit.t); // TODO: do not recompute the hit point
        // println!("hit_point_camera: {:?}", hit_point_camera);

        for light_camera in self.lights.iter() {
            let light_location = if light_shift.is_some() {
                let (shift_right, shift_top) = light_shift.unwrap();

                &light_camera.location + &(&light_camera.right * shift_right + &light_camera.top * shift_top)
            } else {
                light_camera.location.clone()
            };

            let distance_to_light = (&light_location - &hit_point_camera).norm();
            let light_dir = (&light_location - &hit_point_camera).normalize();
            let shadow_ray = Ray {
                origin: &hit_point_camera + &(&light_dir.clone() * 0.0001),
                direction: light_dir.clone(),
            };

            // Diffuse component
            let is_in_shadow = self.objects.iter()
                // .filter(|o| !ptr::eq(*o, &*obj)) TODO: why did we need this?
                .any(|o| o.compute_hit(&shadow_ray, _debug)
                .filter(|hit| hit.t < distance_to_light).is_some());

            if !is_in_shadow {
                let diffuse_cos = hit.normal.dot_product(&light_dir.normalize()).max(0.0);
                let diffuse_light_color = &light_camera.color * (diffuse_cos * self.diffuse_strength);
                color = &color + &diffuse_light_color;
            }

            // Specular light component
            if vis.specular_strength > 0.0 {
                let eye_dir = (&self.camera.origin - &hit_point_camera).normalize();
                let half_vector = (eye_dir + light_dir).normalize();
                let spec_strength = vis.specular_strength * hit.normal.dot_product(&half_vector).max(0.0).powf(64.0);
                let spec_color = &light_camera.color * spec_strength;

                color = &color + &spec_color;
            }

            // Reflection component
            if depth == 0 && vis.reflection_strength > 0.0 {
                let ray_dir_normalized = ray_camera.direction.normalize();
                let reflection_dir = &ray_camera.direction + &hit.normal * (-2.0 * ray_dir_normalized.dot_product(&hit.normal));
                let reflection_rays;

                if vis.reflection_glossiness > 0.0 {
                    // Selecting the first orthogonal vector is a bit tricky
                    // Since we need to make sure that it is not equal to zero
                    // We just try different options: (0, -z, y), (-z, 0, x), (-y, x, 0)
                    let mut u = Vec3::new(0.0, -reflection_dir.z, reflection_dir.y);
                    if u.norm_squared() == 0.0 {
                        u = Vec3::new(-reflection_dir.z, 0.0, reflection_dir.x);
                    }
                    if u.norm_squared() == 0.0 {
                        u = Vec3::new(-reflection_dir.y, reflection_dir.x, 0.0);
                    }
                    u = u.normalize();

                    // Selecting the second orthogonal vector is trivial
                    let v = reflection_dir.cross_product(&u).normalize();

                    // Now, we can generate the rays
                    reflection_rays = (0..NUM_GLOSSY_REFL_RAYS)
                        .map(|_| -> Ray {
                            let u_weight = vis.reflection_glossiness * (rng.gen::<f32>() - 0.5);
                            let v_weight = vis.reflection_glossiness * (rng.gen::<f32>() - 0.5);

                            Ray {
                                origin: &hit_point_camera + &(&reflection_dir.clone() * 0.0001),
                                direction: &reflection_dir + &u * u_weight + &v * v_weight,
                            }
                        }).collect::<Vec<Ray>>();
                } else {
                    reflection_rays = vec![Ray {
                        origin: &hit_point_camera + &(&reflection_dir.clone() * 0.0001),
                        direction: reflection_dir
                    }];
                }

                let reflection_color = reflection_rays.iter().map(
                    |r| &self.compute_ray_color(r, None, rng, depth + 1, false) * (1.0 / reflection_rays.len() as f32))
                    .fold(Color::zero(), |c1, c2| &c1 + &c2);

                color = &color + &(&reflection_color * vis.reflection_strength);
            }

            color = (&color).clamp();
        }

        color
    }

    pub fn compute_pixel(&self, i: u32, j: u32, _debug: bool) -> Color {
        // let shifts = (0..25).map(|_| rng.gen::<f32>()).collect::<Vec<f32>>();
        let rays;
        let mut rng = rand::thread_rng();

        if false {
            rays = iproduct!(0..NUM_DIST_RT_SAMPLES, 0..NUM_DIST_RT_SAMPLES)
                .map(|p: (i32, i32)| self.camera.generate_ray(
                    (i as f32) + (p.0 as f32) / NUM_DIST_RT_SAMPLES as f32 + rng.gen::<f32>(),
                    (j as f32) + (p.1 as f32) / NUM_DIST_RT_SAMPLES as f32 + rng.gen::<f32>()
                ))
                .collect::<Vec<Ray>>();
        } else {
            rays = vec![self.camera.generate_ray(i as f32 + 0.5, j as f32 + 0.5)]
        }

        let mut light_shifts;

        if false {
            light_shifts = iproduct!(0..NUM_DIST_RT_SAMPLES, 0..NUM_DIST_RT_SAMPLES)
                .map(|p: (i32, i32)| Some((
                    (p.0 as f32) / NUM_DIST_RT_SAMPLES as f32 + rng.gen::<f32>(),
                    (p.1 as f32) / NUM_DIST_RT_SAMPLES as f32 + rng.gen::<f32>()
                )))
                .collect::<Vec<Option<(f32, f32)>>>();
            light_shifts.shuffle(&mut rng);
        } else {
            light_shifts = vec![None; 25];
        };

        rays
            .iter()
            .enumerate()
            .map(|(i, ray)| &self.compute_ray_color(ray, light_shifts[i], &mut rng, 0, _debug) * (1.0 / rays.len() as f32))
            .fold(Color::zero(), |c1, c2| &c1 + &c2)
    }
}


#[cfg(test)]
mod scene_tests {
    use super::*;
    use crate::surface::quadrics::Sphere;

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
        assert_eq!(sphere.compute_hit(&ray_a, false).unwrap().t, 4.0);
        assert!(approx_eq!(f32, sphere.compute_hit(&ray_b, false).unwrap().t, 1.0, epsilon = 0.001));
    }
}
