use std::cmp;

use crate::basics::*;
use crate::matrix::*;


// #[derive(Debug, Clone)]
// struct Triangle(Point, Point, Point);

// impl Triangle {
//     // pub fn compute_normal(&self) -> Vec3 {

//     // }

//     // pub fn check_if_intersects(&self) -> Vec3 {

//     // }

//     // pub fn project(&self, ) -> Triangle {

//     // }
//     pub fn get_x_bounds(&self) -> (f32, f32) {
//         (min_of_three(self.0.x, self.1.x, self.2.x), cmp::max(self.0.x, self.1.x, self.2.x))
//     }

//     pub fn get_y_bounds(&self) -> (f32, f32) {
//         (min_of_three(self.0.y, self.1.y, self.2.y), cmp::max(self.0.y, self.1.y, self.2.y))
//     }
// }

// #[inline]
// fn min_of_three<T: cmp::PartialOrd + cmp::Ord>(a: T, b: T, c: T) -> T {
//     cmp::min(cmp::min(a, b), c)
// }

// #[inline]
// fn max_of_three<T: cmp::PartialOrd + cmp::Ord>(a: T, b: T, c: T) -> T {
//     cmp::max(cmp::max(a, b), c)
// }
