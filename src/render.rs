use nannou::image::{DynamicImage, ImageBuffer, Rgb};

use crate::scene::*;
use crate::surface::*;
use crate::basics::*;


pub struct RenderOptions {
    pub projection_type: ProjectionType,
    pub number_of_lights: u32,
    pub camera_z_position: f32,
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

    let scene = Scene {
        objects: vec![
            // &Sphere {
            //     center: Point {x: 1.0, y: -1.5, z: 0.0},
            //     radius: 0.5,
            //     color: Color {r: 1.0, g: 0.0, b: 0.0},
            //     specular_strength: 0.5,
            // },
            // &Ellipsoid {
            //     center: Point {x: 0.0, y: 0.0, z: 0.0},
            //     color: Color {r: 1.0, g: 0.0, b: 0.0},
            //     specular_strength: 0.5,
            //     scale: DiagMat3 {a: 0.75, b: 0.5, c: 0.5}
            // },
            &Cone {
                apex: Point {x: 0.0, y: 0.5, z: 0.0},
                half_angle: 0.3,
                height: 1.0,
                color: Color {r: 1.0, g: 0.0, b: 0.0},
                specular_strength: 0.5,
            },
            // &Sphere {
            //     center: Point {x: 0.0, y: 0.0, z: 0.0},
            //     radius: 0.5,
            //     color: Color {r: 1.0, g: 0.0, b: 0.0},
            //     specular_strength: 0.5,
            // },
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

    let image_buf = ImageBuffer::from_fn(width, height, |i: u32, j: u32| -> Rgb<u8> {
        scene.compute_pixel(i, height - j).into()
    });

    DynamicImage::ImageRgb8(image_buf)
}
