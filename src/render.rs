use nannou::image::{DynamicImage, ImageBuffer, Rgb};

use crate::scene::*;
use crate::surface::*;
use crate::basics::*;

#[derive(Debug, Copy, Clone)]
pub struct RenderOptions {
    pub projection_type: ProjectionType,
    pub number_of_lights: u32,
    pub camera_z_position: f32,
    pub specular_strength: f32,
}


pub fn render(width: u32, height: u32, options: &RenderOptions) -> DynamicImage {
    let plane = Plane::from_y(-1.4, Color {r: 0.5, g: 0.5, b: 0.5});
    let mut lights = vec![Light {
        location: Point {x: 0.0, y: 3.0, z: 0.0},
        color: Color {r: 1.0, g: 1.0, b: 1.0},
    }];

    if options.number_of_lights == 2 {
        lights.push(Light {
            location: Point {x: -3.0, y: 0.0, z: 0.0},
            color: Color {r: 1.0, g: 1.0, b: 1.0},
        });
    }

    let sphere_a = Sphere {
        center: Point {x: 1.0, y: -1.5, z: 0.0},
        radius: 0.5,
        color: Color {r: 1.0, g: 0.0, b: 0.0},
        specular_strength: options.specular_strength,
    };
    let ellipsoid = Ellipsoid {
        center: Point {x: 0.0, y: 0.0, z: 0.0},
        color: Color {r: 1.0, g: 0.0, b: 0.0},
        specular_strength: options.specular_strength,
        scale: DiagMat3 {a: 0.75, b: 0.5, c: 0.5}
    };
    let cone = Cone {
        apex: Point {x: -1.5, y: 1.0, z: 0.5},
        half_angle: 0.5,
        height: 0.5,
        color: Color {r: 1.0, g: 0.0, b: 0.0},
        specular_strength: options.specular_strength,
    };
    let sphere_b = Sphere {
        center: Point {x: 0.0, y: 0.0, z: 0.0},
        radius: 0.5,
        color: Color {r: 1.0, g: 0.0, b: 0.0},
        specular_strength: options.specular_strength,
    };

    let scene = Scene {
        objects: vec![
            &sphere_a,
            &ellipsoid,
            &cone,
            // &sphere_b,
            &plane,
        ],
        camera: Camera::from_z_position(options.camera_z_position, options.projection_type),
        viewing_plane: ViewingPlane {
            z: 0.0,
            x_min: -2.0,
            x_max: 2.0,
            y_min: -1.5,
            y_max: 1.5,
            width: width,
            height: height,
        },
        background_color: Color { r: 0.2, g: 0.5, b: 0.2},
        lights: lights,
        ambient_strength: 0.3,
        diffuse_strength: 0.7,
    };

    let image_vec = iproduct!(0..height, 0..width)
        .map(|p: (u32, u32)| -> Vec<u8> {
            let color = scene.compute_pixel(p.1, height - p.0);
            vec![(color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8]
        })
        .flatten()
        .collect::<Vec<u8>>();

    let image_buf = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width, height, image_vec).unwrap();

    DynamicImage::ImageRgb8(image_buf)
}
