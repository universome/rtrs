use std::ops;
use nannou::image::{Rgb};
use derive_more;


#[derive(Debug, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn clamp(&self) -> Color {
        Color {
            r: self.r.max(0.0).min(1.0),
            g: self.g.max(0.0).min(1.0),
            b: self.b.max(0.0).min(1.0),
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
        (Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }).clamp()
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


#[derive(Debug, Clone, derive_more::Add, derive_more::Sub, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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
