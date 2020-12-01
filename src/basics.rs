use std::ops;
use nannou::image::{Rgb};
use derive_more;


#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        (&Color {r: r, g: g, b: b}).clamp()
    }

    pub fn zero() -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    pub fn clamp(&self) -> Color {
        Color {
            r: self.r.max(0.0).min(1.0),
            g: self.g.max(0.0).min(1.0),
            b: self.b.max(0.0).min(1.0),
        }
    }

    pub fn add_no_clamp(&self, other: &Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl ops::Mul<f32> for &Color {
    type Output = Color;

    fn mul(self, strength: f32) -> Color {
        (Color {
            r: self.r * strength,
            g: self.g * strength,
            b: self.b * strength,
        }).clamp()
    }
}

impl ops::Add<&Color> for &Color {
    type Output = Color;

    fn add(self, other: &Color) -> Color {
        self.add_no_clamp(other).clamp()
    }
}


impl From<Color> for Rgb<u8> {
    fn from(color: Color) -> Self {
        Rgb([
            (color.r * (u8::MAX - 1) as f32) as u8,
            (color.g * (u8::MAX - 1) as f32) as u8,
            (color.b * (u8::MAX - 1) as f32) as u8,
        ])
    }
}


#[derive(Debug, Clone, derive_more::Sub, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 {x: x, y: y, z: z}
    }

    pub fn zero() -> Self {
        Vec3::new(0.0, 0.0, 0.0)
    }

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


impl ops::Index<usize> for Vec3 {
    type Output = f32;

    fn index(&self, idx: usize) -> &f32 {
        match idx {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Value {} is out of bounds for Vec3", idx),
        }
    }
}

macro_rules! impl_add_vec3 {
    ($type_lhs:ty, $type_rhs:ty) => {
        impl ops::Add<$type_rhs> for $type_lhs {
            type Output = Vec3;

            fn add(self, other: $type_rhs) -> Vec3 {
                Vec3 {
                    x: self.x + other.x,
                    y: self.y + other.y,
                    z: self.z + other.z,
                }
            }
        }
    };
}
impl_add_vec3!(Vec3, Vec3);
impl_add_vec3!(Vec3, &Vec3);
impl_add_vec3!(&Vec3, Vec3);
impl_add_vec3!(&Vec3, &Vec3);


impl ops::Mul<f32> for &Vec3 {
    type Output = Vec3;

    fn mul(self, scale: f32) -> Vec3 {
        Vec3 {x: self.x * scale, y: self.y * scale, z: self.z * scale}
    }
}


impl ops::Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}


impl From<&Point> for Vec3 {
    fn from(p: &Point) -> Self {
        Vec3 {x: p.x, y: p.y, z: p.z}
    }
}


#[derive(Debug, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point {x: x, y: y, z: z}
    }

    pub fn zero() -> Self {
        Point {x: 0.0, y: 0.0, z: 0.0}
    }
}


impl ops::Mul<f32> for &Point {
    type Output = Point;

    fn mul(self, scalar: f32) -> Point {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}


impl ops::Add<f32> for &Point {
    type Output = Point;

    fn add(self, scalar: f32) -> Point {
        Point {
            x: self.x + scalar,
            y: self.y + scalar,
            z: self.z + scalar,
        }
    }
}

macro_rules! impl_sub_for_point {
    ($type_lhs:ty, $type_rhs:ty) => {
        impl ops::Sub<$type_rhs> for $type_lhs {
            type Output = Vec3;

            fn sub(self, other: $type_rhs) -> Vec3 {
                Vec3 {
                    x: self.x - other.x,
                    y: self.y - other.y,
                    z: self.z - other.z,
                }
            }
        }
    };
}

impl_sub_for_point!(Point, Point);
impl_sub_for_point!(Point, &Point);
impl_sub_for_point!(&Point, Point);
impl_sub_for_point!(&Point, &Point);


impl ops::Add<&Vec3> for &Point {
    type Output = Point;

    fn add(self, direction: &Vec3) -> Point {
        Point {
            x: self.x + direction.x,
            y: self.y + direction.y,
            z: self.z + direction.z,
        }
    }
}


impl ops::Neg for &Point {
    type Output = Point;

    fn neg(self) -> Point {
        Point::new(-self.x, -self.y, -self.z)
    }
}


impl From<Vec3> for Point {
    fn from(v: Vec3) -> Self {
        Point {x: v.x, y: v.y, z: v.z}
    }
}


#[derive(Debug, Clone)]
pub struct Light {
    pub location: Point,
    pub color: Color,
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

    pub fn compute_t(&self, point: &Point) -> f32 {
        // Assumes that the point lies on the ray
        ((point - &self.origin).norm_squared() / self.direction.norm_squared()).sqrt()
    }
}

#[derive(Debug, Clone)]
pub struct DiagMat3 {
    pub a: f32,
    pub b: f32,
    pub c: f32,
}

impl DiagMat3 {
    pub fn compute_inverse(&self) -> DiagMat3 {
        DiagMat3 {
            a: 1.0 / self.a,
            b: 1.0 / self.b,
            c: 1.0 / self.c,
        }
    }
}

impl ops::Mul<&Vec3> for &DiagMat3 {
    type Output = Vec3;

    fn mul(self, vector: &Vec3) -> Vec3 {
        Vec3 {
            x: vector.x * self.a,
            y: vector.y * self.b,
            z: vector.z * self.c,
        }
    }
}
