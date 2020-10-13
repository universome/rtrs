use rayon::prelude::*;
use nannou::image::{DynamicImage, RgbImage};

use crate::scene::*;
use crate::surface::*;
use crate::basics::*;
use crate::matrix::{Mat3, Transformation};

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub projection_type: ProjectionType,
    pub number_of_lights: u32,
    pub camera_options: CameraOptions,
    pub specular_strength: f32,
}

#[derive(Debug, Clone)]
pub struct CameraOptions {
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vec3,
}


pub fn render(width: u32, height: u32, options: &RenderOptions) -> DynamicImage {
    let mut lights = vec![Light {
        location: Point {x: 0.0, y: 5.0, z: 0.0},
        color: Color {r: 1.0, g: 1.0, b: 1.0},
    }];

    if options.number_of_lights == 2 {
        lights.push(Light {
            location: Point {x: -3.0, y: 0.0, z: 0.0},
            color: Color {r: 1.0, g: 1.0, b: 1.0},
        });
    }

    let lookat_transform = Transformation::create_look_at(
        &options.camera_options.position,
        options.camera_options.yaw,
        options.camera_options.pitch,
    );

    // let sphere = Sphere {
    //     center: Point {x: 1.0, y: -1.5, z: -0.5},
    //     radius: 0.5,
    //     color: Color {r: 0.0, g: 0.0, b: 1.0},
    //     specular_strength: options.specular_strength,
    // };
    // let ellipsoid = Ellipsoid {
    //     center: Point {x: 0.0, y: 0.0, z: 0.0},
    //     color: Color {r: 1.0, g: 0.0, b: 0.0},
    //     specular_strength: options.specular_strength,
    //     scale: DiagMat3 {a: 0.35, b: 0.25, c: 0.25}
    // };
    let sphere = Sphere {
        center: Point {x: 0.0, y: 0.0, z: 0.0},
        radius: 1.0,
        color: Color {r: 0.0, g: 0.0, b: 1.0},
        specular_strength: options.specular_strength,
    };
    let sphere_transform = Transformation {
        transform_mat: Mat3 {rows: [
            Vec3 {x: 0.25, y: 0.0, z: 0.0},
            Vec3 {x: 0.0, y: 0.25, z: 0.0},
            Vec3 {x: 0.0, y: 0.0, z: 0.25},
        ]},
        translation: Vec3 {x: 0.0, y: 0.0, z: 0.0},
    };
    let sphere_transform = &lookat_transform * &sphere_transform;
    let transformed_sphere = TransformedSurface::new(sphere_transform, &sphere);

    let plane = Plane::from_y(-1.4, Color {r: 0.5, g: 0.5, b: 0.5});
    let transformed_plane = TransformedSurface::new(lookat_transform.clone(), &plane);

    let cone = Cone {
        apex: Point {x: -1.0, y: 1.0, z: 0.0},
        half_angle: 0.5,
        height: 0.5,
        color: Color {r: 0.0, g: 1.0, b: 0.0},
        specular_strength: options.specular_strength,
    };
    let cone_transform = Transformation {
        transform_mat: Mat3 {rows: [
            Vec3 {x: 1.0, y: 0.0, z: 0.0},
            Vec3 {x: 0.0, y: 1.0, z: 0.0},
            Vec3 {x: 0.0, y: 0.0, z: 1.0},
        ]},
        translation: Vec3 {x: 0.0, y: 0.0, z: 0.0},
    };
    let cone_transform = &lookat_transform * &cone_transform;
    let transformed_cone = TransformedSurface::new(cone_transform, &cone);

    let scene = Scene {
        objects: vec![
            &transformed_sphere,
            &transformed_cone,
            &transformed_plane,
        ],
        camera: Camera::from_z_position(-1.0, options.projection_type, width, height),
        background_color: Color {r: 0.204, g: 0.596, b: 0.86},
        lights: lights,
        ambient_strength: 0.3,
        diffuse_strength: 0.7,
    };

    let pixels = iproduct!(0..height, 0..width)
        .collect::<Vec<(u32, u32)>>()
        .par_iter()
        .map(|p: &(u32, u32)| -> Color {
            scene.compute_pixel(p.1, height - p.0)
        })
        .collect::<Vec<Color>>();

    let mut img = RgbImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            img.put_pixel(x, y, pixels[(width * y + x) as usize].clone().into());
        }
    }

    DynamicImage::ImageRgb8(img)
}
