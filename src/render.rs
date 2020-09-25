use nannou::image::{DynamicImage, ImageBuffer, Rgb};

use crate::scene::{
    Scene,
    ViewingPlane,
    Sphere,
    Camera,
    Color,
    Point,
    Light,
    Plane,
    ProjectionType,
};


pub fn render(width: u32, height: u32, projection_type: ProjectionType) -> DynamicImage {
    let plane = Plane::from_y(-1.4, Color {r: 0.5, g: 0.5, b: 0.5});

    let scene = Scene {
        objects: vec![
            &Sphere {
                center: Point {x: 1.0, y: -1.5, z: 0.0},
                radius: 0.5,
                color: Color {r: 1.0, g: 0.0, b: 0.0},
            },
            &Sphere {
                center: Point {x: 0.0, y: 0.0, z: 0.0},
                radius: 0.5,
                color: Color {r: 1.0, g: 0.0, b: 0.0},
            },
            &plane,
        ],
        camera: Camera::from_z_position(-10.0, projection_type),
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
        lights: vec![
            Light {
                location: Point {x: 0.0, y: 3.0, z: 0.0},
                color: Color {r: 1.0, g: 1.0, b: 1.0},
            },
            // Light {
            //     location: Point {x: -3.0, y: 0.0, z: 0.0},
            //     color: Color {r: 1.0, g: 1.0, b: 1.0},
            // },
        ],
        ambient_strength: 0.3,
        diffuse_strength: 0.7,
    };

    let image_buf = ImageBuffer::from_fn(width, height, |i: u32, j: u32| -> Rgb<u8> {
        scene.compute_pixel(i, height - j).into()
    });

    DynamicImage::ImageRgb8(image_buf)
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
