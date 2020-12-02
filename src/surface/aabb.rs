use std::mem;

use crate::surface::surface::{Surface, Hit, VisualData};
use crate::basics::*;
use crate::matrix::{Mat3, AffineMat3};
use crate::surface::MIN_RAY_T;


static EPSILON: f32 = 0.0000000001;


#[derive(Debug, Clone)]
pub struct AxisAlignedBox {
    pub min_corner: Point,
    pub max_corner: Point,
}

impl Surface for AxisAlignedBox {
    fn compute_hit(&self, ray: &Ray, _debug: bool) -> Option<Hit> {
        let mut t_min = (self.min_corner.x - ray.origin.x) / (ray.direction.x + EPSILON);
        let mut t_max = (self.max_corner.x - ray.origin.x) / (ray.direction.x + EPSILON);

        if t_min > t_max {
            mem::swap(&mut t_min, &mut t_max);
        };

        let mut t_y_min = (self.min_corner.y - ray.origin.y) / (ray.direction.y + EPSILON);
        let mut t_y_max = (self.max_corner.y - ray.origin.y) / (ray.direction.y + EPSILON);

        if t_y_min > t_y_max {
            mem::swap(&mut t_y_min, &mut t_y_max);
        }

        if t_min > t_y_max || t_y_min > t_max {
            return None;
        }

        if t_y_min > t_min {
            t_min = t_y_min;
        }

        if t_y_max < t_max {
            t_max = t_y_max;
        }

        let mut t_z_min = (self.min_corner.z - ray.origin.z) / (ray.direction.z + EPSILON);
        let mut t_z_max = (self.max_corner.z - ray.origin.z) / (ray.direction.z + EPSILON);

        if t_z_min > t_z_max {
            mem::swap(&mut t_z_min, &mut t_z_max);
        }

        if t_min > t_z_max || t_z_min > t_max {
            return None;
        }

        if t_z_min > t_min {
            t_min = t_z_min;
        }

        if t_z_max < t_max {
            t_max = t_z_max;
        }

        // Returning the value
        let t;
        if t_min < MIN_RAY_T {
            if t_max < MIN_RAY_T {
                return None;
            } else {
                t = t_max;
            }
        } else {
            t = t_min;
        }

        // Returning the dummy normal since we are not going to render it anyway
        Some(Hit {t: t, normal: Vec3 {x: 0.0, y: 1.0, z: 0.0}})
    }
    fn get_visual_data(&self) -> VisualData { VisualData::grey() }
}


#[cfg(test)]
mod box_tests {
    use super::*;

    #[test]
    fn test_ray_box_intersection() {
        let aab = AxisAlignedBox {
            min_corner: Point {x: 0.0, y: 0.0, z: 0.0},
            max_corner: Point {x: 1.0, y: 1.0, z: 1.0},
        };
        let ray = Ray {
            origin: Point {x: 0.0, y: 0.0, z: -1.0},
            direction: Vec3 {x: 0.0, y: 0.0, z: 1.0},
        };

        println!("Hit: {:?}", aab.compute_hit(&ray, false));
        let t = aab.compute_hit(&ray, false).unwrap().t;
        assert!(approx_eq!(f32, t, 1.0));
    }
}
