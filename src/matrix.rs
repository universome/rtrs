use std::ops;
use crate::basics::{Vec3, Point};

#[derive(Debug, Clone)]
pub struct Mat3 {
    pub rows: [Vec3; 3],
}

impl Mat3 {
    fn det(&self) -> f32 {
        self[0][0] * (self[1][1] * self[2][2] - self[2][1] * self[1][2]) -
        self[0][1] * (self[1][0] * self[2][2] - self[1][2] * self[2][0]) +
        self[0][2] * (self[1][0] * self[2][1] - self[1][1] * self[2][0])
    }

    pub fn compute_inverse(&self) -> Mat3 {
        let invdet = 1.0 / self.det();

        Mat3 {rows: [
            Vec3 {
                x: (self[1][1] * self[2][2] - self[2][1] * self[1][2]) * invdet,
                y: (self[0][2] * self[2][1] - self[0][1] * self[2][2]) * invdet,
                z: (self[0][1] * self[1][2] - self[0][2] * self[1][1]) * invdet,
            },
            Vec3 {
                x: (self[1][2] * self[2][0] - self[1][0] * self[2][2]) * invdet,
                y: (self[0][0] * self[2][2] - self[0][2] * self[2][0]) * invdet,
                z: (self[1][0] * self[0][2] - self[0][0] * self[1][2]) * invdet,
            },
            Vec3 {
                x: (self[1][0] * self[2][1] - self[2][0] * self[1][1]) * invdet,
                y: (self[2][0] * self[0][1] - self[0][0] * self[2][1]) * invdet,
                z: (self[0][0] * self[1][1] - self[1][0] * self[0][1]) * invdet,
            },
        ]}
    }

    pub fn transpose(&self) -> Mat3 {
        Mat3 {rows: [
            Vec3 {x: self[0][0], y: self[1][0], z: self[2][0]},
            Vec3 {x: self[0][1], y: self[1][1], z: self[2][1]},
            Vec3 {x: self[0][2], y: self[1][2], z: self[2][2]},
        ]}
    }
}


impl ops::Index<usize> for Mat3 {
    type Output = Vec3;

    fn index(&self, idx: usize) -> &Vec3 {
        &self.rows[idx]
    }
}


impl ops::Mul<&Vec3> for &Mat3 {
    type Output = Vec3;

    fn mul(self, vec: &Vec3) -> Vec3 {
        Vec3 {
            x: self[0].dot_product(vec),
            y: self[1].dot_product(vec),
            z: self[2].dot_product(vec),
        }
    }
}


impl ops::Mul<&Point> for &Mat3 {
    type Output = Point;

    fn mul(self, point: &Point) -> Point {
        let vec: Vec3 = point.into();

        Point {
            x: self[0].dot_product(&vec),
            y: self[1].dot_product(&vec),
            z: self[2].dot_product(&vec),
        }
    }
}


#[derive(Debug, Clone)]
struct Transformation {
    transform: Mat3,
    translation: Vec3,
}

impl Transformation {
    fn compute_inverse(&self) -> Transformation {
        let transform_inv = self.transform.compute_inverse();
        let back_translation = &(&transform_inv * &self.translation) * -1.0;

        Transformation {
            transform: transform_inv,
            translation: back_translation,
        }
    }
}


// struct RotationMatrix {

// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_det() {
        let identity = Mat3 { rows: [
            Vec3 {x: 1.0, y: 0.0, z: 0.0},
            Vec3 {x: 0.0, y: 1.0, z: 0.0},
            Vec3 {x: 0.0, y: 0.0, z: 1.0},
        ]};
        let singular_mat = Mat3 { rows: [
            Vec3 {x: 1.0, y: 2.0, z: 0.0},
            Vec3 {x: 1.0, y: 2.0, z: 0.0},
            Vec3 {x: 1.0, y: 2.0, z: 1.0},
        ]};
        let random_mat = Mat3 { rows: [
            Vec3 {x: 1.3, y: 2.5, z: 3.0},
            Vec3 {x: 1.0, y: 1.0, z: 8.7},
            Vec3 {x: 1.5, y: 6.2, z: 2.5},
        ]};

        assert_eq!(identity.det(), 1.0);
        assert_eq!(singular_mat.det(), 0.0);
        assert!(approx_eq!(f32, random_mat.det(), -26.397));
    }

    #[test]
    fn test_mat_inverse() {
        let random_mat = Mat3 { rows: [
            Vec3 {x: 1.3, y: 2.5, z: 3.0},
            Vec3 {x: 1.0, y: 1.0, z: 8.7},
            Vec3 {x: 1.5, y: 6.2, z: 2.5},
        ]};
        let rando_mat_inv = random_mat.compute_inverse();

        assert!(approx_eq!(f32, rando_mat_inv[0][0], 1.94871, epsilon=0.0001));
        assert!(approx_eq!(f32, rando_mat_inv[0][1], -0.467856, epsilon=0.0001));
        assert!(approx_eq!(f32, rando_mat_inv[0][2], -0.710308, epsilon=0.0001));
        assert!(approx_eq!(f32, rando_mat_inv[1][0], -0.399667, epsilon=0.0001));
        assert!(approx_eq!(f32, rando_mat_inv[1][1], 0.0473539, epsilon=0.0001));
        assert!(approx_eq!(f32, rando_mat_inv[1][2], 0.314809, epsilon=0.0001));
        assert!(approx_eq!(f32, rando_mat_inv[2][0], -0.178051, epsilon=0.0001));
        assert!(approx_eq!(f32, rando_mat_inv[2][1], 0.163276, epsilon=0.0001));
        assert!(approx_eq!(f32, rando_mat_inv[2][2], 0.0454597, epsilon=0.0001));
    }
}
